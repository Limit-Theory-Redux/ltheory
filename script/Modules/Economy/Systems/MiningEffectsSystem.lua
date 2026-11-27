local Registry = require("Core.ECS.Registry")
local Entity = require("Core.ECS.Entity")
local QuickProfiler = require("Shared.Tools.QuickProfiler")
local ScaleConfig = require("Config.Gen.UniverseScaleConfig")
local Materials = require("Shared.Registries.Materials")
local MiningLaser = require("Shared.Definitions.MiningLaserDefs")
local AsteroidMeshPool = require("Modules.CelestialObjects.Systems.AsteroidMeshPool")
local PhysicsComponents = require("Modules.Physics.Components")
local RenderComp = require("Modules.Rendering.Components").Render
local ConstructComponents = require("Modules.Constructs.Components")
local ImpactEffectEntity = require("Modules.Constructs.Entities.ImpactEffectEntity")
local Economy = require("Modules.Economy.Components")

-- Design: doc/script/economy.md

---What mining looks like, apart from what it does (`MiningSystem`): the laser flickers; where it
---hits, the rock glows and sparks; chips of ore break off the spot and drift into the miner's
---hold; and a deposit's rock shrinks as it is mined out and grows back as it regrows, tumbling
---slowly all the while. Runs at PreSim, after the runs it shows have moved.
---@class MiningEffectsSystem
local MiningEffectsSystem = Class("MiningEffectsSystem", function(self)
    self:registerVars()
    self:registerEvents()
end)

function MiningEffectsSystem:registerVars()
    self.profiler = QuickProfiler("MiningEffectsSystem", false, false)
    self.rng = RNG.FromTime()
    -- Off where nobody sees them (a simulation without rendering): the effects cost the frame only.
    self.enabled = true
    self.time = 0
    -- Sparks: one every `sparkEveryS` at the spot, each a short flash.
    self.sparkEveryS = 0.15
    self.sparkS = 0.15
    self.sparkColor = Color(1.2, 0.6, 0.18, 1.0)
    self.sparkRadius = 0.0015
    self.sparkIntensity = 0.12
    -- Chips: one every `chipEveryS`, thrown off the spot and drawn into the miner.
    self.chipEveryS = 0.35
    self.chipLifeS = 8
    self.chipMinRadius = ScaleConfig:toGameUnits(3, "starSystem")
    self.chipMaxRadius = ScaleConfig:toGameUnits(9, "starSystem")
    self.chipEjectSpeed = ScaleConfig:toGameUnits(25, "starSystem")
    self.chipSeekSpeed = ScaleConfig:toGameUnits(60, "starSystem")
    self.maxChips = 60
    -- A mined-out rock keeps this share of its size.
    self.minRockScale = 0.8
    -- Rocks tumble at up to this many radians a second.
    self.maxSpin = 0.02
    ---@type { entity: Entity, rb: RigidBody, velocity: Vec3f, life: number, ship: Entity }[]
    self.chips = {}
    ---@type { entity: Entity }[]
    self.sparks = {}
    ---@type table<integer, { baseScale: number, scale: number, axis: Vec3f, spin: number }>
    self.rocks = {}
end

function MiningEffectsSystem:registerEvents()
    EventBus:subscribe(Event.PreSim, self, self.onPreSim)
end

---@param e EventData
function MiningEffectsSystem:onPreSim(e)
    self:update(e:deltaTime())
end

---@param dt number
function MiningEffectsSystem:update(dt)
    if dt <= 0 or not self.enabled then return end
    self.time = self.time + dt
    self.profiler:start()
    for ship, mining in Registry:view(Economy.Mining) do
        if mining.lasing and mining.beam and Registry:hasEntity(mining.beam) then
            self:updateLaser(ship, mining, dt)
        else
            mining.sparkTimer, mining.chipTimer = 0, 0
        end
    end
    self:updateSparks()
    self:updateChips(dt)
    self:updateRocks(dt)
    self.profiler:stop()
end

---@param ship Entity
---@param mining MiningComponent
---@param dt number
function MiningEffectsSystem:updateLaser(ship, mining, dt)
    local beam = mining.beam:get(ConstructComponents.Beam)
    local spot = beam:getTargetPoint()
    if not spot then return end

    -- A laser of its own, whose width and light flicker.
    local base = MiningLaser.effect.visual
    mining.visual = mining.visual or {
        bodyColor = base.bodyColor, lightColor = base.lightColor, beamWidth = base.beamWidth,
        lightRadius = base.lightRadius, lightIntensity = base.lightIntensity,
    }
    local flicker = 0.9 + 0.1 * math.sin(self.time * 41 + mining.rock % 7) + 0.05 * self.rng:getUniformRange(-1, 1)
    mining.visual.beamWidth = base.beamWidth * flicker
    beam.visual = mining.visual

    mining.sparkTimer = (mining.sparkTimer or 0) + dt
    while mining.sparkTimer >= self.sparkEveryS do
        mining.sparkTimer = mining.sparkTimer - self.sparkEveryS
        self:spawnSpark(spot)
    end
    mining.chipTimer = (mining.chipTimer or 0) + dt
    while mining.chipTimer >= self.chipEveryS do
        mining.chipTimer = mining.chipTimer - self.chipEveryS
        self:spawnChip(ship, mining, spot)
    end
end

---@param spot Position
function MiningEffectsSystem:spawnSpark(spot)
    local jitter = self.sparkRadius * 2
    local entity = ImpactEffectEntity(0, {
        position = Position(spot.x + self.rng:getUniformRange(-jitter, jitter), spot.y + self.rng:getUniformRange(-jitter, jitter),
            spot.z + self.rng:getUniformRange(-jitter, jitter)),
        color = self.sparkColor,
        intensity = self.sparkIntensity,
        radius = self.sparkRadius * self.rng:getUniformRange(0.6, 1.4),
        duration = self.sparkS * self.rng:getUniformRange(0.6, 1.2),
    })
    self.sparks[#self.sparks + 1] = { entity = entity }
end

---@param ship Entity
---@param mining MiningComponent
---@param spot Position
function MiningEffectsSystem:spawnChip(ship, mining, spot)
    if #self.chips >= self.maxChips then return end
    local rock = Entity(mining.rock)
    local rockBody = Registry:hasEntity(rock) and rock:get(PhysicsComponents.RigidBody):getRigidBody()
    if not rockBody then return end
    local centre = rockBody:getPos()
    -- Off the surface, outward, with a spread.
    local out = Vec3f(spot.x - centre.x, spot.y - centre.y, spot.z - centre.z):normalize()
    local spread = 0.6
    local dir = Vec3f(out.x + self.rng:getUniformRange(-spread, spread), out.y + self.rng:getUniformRange(-spread, spread),
        out.z + self.rng:getUniformRange(-spread, spread)):normalize()
    local speed = self.chipEjectSpeed * self.rng:getUniformRange(0.5, 1.2)

    local seed = self.rng:get31()
    local rb = RigidBody.CreateSphere()
    rb:setKinematic(true)
    rb:setScale(self.rng:getUniformRange(self.chipMinRadius, self.chipMaxRadius))
    rb:setPos(spot)
    local entity = Entity.Create("OreChip")
    entity:add(PhysicsComponents.Transform())
    entity:add(PhysicsComponents.RigidBody()):setRigidBody(rb)
    local mesh = AsteroidMeshPool:getFromSeed(seed):get(1e16)
    entity:add(RenderComp({ { mesh = mesh, material = Materials.Asteroid:instance() } }))
    self.chips[#self.chips + 1] = {
        entity = entity, rb = rb, velocity = Vec3f(dir.x * speed, dir.y * speed, dir.z * speed), life = 0, ship = ship,
    }
end

---Forgets the sparks that are gone: `PointLightSystem` ages a light effect and destroys it when it ends.
function MiningEffectsSystem:updateSparks()
    local kept = {}
    for _, spark in ipairs(self.sparks) do
        if Registry:hasEntity(spark.entity) then kept[#kept + 1] = spark end
    end
    self.sparks = kept
end

---A chip is thrown off, then drawn ever harder into its miner, and is gone when it reaches it.
---@param dt number
function MiningEffectsSystem:updateChips(dt)
    local kept = {}
    for _, chip in ipairs(self.chips) do
        chip.life = chip.life + dt
        local shipCmp = Registry:hasEntity(chip.ship) and chip.ship:get(PhysicsComponents.RigidBody)
        local shipBody = shipCmp and shipCmp:getRigidBody()
        local alive = shipBody ~= nil and chip.life < self.chipLifeS
        if alive then
            local pos, target = chip.rb:getPos(), shipBody:getPos()
            local to = Vec3f(target.x - pos.x, target.y - pos.y, target.z - pos.z)
            local distance = to:length()
            if distance < shipBody:getBoundingRadius() * 1.5 then
                alive = false
            else
                local pull = math.min(1, chip.life / 1.5)
                local seek = to:normalize()
                local blend = math.min(1, dt * 1.5 * pull)
                local v = chip.velocity
                chip.velocity = Vec3f(v.x + (seek.x * self.chipSeekSpeed - v.x) * blend,
                    v.y + (seek.y * self.chipSeekSpeed - v.y) * blend, v.z + (seek.z * self.chipSeekSpeed - v.z) * blend)
                v = chip.velocity
                chip.rb:setPos(Position(pos.x + v.x * dt, pos.y + v.y * dt, pos.z + v.z * dt))
            end
        end
        if alive then
            kept[#kept + 1] = chip
        elseif Registry:hasEntity(chip.entity) then
            Registry:destroyEntity(chip.entity)
        end
    end
    self.chips = kept
end

---Each deposit's rock: its size follows what is left of it, and it tumbles about an axis of its own.
---@param dt number
function MiningEffectsSystem:updateRocks(dt)
    for rock, yield, quantity in Registry:iterEntities(Economy.Yield, Economy.Quantity) do
        local rbCmp = rock:get(PhysicsComponents.RigidBody)
        local body = rbCmp and rbCmp:getRigidBody()
        if body then
            local look = self.rocks[rock.id]
            if not look then
                local rng = RNG.Create(rock.id)
                local axis = Vec3f(rng:getUniformRange(-1, 1), rng:getUniformRange(-1, 1), rng:getUniformRange(-1, 1)):normalize()
                look = { baseScale = body:getScale(), scale = body:getScale(), axis = axis, spin = rng:getUniformRange(0.3, 1) * self.maxSpin }
                self.rocks[rock.id] = look
            end
            local share = yield:getMaxQuantity() > 0 and quantity:getQuantity() / yield:getMaxQuantity() or 1
            local scale = look.baseScale * (self.minRockScale + (1 - self.minRockScale) * share)
            if math.abs(scale - look.scale) > look.baseScale * 0.005 then
                look.scale = scale
                body:setScale(scale)
                local transform = rock:get(PhysicsComponents.Transform)
                if transform then transform:setScale(scale) end
            end
            local rot = body:getRot():mul(Quat.FromAxisAngle(look.axis, look.spin * dt))
            body:setKinematicTarget(body:getPos(), rot)
        end
    end
end

return MiningEffectsSystem()
