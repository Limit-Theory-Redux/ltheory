local Registry = require("Core.ECS.Registry")
local Entity = require("Core.ECS.Entity")
local QuickProfiler = require("Shared.Tools.QuickProfiler")
local ScaleConfig = require("Config.Gen.UniverseScaleConfig")
local RaycastHelper = require("Shared.Helpers.RaycastHelper")
local MiningLaser = require("Shared.Definitions.MiningLaserDefs")
local Nav = require("Modules.Navigation")
local PhysicsComponents = require("Modules.Physics.Components")
local ConstructEntities = require("Modules.Constructs.Entities")
local ConstructComponents = require("Modules.Constructs.Components")
local Economy = require("Modules.Economy.Components")
local InventoryManager = require("Modules.Economy.Managers.InventoryManager")

-- Design: doc/script/economy.md

---Mines asteroids. A ship that begins a run (`begin`) has its controls claimed and its body taken
---out of physics (kinematic, not collidable). It is given an ore spot of the rock's deposit
---(`YieldComponent:getSpots`) that no other miner of the rock works if there is one, and a circle
---of its own round the rock: in a plane through that spot, at a radius that clears the rock and the
---ship, flown one way or the other at a speed of its own. It glides onto the circle and flies it,
---nose to where its laser hits. The laser leaves the ship's nose and stops where a ray toward the
---spot meets the rock's surface (cast once a pulse, then kept on the rock); it is on while the spot
---faces the ship, and ore is taken while
---it is, at `rate` units a second. The run ends when the hold is full, the run's quantity is
---taken, the rock is empty or gone, or time is up; the body and controls go back to the ship's
---orders then and the run is marked done (`isDone`, `finish`). Runs at PreSim.
---@class MiningSystem
local MiningSystem = Class("MiningSystem", function(self)
    self:registerVars()
    self:registerEvents()
end)

function MiningSystem:registerVars()
    self.profiler = QuickProfiler("MiningSystem", false, false)
    self.rng = RNG.FromTime()
    -- Units of ore a second while the laser is on the rock.
    self.rate = 0.6
    -- The laser fires in pulses: `pulseOnS` on, `pulseOffS` off, each on a point of its own near the
    -- spot (within `pulseSpread` of the rock's radius). A full capacitor holds `pulsesPerCharge`; an
    -- empty one recharges in `rechargeS` while the ship keeps circling, and the next pulses go to the
    -- spot that faces the ship then. A run is so many short shots over several spots.
    self.pulseOnS = 2.5
    self.pulseOffS = 1.2
    self.pulseSpread = 0.15
    self.pulsesPerCharge = 5
    self.rechargeS = 6
    -- About this share of each lap the spot faces the ship (the laser's on only then): for the
    -- expected rate before any run has ended.
    self.facingShare = 0.5
    -- A run's rate counts this much in the rate observed so far (`expectedRate`); runs this short
    -- don't count.
    self.observedWeight = 0.3
    self.minObservedS = 10
    ---@type number|nil Units a second taken over whole runs, settling and pauses included
    self.observedRate = nil
    -- The circle: this far off the rock's surface plus a few ship radii, with up to `radiusSpread`
    -- of the rock's radius more, flown at `circleSpeed` (each run within `speedSpread` of it).
    self.clearance = ScaleConfig:toGameUnits(150, "starSystem")
    self.shipRadii = 3
    self.radiusSpread = 0.35
    self.circleSpeed = ScaleConfig:toGameUnits(40, "starSystem")
    self.speedSpread = 0.3
    -- The glide from where the ship arrived onto the circle: at this speed, within these seconds.
    self.settleSpeed = ScaleConfig:toGameUnits(150, "starSystem")
    self.minSettleS, self.maxSettleS = 3, 20
    -- The laser reaches the spot while the spot faces the ship within this cosine.
    self.minSpotCosine = 0.15
    ---@type table<integer, table<integer, integer>> Rock id -> spot index -> ship id working it
    self.spotUsers = {}
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

---The physics world of the navigation world: where the rocks' bodies are.
---@return Physics|nil
local function physicsWorld()
    for _, navWorld in Registry:view(Nav.Components.NavWorld) do return navWorld:getPhysics() end
    return nil
end

---@param a Vec3f
---@param b Vec3f
---@return Vec3f
local function cross(a, b)
    return Vec3f(a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x)
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

---A world point in the rock body's own frame (from its centre, along its axes), and back: a laser's
---hit is kept on the rock this way, so it rides the rock's turning between rays.
---@param body RigidBody
---@param p Position
---@return Vec3d
function MiningSystem:toLocal(body, p)
    local c, rot = body:getPos(), body:getRot()
    local right, up, forward = rot:getRight(), rot:getUp(), rot:getForward()
    local dx, dy, dz = p.x - c.x, p.y - c.y, p.z - c.z
    return Vec3d(dx * right.x + dy * right.y + dz * right.z, dx * up.x + dy * up.y + dz * up.z,
        dx * forward.x + dy * forward.y + dz * forward.z)
end

---@param body RigidBody
---@param l Vec3d
---@return Position
function MiningSystem:toWorld(body, l)
    local c, rot = body:getPos(), body:getRot()
    local right, up, forward = rot:getRight(), rot:getUp(), rot:getForward()
    return Position(c.x + right.x * l.x + up.x * l.y + forward.x * l.z, c.y + right.y * l.x + up.y * l.y + forward.y * l.z,
        c.z + right.z * l.x + up.z * l.y + forward.z * l.z)
end

---The spot of the rock's deposit for a new run: one nobody works, nearest the ship; else any.
---@param rock Entity
---@param rockBody RigidBody
---@param from Position
---@return integer|nil index
function MiningSystem:pickSpot(rock, rockBody, from)
    local spots = rock:get(Economy.Yield) and rock:get(Economy.Yield):getSpots() or {}
    if #spots == 0 then return nil end
    local users = self.spotUsers[rock.id] or {}
    local centre = rockBody:getPos()
    local toShip = Vec3f(from.x - centre.x, from.y - centre.y, from.z - centre.z):normalize()
    local best, bestScore = nil, -math.huge
    for index, local3 in ipairs(spots) do
        local dir = self:worldDirection(rockBody, local3)
        local score = dir.x * toShip.x + dir.y * toShip.y + dir.z * toShip.z
        if users[index] and alive(users[index]) then score = score - 10 end
        if score > bestScore then best, bestScore = index, score end
    end
    return best
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
    mining.startedAt = now
    local pos, centre = shipBody:getPos(), rockBody:getPos()
    mining.from = pos
    mining.fromRot = shipBody:getRot()

    -- The spot, held for this ship.
    mining.spot = self:pickSpot(rock, rockBody, pos)
    if mining.spot then
        self.spotUsers[rock.id] = self.spotUsers[rock.id] or {}
        self.spotUsers[rock.id][mining.spot] = ship.id
    end

    -- The circle: in a plane through the spot (so the ship passes over it each lap), tilted about
    -- it at random, at a radius and speed of its own, one way round or the other.
    local spotDir = mining.spot and self:worldDirection(rockBody, rock:get(Economy.Yield):getSpots()[mining.spot])
        or Vec3f(pos.x - centre.x, pos.y - centre.y, pos.z - centre.z):normalize()
    local any = Vec3f(self.rng:getUniformRange(-1, 1), self.rng:getUniformRange(-1, 1), self.rng:getUniformRange(-1, 1))
    local normal = cross(spotDir, any)
    if normal:length() < 1e-4 then normal = cross(spotDir, Vec3f(0, 1, 0)) end
    normal = normal:normalize()
    mining.axisU = spotDir
    mining.axisV = cross(normal, spotDir):normalize()
    local rockRadius = rockBody:getBoundingRadius()
    mining.radius = rockRadius * (1 + self.rng:getUniformRange(0, self.radiusSpread))
        + self.clearance + shipBody:getBoundingRadius() * self.shipRadii
    mining.direction = self.rng:getUniformRange(0, 1) < 0.5 and -1 or 1
    mining.speed = self.circleSpeed * self.rng:getUniformRange(1 - self.speedSpread, 1 + self.speedSpread)
    -- Start where the ship's direction from the rock falls on the circle.
    local rel = Vec3f(pos.x - centre.x, pos.y - centre.y, pos.z - centre.z)
    mining.angle = math.atan2(rel.x * mining.axisV.x + rel.y * mining.axisV.y + rel.z * mining.axisV.z,
        rel.x * mining.axisU.x + rel.y * mining.axisU.y + rel.z * mining.axisU.z)
    local start = self:circlePoint(mining, centre)
    local d = Vec3f(start.x - pos.x, start.y - pos.y, start.z - pos.z):length()
    mining.settleS = math.max(self.minSettleS, math.min(self.maxSettleS, d / self.settleSpeed))
    mining.charge, mining.recharging = 1, false
    mining.pulseTimer, mining.pulseOn = 0, false

    Nav.claim(ship)
    -- Kinematic and out of collisions: a ship circling just off a rock would make contacts with it.
    shipBody:setKinematic(true)
    shipBody:setCollidable(false)
    return true
end

---Moves the run to the free spot that faces the ship most, giving up the one it held.
---@param ship Entity
---@param rock Entity
---@param rockBody RigidBody
---@param mining MiningComponent
---@param at Position
function MiningSystem:changeSpot(ship, rock, rockBody, mining, at)
    local users = self.spotUsers[rock.id] or {}
    self.spotUsers[rock.id] = users
    if mining.spot and users[mining.spot] == ship.id then users[mining.spot] = nil end
    mining.spot = self:pickSpot(rock, rockBody, at)
    if mining.spot then users[mining.spot] = ship.id end
end

---@param mining MiningComponent
---@param centre Position
---@return Position
function MiningSystem:circlePoint(mining, centre)
    local c, s = math.cos(mining.angle) * mining.radius, math.sin(mining.angle) * mining.radius
    local u, v = mining.axisU, mining.axisV
    return Position(centre.x + u.x * c + v.x * s, centre.y + u.y * c + v.y * s, centre.z + u.z * c + v.z * s)
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

---Ends the run: the laser off, the spot given up, the body and controls given back.
---@param ship Entity
---@param mining MiningComponent
function MiningSystem:stop(ship, mining)
    self:laserOff(mining)
    if mining.emitter and Registry:hasEntity(mining.emitter) then Registry:destroyEntity(mining.emitter) end
    mining.emitter = nil
    local users = self.spotUsers[mining.rock]
    if users and mining.spot and users[mining.spot] == ship.id then users[mining.spot] = nil end
    local body = bodyOf(ship)
    if body then
        body:setKinematic(false)
        body:setCollidable(true)
    end
    Nav.unclaim(ship)
    mining.lasing = false
    mining.done = true

    local elapsed = require("Modules.Economy.Systems.MarketplaceSystem"):now() - (mining.startedAt or 0)
    if mining.startedAt and elapsed >= self.minObservedS then
        local rate = mining.taken / elapsed
        self.observedRate = self.observedRate and (self.observedWeight * rate + (1 - self.observedWeight) * self.observedRate) or rate
    end
end

---Units of ore a run takes a second on average, settling, pulse pauses, recharges and the spot
---turned away included: what runs have taken so far, or before any has ended, an estimate from the
---pulses. Far below `rate`, the rate while the laser is on the rock.
---@return number
function MiningSystem:expectedRate()
    if self.observedRate and self.observedRate > 0 then return self.observedRate end
    local pulses = self.pulsesPerCharge
    local duty = pulses * self.pulseOnS / (pulses * (self.pulseOnS + self.pulseOffS) + self.rechargeS)
    return self.rate * duty * self.facingShare
end

---@param mining MiningComponent
function MiningSystem:laserOff(mining)
    if mining.beam and Registry:hasEntity(mining.beam) then Registry:destroyEntity(mining.beam) end
    mining.beam = nil
end

---@param dt number
function MiningSystem:update(dt)
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

    -- Fly: glide onto the circle, then round it.
    local centre = rockBody:getPos()
    mining.angle = mining.angle + mining.direction * mining.speed / mining.radius * dt
    local onCircle = self:circlePoint(mining, centre)
    mining.settle = math.min(1, mining.settle + dt / mining.settleS)
    local s = smoothstep(mining.settle)
    local from = mining.from
    local at = Position(from.x + (onCircle.x - from.x) * s, from.y + (onCircle.y - from.y) * s,
        from.z + (onCircle.z - from.z) * s)

    -- The capacitor: an empty one recharges in full before the laser fires again, and the next
    -- pulses go to the spot that faces the ship then.
    if mining.recharging then
        mining.charge = math.min(1, mining.charge + dt / self.rechargeS)
        if mining.charge >= 1 then
            mining.recharging = false
            mining.pulseTimer, mining.pulseOn = 0, false
            self:changeSpot(ship, rock, rockBody, mining, at)
        end
    end

    -- The pulses: on and off in turn while there is charge; each new pulse spends a share of it and
    -- takes a point of its own near the spot.
    local firing = false
    if s >= 1 and not mining.recharging then
        mining.pulseTimer = mining.pulseTimer - dt
        if mining.pulseTimer <= 0 then
            mining.pulseOn = not mining.pulseOn
            mining.pulseTimer = mining.pulseOn and self.pulseOnS * self.rng:getUniformRange(0.7, 1.3)
                or self.pulseOffS * self.rng:getUniformRange(0.6, 1.4)
            if mining.pulseOn then
                mining.charge = mining.charge - 1 / self.pulsesPerCharge
                local spread = self.pulseSpread
                mining.pulseOffset = Vec3f(self.rng:getUniformRange(-spread, spread), self.rng:getUniformRange(-spread, spread),
                    self.rng:getUniformRange(-spread, spread))
                -- A new pulse finds its point on the surface again.
                mining.hitLocal = nil
            elseif mining.charge <= 1e-6 then
                mining.charge, mining.recharging = 0, true
            end
        end
        firing = mining.pulseOn
    end

    -- Aim at the spot when it faces the ship, else at the rock's centre; the nose follows the aim.
    local spotPoint = firing and self:visibleSpot(rock, rockBody, mining, at) or nil
    local aim = spotPoint or mining.lastAim or centre
    if spotPoint then mining.lastAim = spotPoint end
    local toAim = Vec3f(aim.x - at.x, aim.y - at.y, aim.z - at.z):normalize()
    local facing = Quat.FromLook(toAim, mining.axisV)
    mining.facing = mining.facing and mining.facing:slerp(facing, math.min(1, dt * 2)) or facing
    shipBody:setKinematicTarget(at, s < 1 and mining.fromRot:slerp(mining.facing, s) or mining.facing)

    -- The laser, from the nose to where a ray toward the spot meets the rock. The ray is cast once a
    -- pulse; its hit is kept in the rock's frame and stays on that point of the surface while the
    -- pulse lasts (a ray a frame for every miner is what mining cost most).
    local nose = self:nosePoint(shipBody, at, mining.facing)
    local hit = nil
    if spotPoint then
        if mining.hitLocal == nil then
            local surface = self:surfaceHit(nose, spotPoint, rockBody, shipBody)
            mining.hitLocal = surface and self:toLocal(rockBody, surface) or false
        end
        hit = mining.hitLocal and self:toWorld(rockBody, mining.hitLocal) or nil
    end
    mining.lasing = hit ~= nil
    if hit then
        self:laserOn(ship, mining, nose, hit)
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

---The point a little inside the rock along the run's spot, when the spot faces `from` enough for
---the laser to reach it; nil when it is round the far side.
---@param rock Entity
---@param rockBody RigidBody
---@param mining MiningComponent
---@param from Position
---@return Position|nil
function MiningSystem:visibleSpot(rock, rockBody, mining, from)
    local spots = rock:get(Economy.Yield) and rock:get(Economy.Yield):getSpots() or {}
    local local3 = mining.spot and spots[mining.spot]
    if not local3 then return nil end
    local centre, radius = rockBody:getPos(), rockBody:getBoundingRadius()
    local dir = self:worldDirection(rockBody, local3)
    local toShip = Vec3f(from.x - centre.x, from.y - centre.y, from.z - centre.z):normalize()
    if dir.x * toShip.x + dir.y * toShip.y + dir.z * toShip.z < self.minSpotCosine then return nil end
    -- Inside the hull: the ray toward it meets the surface on the way; a pulse's own point is off the
    -- spot's line by its offset.
    local depth, off = radius * 0.3, mining.pulseOffset or Vec3f(0, 0, 0)
    return Position(centre.x + (dir.x + off.x) * depth, centre.y + (dir.y + off.y) * depth, centre.z + (dir.z + off.z) * depth)
end

---The ship's nose: ahead of its centre along where it faces, by most of its radius.
---@param shipBody RigidBody
---@param at Position
---@param facing Quat
---@return Position
function MiningSystem:nosePoint(shipBody, at, facing)
    local forward = facing:getForward()
    local reach = shipBody:getBoundingRadius() * 0.8
    return Position(at.x + forward.x * reach, at.y + forward.y * reach, at.z + forward.z * reach)
end

---Where the segment from the nose to `target` meets the rock's surface, or nil when it meets
---something else first or nothing.
---@param nose Position
---@param target Position
---@param rockBody RigidBody
---@param shipBody RigidBody
---@return Position|nil
function MiningSystem:surfaceHit(nose, target, rockBody, shipBody)
    local world = physicsWorld()
    if not world then return target end
    local hit = RaycastHelper:castSegment(world, nose, target, { ignoreBody = shipBody })
    if not hit or not hit.position then return nil end
    if hit.body ~= nil and hit.body ~= rockBody then
        -- A ray through the rock's own body: compare by position, the handles may differ.
        local c, p = rockBody:getPos(), hit.position
        local dx, dy, dz = p.x - c.x, p.y - c.y, p.z - c.z
        if math.sqrt(dx * dx + dy * dy + dz * dz) > rockBody:getBoundingRadius() * 1.05 then return nil end
    end
    return Position(hit.position.x, hit.position.y, hit.position.z)
end

---Points the laser from the nose to the hit, making it and its emitter when they are missing.
---@param ship Entity
---@param mining MiningComponent
---@param nose Position
---@param hit Position
function MiningSystem:laserOn(ship, mining, nose, hit)
    if not mining.emitter or not Registry:hasEntity(mining.emitter) then
        mining.emitter = Entity.Create("MiningLaserEmitter", PhysicsComponents.Transform())
    end
    mining.emitter:get(PhysicsComponents.Transform):setPos(nose)
    if not mining.beam or not Registry:hasEntity(mining.beam) then
        mining.beam = ConstructEntities.Beam(mining.rock, {}, {
            source = mining.emitter,
            target = alive(mining.rock),
            effect = MiningLaser.effect,
            visual = MiningLaser.effect.visual,
            targetPoint = hit,
        })
    end
    local beam = mining.beam:get(ConstructComponents.Beam)
    beam.targetPoint, beam.baseTargetPoint = hit, hit
    -- The laser's light sits where it hits.
    mining.beam:get(PhysicsComponents.Transform):setPos(hit)
end

return MiningSystem()
