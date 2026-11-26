local Registry = require("Core.ECS.Registry")
local Entity = require("Core.ECS.Entity")
local QuickProfiler = require("Shared.Tools.QuickProfiler")
local ScaleConfig = require("Config.Gen.UniverseScaleConfig")
local MiningLaser = require("Shared.Definitions.MiningLaserDefs")
local Nav = require("Modules.Navigation")
local PhysicsComponents = require("Modules.Physics.Components")
local ConstructEntities = require("Modules.Constructs.Entities")
local Economy = require("Modules.Economy.Components")
local InventoryManager = require("Modules.Economy.Managers.InventoryManager")

-- Design: doc/script/economy.md

---Mines asteroids. A ship that begins a run (`begin`) has its controls claimed and its body made
---kinematic; it glides onto a circle just off the rock's surface and flies round it slowly, nose
---to the rock. Its laser plays on the ore spot of the rock's deposit (`YieldComponent:getSpots`)
---that faces the ship most, and ore is taken while it does, at `rate` units a second, from the
---rock's quantity into the ship's hold. The run ends when the hold is full, the run's quantity is
---taken, the rock is empty or gone, or time is up; the ship's body and controls are given back
---then and the run is marked done for whoever asked for it (`isDone`, `finish`). Runs at PreSim.
---@class MiningSystem
local MiningSystem = Class("MiningSystem", function(self)
    self:registerVars()
    self:registerEvents()
end)

function MiningSystem:registerVars()
    self.profiler = QuickProfiler("MiningSystem", false, false)
    -- Units of ore a second while the laser is on a spot.
    self.rate = 2
    -- The circle: this far off the rock's surface, flown at this speed.
    self.clearance = ScaleConfig:toGameUnits(250, "starSystem")
    self.circleSpeed = ScaleConfig:toGameUnits(40, "starSystem")
    -- Seconds the glide from where the ship arrived onto the circle takes.
    self.settleS = 4
    -- The laser reaches a spot whose direction is within this cosine of the ship's.
    self.minSpotCosine = 0.3
    self.clock = 0
end

function MiningSystem:registerEvents()
    EventBus:subscribe(Event.PreSim, self, self.onPreSim)
end

---@param e EventData
function MiningSystem:onPreSim(e)
    self:update(e:deltaTime())
end

---@param id integer
---@return Entity|nil
local function alive(id)
    local entity = Entity(id)
    return Registry:hasEntity(entity) and entity or nil
end

---@param entity Entity
---@return RigidBody|nil
local function bodyOf(entity)
    local rbCmp = entity:get(PhysicsComponents.RigidBody)
    return rbCmp and rbCmp:getRigidBody() or nil
end

---Starts a mining run of the ship at the rock, from where the ship is now.
---@param ship Entity A ship that takes navigation orders, with an inventory
---@param rock Entity An asteroid with Yield, Item and Quantity
---@param quantity integer The most to take
---@param maxSeconds number The longest the run lasts
---@param now number Simulated seconds
---@return boolean begun False when the ship mines already or a body is missing
function MiningSystem:begin(ship, rock, quantity, maxSeconds, now)
    if ship:get(Economy.Mining) then return false end
    local shipBody, rockBody = bodyOf(ship), bodyOf(rock)
    if not shipBody or not rockBody then return false end

    local mining = ship:add(Economy.Mining(rock.id, rock:get(Economy.Item):getItem(), quantity, now + maxSeconds))
    local pos, centre = shipBody:getPos(), rockBody:getPos()
    mining.from = pos
    mining.fromRot = shipBody:getRot()
    mining.radius = rockBody:getBoundingRadius() + self.clearance
    mining.angle = math.atan2(pos.z - centre.z, pos.x - centre.x)
    -- The circle passes at the latitude of the deposit's first spot, so the laser reaches a spot
    -- on each lap; a deposit without spots is circled at the height the ship came at.
    local spots = rock:get(Economy.Yield) and rock:get(Economy.Yield):getSpots() or {}
    local latitude = spots[1] and self:worldDirection(rockBody, spots[1]).y or (pos.y - centre.y) / mining.radius
    mining.height = math.max(-0.7, math.min(0.7, latitude)) * mining.radius
    Nav.claim(ship)
    shipBody:setKinematic(true)
    return true
end

---@param ship Entity
---@return MiningComponent|nil
function MiningSystem:runOf(ship)
    return ship:get(Economy.Mining)
end

---Takes the ship's run off it, ending it first if it hasn't ended.
---@param ship Entity
---@return integer taken Units the run took
function MiningSystem:finish(ship)
    local mining = ship:get(Economy.Mining)
    if not mining then return 0 end
    if not mining.done then self:stop(ship, mining) end
    local taken = mining.taken
    ship:remove(Economy.Mining)
    return taken
end

---Ends the run: the laser off, the body and controls given back.
---@param ship Entity
---@param mining MiningComponent
function MiningSystem:stop(ship, mining)
    self:laserOff(mining)
    local body = bodyOf(ship)
    if body then body:setKinematic(false) end
    Nav.unclaim(ship)
    mining.lasing = false
    mining.done = true
end

---@param mining MiningComponent
function MiningSystem:laserOff(mining)
    if mining.beam and Registry:hasEntity(mining.beam) then Registry:destroyEntity(mining.beam) end
    mining.beam = nil
end

---@param dt number
function MiningSystem:update(dt)
    self.clock = self.clock + dt
    self.profiler:start()
    local runs = {}
    for ship, mining in Registry:view(Economy.Mining) do runs[#runs + 1] = { ship, mining } end
    for _, run in ipairs(runs) do
        if not run[2].done then self:updateRun(run[1], run[2], dt) end
    end
    self.profiler:stop()
end

---@param t number
---@return number
local function smoothstep(t)
    t = math.max(0, math.min(1, t))
    return t * t * (3 - 2 * t)
end

---@param ship Entity
---@param mining MiningComponent
---@param dt number
function MiningSystem:updateRun(ship, mining, dt)
    local rock = alive(mining.rock)
    local rockBody = rock and bodyOf(rock)
    local deposit = rock and rock:get(Economy.Quantity)
    local shipBody = bodyOf(ship)
    if not rockBody or not deposit or not shipBody then return self:stop(ship, mining) end

    -- Fly: glide onto the circle, then round it, nose to the rock.
    local centre = rockBody:getPos()
    mining.angle = mining.angle + self.circleSpeed / mining.radius * dt
    local onCircle = Position(
        centre.x + math.cos(mining.angle) * mining.radius,
        centre.y + mining.height,
        centre.z + math.sin(mining.angle) * mining.radius)
    mining.settle = math.min(1, mining.settle + dt / self.settleS)
    local s = smoothstep(mining.settle)
    local from = mining.from
    local at = Position(from.x + (onCircle.x - from.x) * s, from.y + (onCircle.y - from.y) * s,
        from.z + (onCircle.z - from.z) * s)
    local toRock = Vec3f(centre.x - at.x, centre.y - at.y, centre.z - at.z):normalize()
    local facing = Quat.FromLook(toRock, Vec3f(0, 1, 0))
    shipBody:setKinematicTarget(at, s < 1 and mining.fromRot:slerp(facing, s) or facing)

    -- Lase the spot that faces the ship most, once on the circle.
    local spot = s >= 1 and self:visibleSpot(rock, rockBody, at) or nil
    mining.lasing = spot ~= nil
    if spot then
        self:laserOn(ship, mining, spot)
        local inventory = ship:get(Economy.Inventory)
        local room = InventoryManager:getRoomFor(inventory, mining.itemId)
        mining.extracted = mining.extracted + self.rate * dt
        local whole = math.min(math.floor(mining.extracted), deposit:getQuantity(), room, mining.quantity - mining.taken)
        if whole > 0 then
            deposit:setQuantity(deposit:getQuantity() - whole)
            InventoryManager:add(inventory, mining.itemId, whole)
            mining.taken = mining.taken + whole
            mining.extracted = mining.extracted - whole
        end
    else
        self:laserOff(mining)
    end

    local inventory = ship:get(Economy.Inventory)
    local full = InventoryManager:getRoomFor(inventory, mining.itemId) <= 0
    local now = require("Modules.Economy.Systems.MarketplaceSystem"):now()
    if full or mining.taken >= mining.quantity or deposit:getQuantity() <= 0 or now >= mining.untilTime then
        self:stop(ship, mining)
    end
end

---The point on the rock's surface of the ore spot that faces `from` most, or nil when none faces it
---enough to be reached.
---@param rock Entity
---@param rockBody RigidBody
---@param from Position
---@return Position|nil
function MiningSystem:visibleSpot(rock, rockBody, from)
    local yield = rock:get(Economy.Yield)
    if not yield then return nil end
    local centre, radius = rockBody:getPos(), rockBody:getBoundingRadius()
    local toShip = Vec3f(from.x - centre.x, from.y - centre.y, from.z - centre.z):normalize()
    local best, bestCos = nil, self.minSpotCosine
    for _, local3 in ipairs(yield:getSpots()) do
        local dir = self:worldDirection(rockBody, local3)
        local cosine = dir.x * toShip.x + dir.y * toShip.y + dir.z * toShip.z
        if cosine > bestCos then best, bestCos = dir, cosine end
    end
    if not best then return nil end
    -- A little inside the bounding sphere, where the rock's rough surface is.
    local depth = radius * 0.9
    return Position(centre.x + best.x * depth, centre.y + best.y * depth, centre.z + best.z * depth)
end

---A direction in the body's axes, in world axes.
---@param body RigidBody
---@param local3 Vec3f
---@return Vec3f
function MiningSystem:worldDirection(body, local3)
    local rot = body:getRot()
    local right, up, forward = rot:getRight(), rot:getUp(), rot:getForward()
    return Vec3f(
        right.x * local3.x + up.x * local3.y + forward.x * local3.z,
        right.y * local3.x + up.y * local3.y + forward.y * local3.z,
        right.z * local3.x + up.z * local3.y + forward.z * local3.z)
end

---Points the laser at the spot, making it when it is off.
---@param ship Entity
---@param mining MiningComponent
---@param spot Position
function MiningSystem:laserOn(ship, mining, spot)
    if not mining.beam or not Registry:hasEntity(mining.beam) then
        mining.beam = ConstructEntities.Beam(mining.rock, {}, {
            source = ship,
            target = alive(mining.rock),
            effect = MiningLaser.effect,
            visual = MiningLaser.effect.visual,
            targetPoint = spot,
        })
    end
    local beam = mining.beam:get(require("Modules.Constructs.Components").Beam)
    beam.targetPoint, beam.baseTargetPoint = spot, spot
    -- The laser's light sits where it hits.
    mining.beam:get(PhysicsComponents.Transform):setPos(spot)
end

return MiningSystem()
