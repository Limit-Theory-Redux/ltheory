---Weapon AI: target selection and trigger control.
---
---Tick order (owned by the host state's Sim handler; the testbed is the
---reference implementation):
---  1. contact/target lifecycle (spawn, orbit motion, respawn)
---  2. turret transform sync (parent body -> world)
---  3. physics world update
---  4. AIWeaponSystem:update   <- selects targets, holds/releases trigger
---  5. WeaponTrackingSystem:update <- aim solutions per mount
---  6. WeaponSystem:update     <- fire planning, capacitor, spawning
---  7. ProjectileSystem/BeamSystem updates <- effect simulation
---Main-game states must preserve this relative order; EventBus migration
---is deferred until main-state integration settles subscription order.
local PhysicsComponents = require("Modules.Physics.Components")
local ConstructComponents = require("Modules.Constructs.Components")
local CoreComponents = require("Modules.Core.Components")
local WeaponResolver = require("Shared.Content.WeaponResolver")
local WeaponSystem = require("Modules.Constructs.Systems.WeaponSystem")

---@class AIWeaponSystem
local AIWeaponSystem = {}

local function isValidTarget(entity)
    if not entity or not entity.isValid or not entity:isValid() then
        return false
    end

    local targetable = entity:get(ConstructComponents.Targetable)
    local health = entity:get(CoreComponents.Health)
    return targetable ~= nil
        and targetable:isEnabled()
        and (not health or not health:isDestroyed())
end

local function targetRecord(entity)
    if not isValidTarget(entity) then
        return nil
    end

    local rigidBodyComponent = entity:get(PhysicsComponents.RigidBody)
    local body = rigidBodyComponent and rigidBodyComponent:getRigidBody()
    if not body then
        return nil
    end

    local targetable = entity:get(ConstructComponents.Targetable)
    return {
        id = entity.id,
        entity = entity,
        body = body,
        position = body:getPos(),
        enabled = targetable:isEnabled(),
        sizeClass = targetable:getSizeClass(),
    }
end

local function resolveWeapon(turret)
    local weapon = WeaponResolver:resolve({
        weaponId = turret.weaponId,
        weaponRef = turret.weaponRef,
    })
    if turret.weaponRef and not weapon then
        error("unregistered procedural weapon: "
            .. tostring(turret.weaponRef.canonicalKey), 0)
    end
    assert(weapon, "unregistered weapon: " .. tostring(turret.weaponId))
    return weapon
end

function AIWeaponSystem:getWeaponAIConfig(state)
    if state.aiConfig then
        return state.aiConfig
    end

    for _, mount in ipairs(state.turrets or {}) do
        local turret = mount.component
        local weapon = resolveWeapon(turret)
        local weaponAI = weapon.ai or {}
        local firePolicy = weapon.firePolicy or {}
        return {
            targetRange = weaponAI.targetRange,
            defaultMode = firePolicy.defaultMode,
            modeBySizeClass = firePolicy.modeBySizeClass,
        }
    end

    return {}
end

-- Acquire target candidates from the ECS registry: every live entity with
-- a Targetable component (and a rigid body) is a potential contact. This
-- keeps weapon targeting generic - no scenario table required.
function AIWeaponSystem:getCandidatesFromRegistry()
    local Registry = require("Core.ECS.Registry")
    local candidates = {}
    for entity, targetable in Registry:iterEntities(ConstructComponents.Targetable) do
        local record = targetRecord(entity)
        if record then
            record.label = targetable:getTeam() .. ":" .. targetable:getSizeClass()
            candidates[#candidates + 1] = record
        end
    end
    return candidates
end

---Candidate resolution order:
---1. state.mountTargetByMount values (per-mount assignments from an
---   external coordinator) - deduplicated into a candidate set.
---2. state.targetCandidates (explicit list supplied by the host state).
---3. ECS registry scan over Targetable entities.
function AIWeaponSystem:getCandidates(state)
    -- 1. Per-mount assignments: collect distinct contacts already engaged.
    local assigned = {}
    local candidates = {}
    for _, body in pairs(state.mountTargetByMount or {}) do
        local candidate = state.contactRecords and state.contactRecords[body]
        if candidate and not assigned[candidate.id] then
            assigned[candidate.id] = true
            candidates[#candidates + 1] = candidate
        end
    end
    if #candidates > 0 then
        return candidates
    end

    -- 2. Host-supplied explicit list.
    if state.targetCandidates then
        return state.targetCandidates
    end

    -- 3. ECS registry scan.
    return self:getCandidatesFromRegistry()
end

---@param state table
---@param dt number
function AIWeaponSystem:update(state, dt)
    local control = state.control
    local targeting = state.targeting
    if not control or not control:isActive() then
        if state.aiWasActive then
            control:setTriggerHeld(false)
            state.aiWasActive = false
        end
        return
    end

    local aiConfig = self:getWeaponAIConfig(state)
    local range = aiConfig.targetRange
        or (targeting and targeting:getRange())
        or math.huge
    local origin = state.capitalBody and state.capitalBody:getPos()
    if not origin and state.turrets and state.turrets[1] then
        origin = state.turrets[1].body:getPos()
    end
    if not origin then
        control:setTriggerHeld(false)
        return
    end

    -- Per-role prioritized selection: PD hunts small contacts, line/heavy
    -- batteries prefer larger hulls; distance breaks ties. The current
    -- target is kept unless a candidate scores decisively better OR the
    -- rotation timer expires (spreads damage across contacts).
    local candidates = self:getCandidates(state)
    local current = targeting and targeting:getTarget()
    state.aiTargetRotationTimer = (state.aiTargetRotationTimer or 0) + dt
    local rotate = state.aiTargetRotationTimer >= (state.aiTargetRotationInterval or 8.0)
    if rotate then
        state.aiTargetRotationTimer = 0
        current = nil -- force re-selection, excluding nothing but resetting hysteresis
    end
    local excludeEntity = nil
    if rotate and #candidates > 1 then
        -- Prefer a different contact than the one we just left.
        excludeEntity = targeting and targeting:getTarget()
    end
    local pool = candidates
    if excludeEntity then
        local filtered = {}
        for _, candidate in ipairs(candidates) do
            if candidate.entity ~= excludeEntity then
                table.insert(filtered, candidate)
            end
        end
        if #filtered > 0 then
            pool = filtered
        end
    end
    local selected, _ = WeaponSystem:selectPriorityTarget(
        origin,
        pool,
        range,
        state.aiWeaponCombatRole,
        rotate and nil or current)
    local currentRecord = selected and targetRecord(selected.entity) or nil

    if not currentRecord then
        if targeting then
            targeting:setTarget(nil)
        end
        state.weaponTargetEntity = nil
        state.weaponTargetBody = nil
        state.aiTargetSizeClass = nil
        control:setTriggerHeld(false)
        state.aiWasActive = true
        return
    end

    if state.weaponTargetEntity ~= currentRecord.entity then
        Log.Info(string.format(
            "WeaponSystem AI target: entity %s (sizeClass %s)",
            tostring(currentRecord.entity and currentRecord.entity.id or "?"),
            tostring(currentRecord.sizeClass)))
    end
    if targeting then
        targeting:setTarget(currentRecord.entity)
    end
    state.weaponTargetEntity = currentRecord.entity
    state.weaponTargetBody = currentRecord.body
    state.aiTargetSizeClass = currentRecord.sizeClass
    control:setMode(WeaponSystem:selectFireMode(currentRecord.sizeClass, aiConfig))
    control:setTriggerHeld(true)
    state.aiWasActive = true
end

return AIWeaponSystem
