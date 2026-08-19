local PhysicsComponents = require("Modules.Physics.Components")
local ConstructComponents = require("Modules.Constructs.Components")
local CoreComponents = require("Modules.Core.Components")
local ShipWeaponRegistry = require("Shared.Registries.ShipWeaponRegistry")
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

function AIWeaponSystem:getWeaponAIConfig(state)
    if state.aiConfig then
        return state.aiConfig
    end

    for _, mount in ipairs(state.turrets or {}) do
        local turret = mount.component
        local weapon = ShipWeaponRegistry:get(turret.weaponKey)
        if weapon and weapon.ai then
            return weapon.ai
        end
    end

    return {}
end

function AIWeaponSystem:getCandidates(state)
    if state.targetCandidates then
        return state.targetCandidates
    end

    local candidate = targetRecord(state.targetEntity)
    if candidate then
        return { candidate }
    end
    return {}
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

    local current = targeting and targeting:getTarget()
    local currentRecord = targetRecord(current)
    if currentRecord then
        local dx = currentRecord.position.x - origin.x
        local dy = currentRecord.position.y - origin.y
        local dz = currentRecord.position.z - origin.z
        if dx * dx + dy * dy + dz * dz > range * range then
            currentRecord = nil
        end
    end

    if not currentRecord then
        local candidates = self:getCandidates(state)
        local selected = WeaponSystem:selectNearestTarget(origin, candidates, range)
        currentRecord = selected and targetRecord(selected.entity) or nil
    end

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
