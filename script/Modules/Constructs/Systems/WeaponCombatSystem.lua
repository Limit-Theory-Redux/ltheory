---Weapons in the game loop: runs every armed ship's WeaponBattery each frame.
---
---The game state owns one per game (`game.weaponCombat`) and calls it around
---the physics step:
---  preUpdate(dt)   before the physics step: refresh the target candidates
---                  (every live entity with a Targetable component and a rigid
---                  body), spread each battery's mounts over its contacts,
---                  move the turrets with their ships
---  update(dt)      right after the physics step, per battery:
---                  AIWeaponSystem -> WeaponTrackingSystem -> WeaponSystem
---  postUpdate(dt)  in the post-sim step, per battery:
---                  ProjectileSystem -> BeamSystem -> expired effect lights
---  clear()         when the scene goes: every battery's beams, projectiles
---                  and lights, the death explosions, and the projectile
---                  render entity
---
---A battery engages the Targetable contacts within its engagement range,
---except its own ship and the contacts the hostility rule spares (by default
---those of its own team; see `setHostility`). Ships without a
---Targetable component (the player's ship, for now) are never shot at.
---Batteries whose ship was destroyed are dropped on the next preUpdate.
---A contact that dies explodes where it was (Config.weapons.deathExplosion,
---by its size class); ExplosionEffectSystem ages the explosions and removes
---them when they are done, so the game loop must run it.
---Nothing runs while the game is paused (dt = 0).
local Registry = require("Core.ECS.Registry")
local Entity = require("Core.ECS.Entity")
local PhysicsComponents = require("Modules.Physics.Components")
local CoreComponents = require("Modules.Core.Components")
local ConstructComponents = require("Modules.Constructs.Components")
local RenderingComponents = require("Modules.Rendering.Components")
local CameraManager = require("Modules.Cameras.Managers.CameraManager")
local AIWeaponSystem = require("Modules.Constructs.Systems.AIWeaponSystem")
local WeaponTrackingSystem = require("Modules.Constructs.Systems.WeaponTrackingSystem")
local WeaponSystem = require("Modules.Constructs.Systems.WeaponSystem")
local ProjectileSystem = require("Modules.Constructs.Systems.ProjectileSystem")
local BeamSystem = require("Modules.Constructs.Systems.BeamSystem")
local ExplosionEffectEntity = require("Modules.Constructs.Entities.ExplosionEffectEntity")
local Pulse = require("Legacy.GameObjects.Entities.Effects.Pulse")

---@class WeaponCombatSystem
---@field world Physics
---@field batteries WeaponBattery[]
---@overload fun(world: Physics): WeaponCombatSystem
local WeaponCombatSystem = Class("WeaponCombatSystem", function(self, world)
    assert(world, "the weapon combat system needs the physics world")
    self.world = world
    self.batteries = {}
    self.batteryCount = 0 -- for unique capacitor owner keys
    -- Entity id -> target candidate record. Records (and their entity handles)
    -- live as long as their contact, so the weapon systems can compare them
    -- across frames.
    self.targets = {}
    self.frame = 0
    self.renderEntity = nil
    self.projectileScratch = {}
    -- Death explosions still burning (ExplosionEffectSystem removes them)
    self.explosions = {}
    self.explosionCount = 0 -- for each explosion's seed
end)

---Run a battery from now on. Gives it the physics world and a capacitor
---owner key of its own.
---@param battery WeaponBattery
---@return WeaponBattery
function WeaponCombatSystem:addBattery(battery)
    self.batteryCount = self.batteryCount + 1
    battery.world = battery.world or self.world
    battery.capacitorOwnerKey = battery.capacitorOwnerKey
        or string.format("%s#%d", battery.name or "battery", self.batteryCount)
    table.insert(self.batteries, battery)
    self:ensureRenderEntity()
    return battery
end

---Stop running a battery and remove what it spawned (not its ship).
---@param battery WeaponBattery
function WeaponCombatSystem:removeBattery(battery)
    for index = #self.batteries, 1, -1 do
        if self.batteries[index] == battery then
            table.remove(self.batteries, index)
            battery:destroy()
            return
        end
    end
end

---@return WeaponBattery[]
function WeaponCombatSystem:getBatteries()
    return self.batteries
end

---Projectiles of all batteries, drawn by one render entity.
function WeaponCombatSystem:ensureRenderEntity()
    if self.renderEntity and self.renderEntity:isValid() then
        return
    end
    self.renderEntity = Entity.Create(
        "WeaponProjectileEffects",
        RenderingComponents.Render(function(_, blendMode)
            local projectiles = self:collectProjectiles()
            if #projectiles > 0 then
                Pulse.Render(projectiles, {
                    mode = blendMode,
                    eye = CameraManager:getEye(),
                })
            end
        end))
end

---@return table[] the live projectiles of all batteries
function WeaponCombatSystem:collectProjectiles()
    local batteries = self.batteries
    if #batteries == 1 then
        return batteries[1].projectiles
    end
    local list = self.projectileScratch
    local count = 0
    for _, battery in ipairs(batteries) do
        for _, projectile in ipairs(battery.projectiles) do
            count = count + 1
            list[count] = projectile
        end
    end
    for index = #list, count + 1, -1 do
        list[index] = nil
    end
    return list
end

---The live contacts this frame, as stable candidate records.
---@param dt number
---@return table[]
function WeaponCombatSystem:refreshTargets(dt)
    self.frame = self.frame + 1
    local frame = self.frame
    local live = {}
    for entity, targetable in Registry:iterEntities(ConstructComponents.Targetable) do
        local health = entity:get(CoreComponents.Health)
        local bodyComponent = entity:get(PhysicsComponents.RigidBody)
        local body = bodyComponent and bodyComponent:getRigidBody()
        if targetable:isEnabled() and body and not (health and health:isDestroyed()) then
            local record = self.targets[entity.id]
            if not record or record.body ~= body then
                record = {
                    id = entity.id,
                    entity = entity,
                    body = body,
                    enabled = true,
                    team = targetable:getTeam(),
                    sizeClass = targetable:getSizeClass(),
                    label = targetable:getTeam() .. ":" .. targetable:getSizeClass(),
                }
                self.targets[entity.id] = record
            end
            local position = body:getPos()
            local previous = record.position
            if previous and dt > 0 then
                record.velocity = Vec3f(
                    (position.x - previous.x) / dt,
                    (position.y - previous.y) / dt,
                    (position.z - previous.z) / dt)
            else
                record.velocity = Vec3f()
            end
            record.position = position
            record.team = targetable:getTeam()
            record.sizeClass = targetable:getSizeClass()
            local maxHealth = health and health:getMaxHealth() or 0
            -- Weakened contacts are attractive finishes (WeaponSystem:chooseBestTarget)
            record.healthFraction = maxHealth > 0 and health:getCurrentHealth() / maxHealth or 1.0
            record.frame = frame
            record.exploded = nil -- alive (again): it explodes when it dies
            live[#live + 1] = record
        end
    end
    for id, record in pairs(self.targets) do
        if record.frame ~= frame then
            self.targets[id] = nil
        end
    end
    -- Registry iteration order is a hash order: sort for repeatable choices
    table.sort(live, function(a, b) return a.id < b.id end)
    return live
end

---Whether a battery of team `own` engages a contact of team `other`. By default every other team is
---an enemy and a battery without a team engages everything; a game with factions replaces this with
---`WeaponCombatSystem.setHostility`.
---@param own string|nil
---@param other string|nil
---@return boolean
local function differentTeams(own, other)
    return own == nil or other ~= own
end

local isHostile = differentTeams

---Replaces the rule that decides whom a battery engages (nil restores the default: other teams).
---@param rule (fun(own: string|nil, other: string|nil): boolean)|nil
function WeaponCombatSystem.setHostility(rule)
    isHostile = rule or differentTeams
end

---The contacts one battery may engage.
---@param battery WeaponBattery
---@param live table[]
local function selectCandidates(battery, live)
    local candidates = battery.targetCandidates
    for index = #candidates, 1, -1 do
        candidates[index] = nil
    end
    local origin = battery.shipBody:getPos()
    local rangeSquared = battery.engagementRange * battery.engagementRange
    local shipId = battery.ship.id
    for _, record in ipairs(live) do
        if record.id ~= shipId and isHostile(battery.team, record.team) then
            local dx = record.position.x - origin.x
            local dy = record.position.y - origin.y
            local dz = record.position.z - origin.z
            if dx * dx + dy * dy + dz * dz <= rangeSquared then
                candidates[#candidates + 1] = record
            end
        end
    end
end

---Before the physics step: target candidates, per-mount targets, turrets.
---@param dt number
function WeaponCombatSystem:preUpdate(dt)
    if dt <= 0 or #self.batteries == 0 then
        return
    end
    for index = #self.batteries, 1, -1 do
        local battery = self.batteries[index]
        if not battery:isValid() then
            Log.Warn("WeaponCombatSystem: the ship of %s is gone, dropping its battery", battery.name)
            table.remove(self.batteries, index)
            battery:destroy()
        end
    end
    local live = self:refreshTargets(dt)
    for _, battery in ipairs(self.batteries) do
        selectCandidates(battery, live)
        battery:assignTargets()
        battery:syncTurretTransforms()
    end
end

---Right after the physics step: the turrets onto where their ship moved, target choice and
---trigger, aim, firing. (Turrets are out of the physics world: without this a fast ship's turrets
---trail it by the step's move.)
---@param dt number
function WeaponCombatSystem:update(dt)
    if dt <= 0 then
        return
    end
    for _, battery in ipairs(self.batteries) do
        battery:syncTurretTransforms()
        AIWeaponSystem:update(battery, dt)
        WeaponTrackingSystem:update(battery, dt)
        WeaponSystem:update(battery, dt)
    end
    self:explodeDestroyedTargets()
end

---Post-sim step: shots in flight, beams, expired effect lights.
---@param dt number
function WeaponCombatSystem:postUpdate(dt)
    if dt <= 0 then
        return
    end
    for _, battery in ipairs(self.batteries) do
        ProjectileSystem:update(battery, dt)
        BeamSystem:update(battery, dt)
        battery:pruneLightEffects()
    end
    self:explodeDestroyedTargets()
    for index = #self.explosions, 1, -1 do
        if not self.explosions[index]:isValid() then
            table.remove(self.explosions, index)
        end
    end
end

---A death explosion for every contact that died since the last check, where
---it is now (its body is still there: whoever owns the contact removes it
---later). Each contact explodes once.
function WeaponCombatSystem:explodeDestroyedTargets()
    local dead
    for _, record in pairs(self.targets) do
        if not record.exploded and record.entity:isValid() then
            local health = record.entity:get(CoreComponents.Health)
            if health and health:isDestroyed() then
                record.exploded = true
                dead = dead or {}
                dead[#dead + 1] = record
            end
        end
    end
    if not dead then
        return
    end
    -- Same seeds in the same order on every run (pairs has no fixed order)
    table.sort(dead, function(a, b) return a.id < b.id end)
    local presets = Config.weapons.deathExplosion
    for _, record in ipairs(dead) do
        local preset = presets[record.sizeClass]
        if preset then
            local pos = record.body:getPos()
            self.explosionCount = self.explosionCount + 1
            self.explosions[#self.explosions + 1] = ExplosionEffectEntity(self.explosionCount * 7919, {
                position = Position(pos.x, pos.y, pos.z),
                size = preset.size,
                duration = preset.duration,
                lightIntensity = preset.lightIntensity,
                lightRadius = preset.lightRadius,
            })
        end
    end
end

---Remove every battery (with its beams, projectiles and lights, not its ship),
---the death explosions and the projectile render entity. The system can be
---used again afterwards.
function WeaponCombatSystem:clear()
    for index = #self.batteries, 1, -1 do
        self.batteries[index]:destroy()
        self.batteries[index] = nil
    end
    for _, explosion in ipairs(self.explosions) do
        if explosion:isValid() then
            Registry:destroyEntity(explosion, Registry.DESTROY_MODE.DESTROY_CHILDREN)
        end
    end
    self.explosions = {}
    self.targets = {}
    self.projectileScratch = {}
    if self.renderEntity and self.renderEntity:isValid() then
        Registry:destroyEntity(self.renderEntity, Registry.DESTROY_MODE.DESTROY_CHILDREN)
    end
    self.renderEntity = nil
end

return WeaponCombatSystem
