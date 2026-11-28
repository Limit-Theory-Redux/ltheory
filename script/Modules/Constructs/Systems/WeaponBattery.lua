---One armed ship's weapons at run time.
---
---The weapon systems (AIWeaponSystem, WeaponTrackingSystem, WeaponSystem,
---ProjectileSystem, BeamSystem) keep no state of their own: they are called
---with a duck-typed `state` that they read and write. A WeaponBattery is that
---state for one ship: its weapon control, targeting, tracking and capacitor
---components, its turrets, the line-of-sight obstacles, the live beams and
---projectiles, the target candidates and the per-mount target assignment;
---plus the callbacks the systems call (spawnBeam, spawnProjectile, removeBeam,
---removeProjectile, onTargetDestroyed).
---
---A battery is built from an ArmedShipBuilder construction and run by a
---WeaponCombatSystem, which feeds it target candidates and calls the systems
---in order each frame. The battery never destroys its ship: the ship is a
---ConstructManager handle owned by whoever built it.
local Registry = require("Core.ECS.Registry")
local Entity = require("Core.ECS.Entity")
local PhysicsComponents = require("Modules.Physics.Components")
local CoreComponents = require("Modules.Core.Components")
local ConstructComponents = require("Modules.Constructs.Components")
local ConstructEntities = require("Modules.Constructs.Entities")
local WeaponSystem = require("Modules.Constructs.Systems.WeaponSystem")
local WeaponRegistry = require("Shared.Registries.WeaponRegistry")
local Pulse = require("Legacy.GameObjects.Entities.Effects.Pulse")
local ScaleConfig = require("Config.Gen.UniverseScaleConfig")

-- Ships at least this big (bounding radius, metres) test their own hull in their batteries' line of
-- sight; smaller ones leave it out and rely on their turrets' arcs.
local SELF_OCCLUSION_MIN_RADIUS_M = 400

-- Where surface target points wander and how far they may face away from the
-- shooter (see WeaponSystem:sampleTargetPoint).
local DEFAULT_TARGET_POINT = {
    motionAmplitude = 0.08,
    motionFrequency = 0.60,
}

-- The engagement range is this many times the longest weapon range: contacts
-- further out are not tracked at all.
local ENGAGEMENT_RANGE_FACTOR = 2.0

-- Pulse effect structs come from a memory pool and are never freed by the
-- projectile code (Pulse's destructor expects a reference-counted source),
-- so spent ones are kept here and reused.
local freePulses = {}

local function acquirePulse()
    local count = #freePulses
    if count > 0 then
        local pulse = freePulses[count]
        freePulses[count] = nil
        return pulse
    end
    return Pulse:new()
end

local function releasePulse(pulse)
    if pulse then
        freePulses[#freePulses + 1] = pulse
    end
end

---@class WeaponBatteryOptions
---@field name string|nil shown in logs and overlays (default: "battery")
---@field team string|nil contacts of this Targetable team are not engaged
---@field aiActive boolean|nil the weapon AI picks targets and fires (default: the control's own setting)
---@field targetPoint table|nil {motionAmplitude, motionFrequency, minFacingDot} for surface target points
---@field engagementRange number|nil contacts further away are ignored (default: twice the longest weapon range)
---@field aiConfig table|nil AIWeaponSystem config override {targetRange, defaultMode, modeBySizeClass}
---@field aiTargetRotationInterval number|nil seconds between AI focus target rotations (default 8)

---@class WeaponBattery
---@field name string
---@field construction ArmedShipConstruction
---@field ship Entity
---@field shipBody RigidBody
---@field world Physics|nil set by WeaponCombatSystem:addBattery
---@field capacitorOwnerKey string|nil unique per battery, set by WeaponCombatSystem:addBattery
---@field projectiles table[]
---@field beams table[]
---@field lightEffects Entity[]
---@field targetCandidates table[]
---@overload fun(construction: ArmedShipConstruction, options: WeaponBatteryOptions|nil): WeaponBattery
local WeaponBattery = Class("WeaponBattery", function(self, construction, options)
    assert(type(construction) == "table" and construction.shipHandle,
        "a weapon battery needs an armed ship construction (ArmedShipBuilder:build)")
    options = options or {}

    self.name = options.name or "battery"
    self.construction = construction
    self.team = options.team
    self.ship = construction.ship
    self.shipHandle = construction.shipHandle
    self.shipBody = construction.shipBody
    self.shipBodyComponent = construction.shipHandle.bodyComponent
    self.radius = math.max(construction.shipRadius or 0, 0.0001)

    -- The state the weapon systems read
    self.capitalBody = construction.shipBody
    self.control = construction.control
    self.targeting = construction.targeting
    self.weaponTrackingComponent = construction.weaponTrackingComponent
    self.weaponCapacitor = construction.capacitor
    self.turrets = construction.turrets
    self.turretsById = construction.turretsById
    self.mountIds = construction.mountIds
    self.mountCount = #construction.mountIds
    self.weapon = construction.defaultWeapon
    self.aiWeaponCombatRole = construction.defaultWeapon and construction.defaultWeapon.combatRole
    self.aiConfig = options.aiConfig
    self.aiTargetRotationInterval = options.aiTargetRotationInterval
    self.targetPointOptions = options.targetPoint or DEFAULT_TARGET_POINT
    -- A large ship's own hull can stand between a battery and its target (a superstructure, a
    -- turret on the far side), so its line of sight tests the hull. A small ship's can't, past what
    -- its turrets' arcs already forbid: its line of sight leaves the ship out, one ray in the engine.
    if self.radius >= ScaleConfig:toGameUnits(SELF_OCCLUSION_MIN_RADIUS_M, "starSystem") then
        self.losObstacles = {
            -- No fixed position: the hull test follows the ship's body.
            { body = construction.shipBody, kind = "hull", radius = self.radius },
        }
    else
        self.losObstacles = {}
        self.losIgnoreBody = construction.shipBody
    end
    self.projectiles = {}
    self.beams = {}
    self.lightEffects = {}
    self.lastShotOrder = {}
    self.targetCandidates = {}

    -- Per-mount combat role, for spreading the mounts over the contacts
    self.weaponRoleByMount = {}
    for _, loadout in ipairs(construction.loadout or {}) do
        if loadout.mountId and loadout.weapon then
            self.weaponRoleByMount[loadout.mountId] = loadout.weapon.combatRole
        end
    end

    local longestRange = self.weapon and self.weapon.range or 0
    for _, turret in ipairs(self.turrets) do
        local weapon = WeaponSystem:resolveWeapon(turret.component)
        longestRange = math.max(longestRange, weapon and weapon.range or 0)
    end
    self.longestRange = longestRange
    self.engagementRange = options.engagementRange or longestRange * ENGAGEMENT_RANGE_FACTOR

    self.manualTrigger = false
    if options.aiActive ~= nil then
        self:setAIActive(options.aiActive)
    end
end)

---False once the ship is gone (its construct was destroyed).
---@return boolean
function WeaponBattery:isValid()
    return self.ship ~= nil and self.ship:isValid()
end

---@return boolean
function WeaponBattery:isAIActive()
    return self.control:isActive()
end

---Turn the weapon AI on (it picks the focus target, the fire mode and holds
---the trigger) or off (the turrets keep tracking; `setTriggerHeld` fires).
---@param active boolean
function WeaponBattery:setAIActive(active)
    active = active == true
    self.control:setActive(active)
    if self.targeting then
        self.targeting:setAutoAcquireEnabled(active)
    end
    self.control:setSequenceIndex(1)
    self.manualTrigger = false
    self.control:setTriggerHeld(false)
    -- No stale focus from the other mode
    self.weaponTargetEntity = nil
    self.weaponTargetBody = nil
end

---The fire mode while the AI is off (the AI picks its own).
---@param mode integer Enums.Weapon.FireMode
function WeaponBattery:setFireMode(mode)
    if self.control:isActive() then
        return
    end
    self.control:setMode(mode)
    self.control:setSequenceIndex(1)
end

---Hold or release the trigger while the AI is off. Releasing it starts the
---firing sequence over.
---@param held boolean
function WeaponBattery:setTriggerHeld(held)
    held = held == true
    if self.manualTrigger and not held then
        self.control:setSequenceIndex(1)
    end
    self.manualTrigger = held
end

---Start the firing sequence over and forget the last impacts.
function WeaponBattery:resetSequence()
    self.control:setSequenceIndex(1)
    self.lastImpact = nil
    self.lastBeamImpact = nil
end

---Charge and capacity of all capacitor banks, and of the first bank of each
---group (laser, plasma).
---@return { charge: number, maxCharge: number, laser: table|nil, plasma: table|nil }
function WeaponBattery:getCapacitorSummary()
    local summary = { charge = 0, maxCharge = 0 }
    local capacitor = self.weaponCapacitor
    if not capacitor then
        return summary
    end
    for _, bank in ipairs(capacitor:getBanks()) do
        summary.charge = summary.charge + bank.charge
        summary.maxCharge = summary.maxCharge + bank.maxCharge
    end
    summary.laser = capacitor:getBanks(Enums.Weapon.CapacitorGroup.Laser)[1]
    summary.plasma = capacitor:getBanks(Enums.Weapon.CapacitorGroup.Plasma)[1]
    return summary
end

---The surface triangles and target-point seed of a contact, once: weapons aim
---at points on a ship's hull rather than at its centre. Contacts without a
---generated mesh are aimed at their centre.
---@param candidate table a target candidate record (cached by WeaponCombatSystem)
local function resolveTargetShape(candidate)
    if candidate.shapeResolved then
        return
    end
    candidate.shapeResolved = true
    local entity = candidate.entity
    local shipData = entity:get(ConstructComponents.ShipData)
    local mesh = shipData and shipData:getGeneratedMesh()
    if mesh then
        candidate.surface = WeaponSystem:buildTargetSurface(mesh)
    end
    -- Same seed TargetGenerator gives its targets
    local seedComponent = entity:get(CoreComponents.Seed)
    local pointRng = RNG.Create(seedComponent and seedComponent:getSeed() or candidate.id)
    candidate.targetPointSeed = pointRng:getInt(0, 2147483646)
end

---Spread the mounts over the contacts (each mount picks the best contact for
---its role, in range, else the nearest one), and choose the focus target
---while the AI is off. Runs before the physics step; the AI picks its own
---focus after it.
function WeaponBattery:assignTargets()
    local candidates = self.targetCandidates
    local aiActive = self.control:isActive()
    -- The commander's choice (a fighting job's target), which the mounts take while it is in reach.
    local focusId = self.control:getFocusTarget()
    self.focusTargetEntity = focusId and Entity(focusId) or nil

    if #candidates == 0 then
        self.mountTargetByMount = nil
        self.mountTargetEntities = nil
        self.contactRecords = nil
        self.contactTrackKeys = nil
        self.mountSurfaces = nil
        self.mountTargetPointSeeds = nil
        self.contactTracks = nil
        if not aiActive then
            self.weaponTargetEntity = nil
            self.weaponTargetBody = nil
        end
        self.control:setTriggerHeld(false)
        return
    end

    -- body -> candidate record, read by AIWeaponSystem and the tracking
    local mountTargetEntities = {}
    local contactTrackKeys = {}
    local liveTrackKeys = {}
    for _, candidate in ipairs(candidates) do
        local key = tostring(candidate.id)
        contactTrackKeys[candidate.body] = key
        liveTrackKeys[key] = true
        mountTargetEntities[candidate.body] = candidate
    end
    self.mountTargetEntities = mountTargetEntities
    self.contactRecords = mountTargetEntities
    self.contactTrackKeys = contactTrackKeys
    -- Drop the motion tracks of contacts that are gone
    for key in pairs(self.contactTracks or {}) do
        if not liveTrackKeys[key] then
            self.contactTracks[key] = nil
        end
    end

    local mountTargetByMount = {}
    local mountSurfaces = {}
    local mountTargetPointSeeds = {}
    local range = (self.weapon and self.weapon.range or self.longestRange) * 0.9
    for _, mountId in ipairs(self.mountIds) do
        local mount = self.turretsById[mountId]
        if mount and mount.body then
            local origin = mount.body:getPos()
            local chosen = WeaponSystem:chooseBestTarget(
                mount,
                candidates,
                origin,
                range,
                self.weaponRoleByMount[mountId],
                self.focusTargetEntity)
                -- Nothing in range: track the nearest so the turret stays ready
                or WeaponSystem:selectNearestTarget(origin, candidates, math.huge)
            if chosen then
                resolveTargetShape(chosen)
                mountTargetByMount[mountId] = chosen.body
                mountSurfaces[mountId] = chosen.surface
                mountTargetPointSeeds[mountId] = chosen.targetPointSeed
            end
        end
    end
    self.mountTargetByMount = mountTargetByMount
    self.mountSurfaces = mountSurfaces
    self.mountTargetPointSeeds = mountTargetPointSeeds

    if aiActive then
        -- AIWeaponSystem holds the trigger again when it has a target
        self.control:setTriggerHeld(false)
        return
    end

    -- AI off: the focus is the best contact in range (kept unless another
    -- is decisively better); the player holds the trigger.
    local origin = self.shipBody:getPos()
    local focusRange = self.targeting and self.targeting:getRange() or range
    local focus = WeaponSystem:selectPriorityTarget(
        origin, candidates, focusRange, self.aiWeaponCombatRole, self.weaponTargetEntity)
    self.weaponTargetEntity = focus and focus.entity or nil
    self.weaponTargetBody = focus and focus.body or nil
    self.control:setTriggerHeld(self.manualTrigger and focus ~= nil)
end

---Move the turret bodies with the ship (they are kept out of the physics
---world, so nothing else does).
function WeaponBattery:syncTurretTransforms()
    local bodyComponent = self.shipBodyComponent
    local parentRotation = self.shipBody:getRot()
    for _, turret in ipairs(self.turrets) do
        turret.body:setPos(bodyComponent:toWorldScaled(
            turret.bodyLocalPosition or turret.localPosition))
        turret.body:setRot(parentRotation * turret.localRotation)
    end
end

---The contact a mount engages, else the battery's focus.
---@return Entity|nil entity, RigidBody|nil body
function WeaponBattery:getMountTarget(mountId)
    local body = self.mountTargetByMount and self.mountTargetByMount[mountId]
    local record = body and self.mountTargetEntities and self.mountTargetEntities[body]
    if record then
        return record.entity, record.body
    end
    return self.weaponTargetEntity, self.weaponTargetBody
end

---WeaponSystem callback: a beam weapon fired.
---@param mount table
---@param solution table
---@param weapon table
---@param shotSerial integer
function WeaponBattery:spawnBeam(mount, solution, weapon, shotSerial)
    assert(weapon.effect.kind == Enums.Weapon.Effect.Beam,
        "projectile weapons cannot spawn beams")
    local presentation = WeaponRegistry:getPresentation(weapon)
    local beamTargetPoint = solution.targetPoint or solution.position
    local entity = ConstructEntities.Beam(shotSerial, {}, {
        source = mount.entity,
        target = (self:getMountTarget(mount.mountId)),
        effect = weapon.effect,
        visual = presentation,
        damagePerSecond = WeaponRegistry:getDamagePerSecond(weapon),
        duration = weapon.effect.duration or weapon.cooldown,
        targetPoint = beamTargetPoint,
        targetPointLocal = solution.targetPointLocal,
        aimAngles = solution.aimAngles,
        swayPhase = solution.swayPhase,
        swayTime = solution.swayTime,
        swayBasis = solution.swayBasis,
    })
    table.insert(self.beams, {
        entity = entity,
        component = entity:get(ConstructComponents.Beam),
        sourceBody = mount.body,
        targetPoint = beamTargetPoint,
        targetPointLocal = solution.targetPointLocal,
        mountId = mount.mountId,
        shotSerial = shotSerial,
        lightColor = presentation.lightColor,
        lightRadius = presentation.lightRadius,
        lightIntensity = presentation.lightIntensity,
    })
end

---WeaponSystem callback: a projectile weapon fired.
---@param mount table
---@param solution table
---@param weapon table
---@param shotSerial integer
function WeaponBattery:spawnProjectile(mount, solution, weapon, shotSerial)
    local sourcePosition = mount.body:getPos()
    local sourceVelocity = mount.body:getVelocity()
    local dx = solution.position.x - sourcePosition.x
    local dy = solution.position.y - sourcePosition.y
    local dz = solution.position.z - sourcePosition.z
    local length = math.sqrt(dx * dx + dy * dy + dz * dz)
    if length < 0.000001 then
        return
    end

    local projectile = weapon.effect
    assert(projectile.kind == Enums.Weapon.Effect.Projectile, "beam weapons cannot spawn projectiles")
    local visual = projectile.visual
    local scale = projectile.speed / length
    local velocity = Vec3f(
        dx * scale + sourceVelocity.x,
        dy * scale + sourceVelocity.y,
        dz * scale + sourceVelocity.z)
    local pulse = acquirePulse()
    pulse.pos = sourcePosition
    pulse.vel = velocity
    pulse.dir = velocity:normalize()
    pulse.lifeMax = projectile.lifetime
    pulse.life = pulse.lifeMax
    pulse.dist = 0

    local targetEntity, targetBody = self:getMountTarget(mount.mountId)
    local entity = ConstructEntities.Projectile(shotSerial, {}, {
        source = mount.entity,
        effect = projectile,
        position = sourcePosition,
        velocity = velocity,
        damage = weapon.damage,
        lifetime = projectile.lifetime,
        scale = projectile.scale,
        guidance = projectile.guidance,
        targetBody = targetBody,
        targetEntity = targetEntity,
        bodyMesh = nil,
    })
    table.insert(self.projectiles, {
        entity = entity,
        body = entity:get(PhysicsComponents.RigidBody):getRigidBody(),
        component = entity:get(ConstructComponents.Projectile),
        mountId = mount.mountId,
        shotSerial = shotSerial,
        effect = pulse,
        pulseDistance = 0,
        lightColor = visual.lightColor,
        lightRadius = visual.lightRadius,
        lightIntensity = visual.lightIntensity,
        pColorR = visual.bodyColor.r,
        pColorG = visual.bodyColor.g,
        pColorB = visual.bodyColor.b,
        pulseHeadSize = visual.headSize,
        pulseTailWidth = visual.tailWidth,
        pulseTailLength = visual.tailLength,
        shaderKey = projectile.shaderKey or (projectile.archetype == "missile" and "missile"),
    })
end

---BeamSystem callback: a beam ended.
---@param index integer
function WeaponBattery:removeBeam(index)
    local beam = table.remove(self.beams, index)
    if beam and beam.entity:isValid() then
        Registry:destroyEntity(beam.entity, Registry.DESTROY_MODE.DESTROY_CHILDREN)
    end
end

---ProjectileSystem callback: a projectile hit something or ran out.
---@param index integer
function WeaponBattery:removeProjectile(index)
    local projectile = table.remove(self.projectiles, index)
    if not projectile then
        return
    end
    releasePulse(projectile.effect)
    if projectile.entity:isValid() then
        Registry:destroyEntity(projectile.entity, Registry.DESTROY_MODE.DESTROY_CHILDREN)
    end
end

function WeaponBattery:clearBeams()
    for index = #self.beams, 1, -1 do
        self:removeBeam(index)
    end
end

function WeaponBattery:clearProjectiles()
    for index = #self.projectiles, 1, -1 do
        self:removeProjectile(index)
    end
end

---ProjectileSystem/BeamSystem callback: a contact was destroyed (BeamSystem
---passes it, ProjectileSystem does not). Whoever owns the contact removes it;
---the battery stops tracking it.
---@param entity Entity|nil
function WeaponBattery:onTargetDestroyed(entity)
    if entity and self.contactTracks then
        self.contactTracks[tostring(entity.id)] = nil
    end
end

---Forget the effect lights that have expired.
function WeaponBattery:pruneLightEffects()
    local effects = self.lightEffects
    for index = #effects, 1, -1 do
        local effect = effects[index]
        if not effect or not effect:isValid() then
            table.remove(effects, index)
        end
    end
end

---Remove everything the battery spawned (beams, projectiles, effect lights)
---and release its capacitor reservation. The ship stays.
function WeaponBattery:destroy()
    self:clearBeams()
    self:clearProjectiles()
    for _, effect in ipairs(self.lightEffects) do
        if effect and effect:isValid() then
            Registry:destroyEntity(effect, Registry.DESTROY_MODE.DESTROY_CHILDREN)
        end
    end
    self.lightEffects = {}
    WeaponSystem:releaseStateCapacitorReservation(self)
    self.control:setTriggerHeld(false)
    self.targetCandidates = {}
    self.mountTargetByMount = nil
    self.mountTargetEntities = nil
    self.contactRecords = nil
    self.contactTracks = nil
    self.weaponTargetEntity = nil
    self.weaponTargetBody = nil
end

return WeaponBattery
