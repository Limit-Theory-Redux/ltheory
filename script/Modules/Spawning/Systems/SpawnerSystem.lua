local Registry            = require("Core.ECS.Registry")
local QuickProfiler       = require("Shared.Tools.QuickProfiler")
local PhysicsComponents   = require("Modules.Physics.Components")
local ConstructComponents = require("Modules.Constructs.Components")
local Items               = require("Shared.Registries.Items")
local StationEconomies    = require("Shared.Definitions.StationEconomyDefs")
local OreField            = require("Shared.Helpers.OreField")
local EconomySetup        = require("Modules.Economy.Managers.EconomySetup")
local ScaleConfig         = require("Config.Gen.UniverseScaleConfig")
-- Required for their event subscriptions: they run the economy and the dockings at PreSim.
local _                   = require("Modules.Economy.Systems")
local _                   = require("Modules.Docking.Systems")

-- Design: doc/script/spawning.md

---@class SpawnHooks
---@field onSpawned fun(entity: Entity, label: string|nil, kind: string)|nil Told of every station, agent and ore field spawned

---@class SpawnedStation
---@field entity Entity
---@field handle table The ConstructManager handle
---@field name string
---@field kind string A key of `StationEconomyDefs`

---Spawns what lives in a star system's economy: stations of an economic kind, ore fields, and ships
---that become traders or miners. It is the one place that does: scenarios, testbeds and the debug
---console call it directly, and the system population built from a ruleset goes through it too.
---Stations and ships are made by the scene's `ConstructManager` and rocks by `OreField`; the
---spawner keeps them, so `clear` takes them all down with the scene. A new ship becomes an agent
---after the next physics step (at PostSim), when its body is in the query index navigation reads.
---@class SpawnerSystem
local SpawnerSystem = Class("SpawnerSystem", function(self)
    self:registerVars()
    self:registerEvents()
end)

function SpawnerSystem:registerVars()
    self.profiler = QuickProfiler("SpawnerSystem", false, false)
    self.rng = RNG.FromTime()
    -- A station's radius, in metres, unless its spawn names another.
    self.stationRadiusM = 3000
    -- What a ship of each role starts with: its hold's mass capacity and its credits.
    self.roles = {
        [Enums.EconomyAgentRole.Trader] = { label = "Trader", capacity = 600, credits = 8000 },
        [Enums.EconomyAgentRole.Miner] = { label = "Miner", capacity = 480, credits = 500 },
    }
    self:reset()
end

---Forgets the scene (what `attach` and the spawns set up), without destroying anything.
function SpawnerSystem:reset()
    self.world, self.constructs, self.hooks = nil, nil, {}
    ---@type SpawnedStation[]
    self.stations = {}
    ---@type { entity: Entity, handle: table, name: string, role: EconomyAgentRole }[]
    self.agents = {}
    ---@type Entity[]
    self.rocks = {}
    self.pending = {}
    self.counts = {}
end

function SpawnerSystem:registerEvents()
    EventBus:subscribe(Event.PostSim, self, self.onPostSim)
end

---Gives the spawner the scene to spawn into. One scene at a time.
---@param world Physics
---@param constructs ConstructManager The scene's, which makes and destroys the stations and ships
---@param hooks SpawnHooks|nil
function SpawnerSystem:attach(world, constructs, hooks)
    assert(not self.world, "SpawnerSystem:attach: a scene is attached already (clear it first)")
    self.world, self.constructs, self.hooks = world, constructs, hooks or {}
end

---@return boolean
function SpawnerSystem:isAttached()
    return self.world ~= nil
end

---Destroys everything the spawner made in the scene and forgets the scene. Call it after
---`Nav.detach`, while the bodies are still in the physics world.
function SpawnerSystem:clear()
    if not self.world then return end
    for _, agent in ipairs(self.agents) do self.constructs:destroy(agent.handle) end
    for _, station in ipairs(self.stations) do self.constructs:destroy(station.handle) end
    for _, rock in ipairs(self.rocks) do
        if Registry:hasEntity(rock) then
            local rbCmp = rock:get(PhysicsComponents.RigidBody)
            if rbCmp and rbCmp:getRigidBody() then self.world:removeRigidBody(rbCmp:getRigidBody()) end
            Registry:destroyEntity(rock)
        end
    end
    self:reset()
end

---@param entity Entity
---@param label string|nil
---@param kind string
function SpawnerSystem:announce(entity, label, kind)
    if self.hooks.onSpawned then self.hooks.onSpawned(entity, label, kind) end
end

---@param prefix string
---@return string name `prefix n`, numbered per prefix
function SpawnerSystem:nextName(prefix)
    self.counts[prefix] = (self.counts[prefix] or 0) + 1
    return string.format("%s %d", prefix, self.counts[prefix])
end

---A station of an economic kind, with its production, market, credits and stock.
---@param spec { kind: string, position: Position, name: string|nil, seed: integer|nil, radiusM: number|nil }
---@return Entity station
function SpawnerSystem:spawnStation(spec)
    assert(self.world, "SpawnerSystem:spawnStation: no scene attached")
    local definition = assert(StationEconomies[spec.kind], "SpawnerSystem: no station economy " .. tostring(spec.kind))
    local radius = ScaleConfig:toGameUnits(spec.radiusM or self.stationRadiusM, "starSystem")
    local handle = self.constructs:createStation({
        seed = spec.seed or self.rng:get31(),
        config = { position = spec.position, scale = radius, isKinematic = true },
    })
    -- The generator scales a mesh of its own size: the body is rescaled to the radius asked for.
    local body = handle.rigidBody
    body:setScale(body:getScale() * radius / body:getBoundingRadius())
    local name = spec.name or self:nextName(definition.name)
    EconomySetup.equipStation(handle.root, definition, name)
    self.stations[#self.stations + 1] = { entity = handle.root, handle = handle, name = name, kind = spec.kind }
    self:announce(handle.root, name, "station")
    return handle.root
end

---A field of rocks that yield an item (iron ore when none is named). Sizes are in game units.
---@param spec { centre: Position, halfExtent: number, count: integer, minRadius: number, maxRadius: number, gap: number, quantity: integer, regrowPerSecond: number|nil, item: ItemDefinition|nil, seed: integer|nil }
---@return Entity[] rocks
function SpawnerSystem:spawnOreField(spec)
    assert(self.world, "SpawnerSystem:spawnOreField: no scene attached")
    local rocks = OreField.build(self.world, RNG.Create(spec.seed or self.rng:get31()), {
        centre = spec.centre, halfExtent = spec.halfExtent, count = spec.count,
        minRadius = spec.minRadius, maxRadius = spec.maxRadius, gap = spec.gap,
        item = spec.item or Items.RawMaterials.IronOre, quantity = spec.quantity, regrowPerSecond = spec.regrowPerSecond,
    })
    for _, rock in ipairs(rocks) do
        self.rocks[#self.rocks + 1] = rock
        self:announce(rock, nil, "rock")
    end
    return rocks
end

---A ship that becomes an agent of `role` after the next physics step.
---@param spec { role: EconomyAgentRole, position: Position, name: string|nil, owner: Entity|nil, seed: integer|nil }
---@return Entity ship
function SpawnerSystem:spawnAgent(spec)
    assert(self.world, "SpawnerSystem:spawnAgent: no scene attached")
    local role = assert(self.roles[spec.role], "SpawnerSystem: no agent role " .. tostring(spec.role))
    local handle = self.constructs:createShip({
        seed = spec.seed or self.rng:get31(),
        shipType = Enums.ShipType.Fighter,
        -- The Solo hull at its own scale: the one the Solo flight profile's thrust and torque are
        -- tuned for (another hull turns too lightly or too heavily for them).
        config = {
            hull = Enums.ShipHulls.Solo,
            position = spec.position,
            scale = Config.game.shipHulls.scale[Enums.ShipHulls.Solo],
            isKinematic = false,
            flightProfile = Enums.FlightProfile.Solo,
        },
    })
    local ship = handle.root
    ship:add(ConstructComponents.TravelDrive())
    ship:add(ConstructComponents.ControlInput())
    ship:add(PhysicsComponents.GravityWell(ConstructComponents.FlightProfile.ProfileOf(ship).linearDrag))
    local name = spec.name or self:nextName(role.label)
    self.agents[#self.agents + 1] = { entity = ship, handle = handle, name = name, role = spec.role }
    self.pending[#self.pending + 1] = { entity = ship, role = spec.role, name = name, owner = spec.owner }
    self:announce(ship, name, "agent")
    return ship
end

---Agents of each role in a ring around a station, beyond its hull.
---@param station Entity
---@param counts table<EconomyAgentRole, integer>
---@return Entity[] ships
function SpawnerSystem:spawnAgentsAround(station, counts)
    local body = station:get(PhysicsComponents.RigidBody):getRigidBody()
    local home, radius = body:getPos(), body:getBoundingRadius()
    local total = 0
    for _, count in pairs(counts) do total = total + count end
    local ships, index = {}, 0
    for _, role in ipairs({ Enums.EconomyAgentRole.Trader, Enums.EconomyAgentRole.Miner }) do
        for _ = 1, counts[role] or 0 do
            index = index + 1
            local angle = index * 2 * math.pi / total
            ships[#ships + 1] = self:spawnAgent({
                role = role,
                position = Position(home.x + math.cos(angle) * radius * 6, home.y + radius * 2, home.z + math.sin(angle) * radius * 6),
            })
        end
    end
    return ships
end

---@return SpawnedStation[]
function SpawnerSystem:getStations()
    return self.stations
end

---@return { entity: Entity, name: string, role: EconomyAgentRole }[]
function SpawnerSystem:getAgents()
    return self.agents
end

---@param data EventData
function SpawnerSystem:onPostSim(data)
    self:startPending()
end

---Makes the ships spawned before this physics step agents.
function SpawnerSystem:startPending()
    if #self.pending == 0 then return end
    for _, ship in ipairs(self.pending) do
        if Registry:hasEntity(ship.entity) then
            local role = self.roles[ship.role]
            EconomySetup.equipAgent(ship.entity, {
                role = ship.role, capacity = role.capacity, name = ship.name,
                owner = ship.owner, credits = not ship.owner and role.credits or nil,
            })
        end
    end
    self.pending = {}
end

return SpawnerSystem()
