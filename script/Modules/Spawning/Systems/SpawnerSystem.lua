local Registry            = require("Core.ECS.Registry")
local QuickProfiler       = require("Shared.Tools.QuickProfiler")
local ScaleConfig         = require("Config.Gen.UniverseScaleConfig")
local PhysicsComponents   = require("Modules.Physics.Components")
local ConstructComponents = require("Modules.Constructs.Components")
local NameComponent       = require("Modules.Core.Components.NameComponent")
local Items               = require("Shared.Registries.Items")
local StationEconomies    = require("Shared.Definitions.StationEconomyDefs")
local ShipKinds           = require("Shared.Definitions.ShipKindDefs")
local OreField            = require("Shared.Helpers.OreField")
local Economy             = require("Modules.Economy.Components")
local EconomySetup        = require("Modules.Economy.Managers.EconomySetup")
local AccountManager      = require("Modules.Economy.Managers.AccountManager")
local Factions            = require("Modules.Factions.Components")
local FactionManager      = require("Modules.Factions.Managers.FactionManager")
local SpawningComponents  = require("Modules.Spawning.Components")
-- Required for their event subscriptions: they run the economy, the dockings and the factions.
local EconomySystems      = require("Modules.Economy.Systems")
local _                   = require("Modules.Docking.Systems")
local _                   = require("Modules.Factions.Systems")
local PeopleManager       = require("Modules.People.Managers.PeopleManager")
local _                   = require("Modules.People.Systems")

-- Design: doc/script/spawning.md

---@class SpawnHooks
---@field onSpawned fun(entity: Entity, label: string|nil, kind: string)|nil Told of every station, ship and ore field spawned
---@field weaponCombat WeaponCombatSystem|nil The scene's weapons: armed ships are built with turrets when there is one

---@class SpawnedStation
---@field entity Entity
---@field handle table The ConstructManager handle
---@field name string
---@field kind string A key of `StationEconomyDefs`

---@class SpawnedShip
---@field entity Entity
---@field handle table The ConstructManager handle
---@field name string
---@field kind string A key of `ShipKindDefs`
---@field battery WeaponBattery|nil Its weapons, when it is armed

---@class SpawnedFaction
---@field entity Entity
---@field leader Entity The player that owns its stations and ships, and pays for them
---@field stations Entity[]

---Spawns what lives in a star system's economy: stations of an economic kind, ore fields, factions,
---and AI ships of a kind (`ShipKindDefs`: a hauler, a miner, an armed fighter), whose AI chooses its
---jobs by the equipment it carries. It is the one place that does: scenarios, testbeds and the debug
---console call it directly (`spawnStation`, `spawnOreField`, `spawnAgent`), and `populate` builds a
---star system's planned economy (`EconomyLayoutComponent`) through it. It keeps the factions' fleets
---going: every `fleetIntervalS` a faction buys the kind of ship the most valuable unfilled work on the
---job board calls for, at `shipCost`, from its leader's account.
---Stations and ships are made by the scene's `ConstructManager` (armed ones through
---`ArmedShipBuilder`, run by the scene's weapons) and rocks by `OreField`; the spawner keeps them, so
---`clear` takes them all down with the scene. A new ship becomes an agent after the next physics step
---(at PostSim), when its body is in the query index navigation reads.
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
    self.fleetIntervalS = 30
    self.shipCost = 4000
    -- The unfilled work, in credits, that makes a faction buy a ship for it, and the most ships of a
    -- kind a faction runs.
    self.fleetDemand = 1500
    self.maxFleetPerKind = 8
    self:reset()
end

---Forgets the scene (what `attach` and the spawns set up), without destroying anything.
function SpawnerSystem:reset()
    self.world, self.constructs, self.hooks = nil, nil, {}
    ---@type SpawnedStation[]
    self.stations = {}
    ---@type SpawnedShip[]
    self.agents = {}
    ---@type Entity[]
    self.rocks = {}
    ---@type SpawnedFaction[]
    self.factions = {}
    self.pending = {}
    self.counts = {}
    self.nextFleetAt = nil
    ---@type Entity[] The people who man the ships it spawned, while they live
    self.people = {}
    self.known = {}
    -- The hull design of each faction's ships of a kind: a seed, keyed "<faction leader id>:<kind>".
    self.designs = {}
end

---The seed of the hull a faction builds its ships of a kind on: drawn at its first ship of the kind
---and kept, so its haulers look alike, its fighters alike (and the hull's mesh and collision shape
---are made once, `ShipGenerator`'s cache). A ship without an owner gets a hull of its own.
---@param owner Entity|nil
---@param kind ShipKindDefinition
---@param rng RNG|nil Draws a new design (the spawner's own by default)
---@return integer seed
function SpawnerSystem:designSeed(owner, kind, rng)
    rng = rng or self.rng
    if not owner then return rng:get31() end
    local key = owner.id .. ":" .. kind.name
    if not self.designs[key] then self.designs[key] = rng:get31() end
    return self.designs[key]
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
    for _, ship in ipairs(self.agents) do
        if ship.battery and self.hooks.weaponCombat then self.hooks.weaponCombat:removeBattery(ship.battery) end
        self.constructs:destroy(ship.handle)
    end
    for _, station in ipairs(self.stations) do self.constructs:destroy(station.handle) end
    for _, rock in ipairs(self.rocks) do
        if Registry:hasEntity(rock) then
            local rbCmp = rock:get(PhysicsComponents.RigidBody)
            if rbCmp and rbCmp:getRigidBody() then self.world:removeRigidBody(rbCmp:getRigidBody()) end
            Registry:destroyEntity(rock)
        end
    end
    for _, faction in ipairs(self.factions) do
        for _, entity in ipairs({ faction.leader, faction.entity }) do
            if Registry:hasEntity(entity) then Registry:destroyEntity(entity) end
        end
    end
    for _, person in ipairs(self.people) do
        if Registry:hasEntity(person) then Registry:destroyEntity(person) end
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

-- Spawns ---------------------------------------------------------------------------------------------

---A faction with a leader, the player that owns what the faction spawns and pays for it.
---@param spec { type: integer, name: string|nil, seed: integer|nil, credits: number|nil }
---@return Entity faction, Entity leader
function SpawnerSystem:spawnFaction(spec)
    local faction = FactionManager:create({ type = spec.type, name = spec.name, seed = spec.seed, treasury = 0 })
    local name = faction:get(Factions.Faction):getName()
    local leader = FactionManager:createMember(faction, name .. " Command", spec.credits or 60000)
    self.factions[#self.factions + 1] = { entity = faction, leader = leader, stations = {} }
    return faction, leader
end

---@param leader Entity
---@return SpawnedFaction|nil
function SpawnerSystem:factionEntryOf(leader)
    for _, entry in ipairs(self.factions) do
        if entry.leader.id == leader.id then return entry end
    end
    return nil
end

---A station of an economic kind, with its production, market, credits and stock.
---@param spec { kind: string, position: Position, name: string|nil, seed: integer|nil, radiusM: number|nil, owner: Entity|nil }
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
    if spec.owner then
        handle.root:add(Economy.Ownership(spec.owner.id))
        FactionManager:enlist(handle.root)
        local entry = self:factionEntryOf(spec.owner)
        if entry then entry.stations[#entry.stations + 1] = handle.root end
    end
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

---The hull of a ship of a kind: a Solo hull, with turrets for an armed kind when the scene has
---weapons and the build works (else it flies unarmed and its AI takes no fighting jobs).
---@param kind ShipKindDefinition
---@param spec table
---@param name string
---@return table handle, WeaponBattery|nil battery
function SpawnerSystem:buildHull(kind, spec, name)
    local config = {
        hull = Enums.ShipHulls.Solo,
        position = spec.position,
        scale = Config.game.shipHulls.scale[Enums.ShipHulls.Solo],
        isKinematic = false,
        flightProfile = Enums.FlightProfile.Solo,
    }
    local seed = spec.seed or self:designSeed(spec.owner, kind)
    local weaponCombat = self.hooks.weaponCombat
    if kind.armed and weaponCombat then
        local ok, construction = pcall(function()
            local ArmedShipBuilder = require("Modules.Constructs.Managers.ArmedShipBuilder")
            return ArmedShipBuilder:build(self:fighterRecipe(seed, config), { constructManager = self.constructs, shipConfig = config })
        end)
        if ok and construction then
            local WeaponBattery = require("Modules.Constructs.Systems.WeaponBattery")
            local battery = weaponCombat:addBattery(WeaponBattery(construction, {
                name = name, aiActive = true, team = spec.owner and FactionManager:teamOf(spec.owner) or FactionManager.INDEPENDENT_TEAM,
            }))
            return construction.shipHandle, battery
        end
        Log.Warn("[Spawner] %s flies unarmed: %s", name, tostring(construction))
    end
    return self.constructs:createShip({ seed = seed, shipType = Enums.ShipType.Fighter, config = config }), nil
end

---The armament of a fighter: a mirrored pair of small burst-laser turrets on the top of its fore hull.
---@param seed integer
---@param config table The hull's config
---@return table recipe For `ArmedShipBuilder:build`
function SpawnerSystem:fighterRecipe(seed, config)
    local Weapon = Enums.Weapon
    local mount = function(side)
        return {
            mountId = "fore_" .. side, pairId = Weapon.MountPairId.ForeOuter, zone = Weapon.MountZone.Fore,
            side = side, surfaceBand = Weapon.MountSurfaceBand.Top, mountSizeClass = Weapon.MountSizeClass.M,
            mountRole = Weapon.MountRole.Line,
        }
    end
    return {
        seed = seed,
        ship = { shipType = Enums.ShipType.Fighter, config = config },
        mountDiscovery = {
            specifications = { mount(Weapon.MountSide.Port), mount(Weapon.MountSide.Starboard) },
            options = { minNormalDot = 0.2 },
        },
        proceduralWeapons = {
            {
                key = Weapon.ProceduralKey.BurstLaser,
                args = {
                    family = Weapon.Family.Laser, variant = Weapon.Variant.ShortBurst, baseWeaponId = Weapon.Type.Laser,
                    burstCount = 3, duration = 0.3,
                },
            },
        },
        loadout = {
            { mountId = "fore_" .. Weapon.MountSide.Port, generatedKey = Weapon.ProceduralKey.BurstLaser },
            { mountId = "fore_" .. Weapon.MountSide.Starboard, generatedKey = Weapon.ProceduralKey.BurstLaser },
        },
        defaultWeapon = { generatedKey = Weapon.ProceduralKey.BurstLaser },
        requireLoadoutForAllMounts = false,
    }
end

---An AI ship of a kind (`ShipKindDefs`), manned by a crew of its kind's roles, which becomes an agent
---after the next physics step: its commander then chooses its jobs by what it carries. With an
---`owner` (a faction's leader) the crew are the owner's employees, people it has on its books but on
---no ship first (a lost ship's crew), new ones else. Without one it is an independent: its captain
---owns it, starts with `credits` (its kind's by default) and employs the rest of the crew.
---@param spec { kind: string, position: Position, name: string|nil, owner: Entity|nil, seed: integer|nil, credits: number|nil }
---@return Entity ship
function SpawnerSystem:spawnAgent(spec)
    assert(self.world, "SpawnerSystem:spawnAgent: no scene attached")
    local kind = assert(ShipKinds[spec.kind], "SpawnerSystem: no ship kind " .. tostring(spec.kind))
    local name = spec.name or self:nextName(kind.name)
    local handle, battery = self:buildHull(kind, spec, name)
    local ship = handle.root
    if not ship:get(ConstructComponents.TravelDrive) then ship:add(ConstructComponents.TravelDrive()) end
    if not ship:get(ConstructComponents.ControlInput) then ship:add(ConstructComponents.ControlInput()) end
    ship:add(PhysicsComponents.GravityWell(ConstructComponents.FlightProfile.ProfileOf(ship).linearDrag))
    ship:get(NameComponent):setName(name)
    local owner = spec.owner
    if owner then
        PeopleManager:man(ship, {
            roles = kind.crew, rng = self.rng, employer = owner, faction = FactionManager:factionOf(owner),
            hire = function() return PeopleManager:unassigned(owner)[1] end,
        })
    else
        -- An independent: the captain owns the ship and pays the others.
        owner = PeopleManager:manIndependent(ship, { roles = kind.crew, rng = self.rng, credits = spec.credits or kind.credits })
    end
    for _, member in ipairs(PeopleManager:membersOf(ship)) do
        if not self.known[member.person.id] then
            self.known[member.person.id] = true
            self.people[#self.people + 1] = member.person
        end
    end
    ship:add(Economy.Ownership(owner.id))
    FactionManager:enlist(ship)
    self.agents[#self.agents + 1] = { entity = ship, handle = handle, name = name, kind = spec.kind, battery = battery }
    self.pending[#self.pending + 1] = { entity = ship, kind = spec.kind, name = name, owner = owner }
    self:announce(ship, name, "ship")
    return ship
end

---Ships of each kind in a ring around a station, beyond its hull.
---@param station Entity
---@param counts table<string, integer> Ship kind -> how many
---@param owner Entity|nil
---@return Entity[] ships
function SpawnerSystem:spawnAgentsAround(station, counts, owner)
    local ships = {}
    local index = 0
    for _, kind in ipairs({ "Hauler", "Miner", "Fighter" }) do
        for _ = 1, counts[kind] or 0 do
            index = index + 1
            ships[#ships + 1] = self:spawnAgent({ kind = kind, position = self:besideStation(station, index), owner = owner })
        end
    end
    return ships
end

---@return SpawnedStation[]
function SpawnerSystem:getStations()
    return self.stations
end

---@return SpawnedShip[]
function SpawnerSystem:getAgents()
    return self.agents
end

---@return SpawnedFaction[]
function SpawnerSystem:getFactions()
    return self.factions
end

---A place in a ring round a station, beyond its hull.
---@param station Entity
---@param index integer
---@return Position
function SpawnerSystem:besideStation(station, index)
    local body = station:get(PhysicsComponents.RigidBody):getRigidBody()
    local home, radius = body:getPos(), body:getBoundingRadius()
    local angle = index * 2.4
    return Position(home.x + math.cos(angle) * radius * 6, home.y + radius * 2, home.z + math.sin(angle) * radius * 6)
end

---@param data EventData
function SpawnerSystem:onPostSim(data)
    self:startPending()
    self:upkeepFleets()
end

---Makes the ships spawned before this physics step agents, with the equipment of their kind.
function SpawnerSystem:startPending()
    if #self.pending == 0 then return end
    for _, ship in ipairs(self.pending) do
        if Registry:hasEntity(ship.entity) then
            local kind = ShipKinds[ship.kind]
            EconomySetup.equipAgent(ship.entity, {
                kind = ship.kind, capacity = kind.capacity, miningLaser = kind.miningLaser, name = ship.name,
                owner = ship.owner, credits = not ship.owner and kind.credits or nil,
            })
        end
    end
    self.pending = {}
end

-- Population ---------------------------------------------------------------------------------------

---The planets by their distance from the system's centre, nearest first: the order the layout's
---planet indices count in (the generator gives planets growing orbits).
---@param planets Entity[]
---@return Entity[]
local function byOrbit(planets)
    local sorted = {}
    for _, planet in ipairs(planets) do sorted[#sorted + 1] = planet end
    local function distance(planet)
        local p = planet:get(PhysicsComponents.Transform):getPos()
        return p.x * p.x + p.y * p.y + p.z * p.z
    end
    table.sort(sorted, function(a, b) return distance(a) < distance(b) end)
    return sorted
end

---Where a layout place is in the scene.
---@param planets Entity[] By orbit
---@param place LayoutPlace
---@return Position
local function positionOf(planets, place)
    local planet = planets[math.min(place.planet, #planets)]
    local transform = planet:get(PhysicsComponents.Transform)
    local centre = transform:getPos()
    local rbCmp = planet:get(PhysicsComponents.RigidBody)
    local radius = rbCmp and rbCmp:getRigidBody() and rbCmp:getRigidBody():getBoundingRadius() or transform:getScale()
    local angle = math.rad(place.angleDeg)
    return Position(centre.x + math.cos(angle) * place.radii * radius, centre.y + place.heightRadii * radius,
        centre.z + math.sin(angle) * place.radii * radius)
end

---@param count integer
---@param density number
---@return integer
local function scaled(count, density)
    if density <= 0 then return 0 end
    return math.max(count > 0 and 1 or 0, math.floor(count * density + 0.5))
end

---Builds the star system's planned economy (`EconomyLayoutComponent`, from its generation) into the
---scene: the factions, each with a leader that owns its stations and ships, the stations, the ore
---fields, and the ships, their numbers scaled by `density` (1 as planned, 0 none).
---@param planets Entity[] The system's planets
---@param density number|nil
---@return boolean populated False without a layout, planets or density
function SpawnerSystem:populate(planets, density)
    assert(self.world, "SpawnerSystem:populate: no scene attached")
    density = density or 1
    local layoutCmp
    for _, cmp in Registry:view(SpawningComponents.EconomyLayout) do layoutCmp = cmp break end
    if not layoutCmp or #planets == 0 or density <= 0 then return false end
    local layout = layoutCmp:getLayout()
    planets = byOrbit(planets)

    for _, plan in ipairs(layout.factions) do
        local rng = RNG.Create(plan.seed)
        local _, leader = self:spawnFaction({ type = plan.type, seed = plan.seed })
        local stations = {}
        for _, place in ipairs(plan.stations) do
            stations[#stations + 1] = self:spawnStation({ kind = place.kind, position = positionOf(planets, place), owner = leader, seed = rng:get31() })
        end
        local index = 0
        for _, ships in ipairs({ { "Hauler", plan.haulers }, { "Miner", plan.miners }, { "Fighter", plan.fighters } }) do
            for _ = 1, scaled(ships[2] or 0, density) do
                index = index + 1
                local station = stations[(index - 1) % #stations + 1]
                self:spawnAgent({ kind = ships[1], position = self:besideStation(station, index), owner = leader,
                    seed = self:designSeed(leader, ShipKinds[ships[1]], rng) })
            end
        end
    end
    for _, field in ipairs(layout.oreFields) do
        self:spawnOreField({
            centre = positionOf(planets, field), halfExtent = 1.0, count = field.count,
            minRadius = 0.06, maxRadius = 0.1, gap = 0.25, quantity = 400, regrowPerSecond = 0.5,
        })
    end
    if layout.marauders then
        local plan = layout.marauders
        local _, boss = self:spawnFaction({ type = Enums.FactionType.Marauders, credits = 20000 })
        local den = positionOf(planets, plan.place)
        local spread = ScaleConfig:toGameUnits(20e3, "starSystem")
        for i = 1, scaled(plan.pirates, density) do
            local angle = i * 2 * math.pi / plan.pirates
            self:spawnAgent({ kind = "Fighter", owner = boss,
                position = Position(den.x + math.cos(angle) * spread, den.y, den.z + math.sin(angle) * spread) })
        end
    end
    -- Independents start by the stations in turn.
    local index = 0
    for _, independent in ipairs(layout.independents or {}) do
        if #self.stations > 0 then
            index = index + 1
            local station = self.stations[(index - 1) % #self.stations + 1].entity
            self:spawnAgent({ kind = independent.kind, position = self:besideStation(station, 20 + index), seed = independent.seed })
        end
    end
    return true
end

-- Fleets ---------------------------------------------------------------------------------------------

-- The kind of ship each kind of work calls for.
local KIND_FOR_WORK = { trade = "Hauler", mining = "Miner", patrol = "Fighter" }

---Buys ships for the factions the spawner made, where there is work for them: the most valuable
---unfilled work of the job board (trade, mining, security) at a value of at least `fleetDemand`, a
---ship of the kind it calls for, while the faction has fewer than `maxFleetPerKind` of that kind and
---its leader can pay `shipCost`. Marauders buy a fighter only out of loot to spare (twice its cost),
---while there is twice as much prey as their fighters.
---Lost ships are replaced the same way.
function SpawnerSystem:upkeepFleets()
    if #self.factions == 0 then return end
    local now = EconomySystems.Marketplace:now()
    if not self.nextFleetAt then self.nextFleetAt = now + self.fleetIntervalS return end
    if now < self.nextFleetAt then return end
    self.nextFleetAt = now + self.fleetIntervalS

    local agentSystem, board = EconomySystems.EconomyAgent, EconomySystems.JobBoard
    local claimed = function(key) return agentSystem:claimed(key) end
    local demand = {}
    for work in pairs(KIND_FOR_WORK) do demand[work] = board:unfilledValue(work, claimed) end

    for _, entry in ipairs(self.factions) do
        local fleet = {}
        for _, ship in ipairs(self.agents) do
            if Registry:hasEntity(ship.entity) and FactionManager:factionOf(ship.entity) == entry.entity then
                fleet[ship.kind] = (fleet[ship.kind] or 0) + 1
            end
        end
        local marauders = entry.entity:get(Factions.Faction):getType() == Enums.FactionType.Marauders
        local work = nil
        if marauders then
            -- Marauders buy a raider only out of loot to spare, while there is prey for more of them.
            local prey = board:countRaidTargets(entry.entity)
            local balance = AccountManager:balanceOf(entry.leader)
            if prey >= 2 * ((fleet.Fighter or 0) + 1) and balance >= 2 * self.shipCost then work = "patrol" end
        else
            for kindOfWork, value in pairs(demand) do
                if value >= self.fleetDemand and (not work or value > demand[work]) then work = kindOfWork end
            end
        end
        local kind = work and KIND_FOR_WORK[work]
        local home = nil
        for _, station in ipairs(entry.stations) do
            if Registry:hasEntity(station) then home = station break end
        end
        local origin = home and self:besideStation(home, #self.agents + 1)
        if not origin and marauders then
            for _, ship in ipairs(self.agents) do
                if Registry:hasEntity(ship.entity) and FactionManager:factionOf(ship.entity) == entry.entity then
                    local p = ship.entity:get(PhysicsComponents.RigidBody):getRigidBody():getPos()
                    origin = Position(p.x, p.y + 0.3, p.z)
                    break
                end
            end
        end
        if kind and origin and (fleet[kind] or 0) < self.maxFleetPerKind
            and AccountManager:withdraw(AccountManager:accountOf(entry.leader), self.shipCost) then
            local ship = self:spawnAgent({ kind = kind, position = origin, owner = entry.leader })
            Log.Debug("[Spawner] %s bought %s for %d", tostring(entry.entity), tostring(ship), self.shipCost)
            if work and demand[work] then demand[work] = demand[work] - self.fleetDemand end
        end
    end
end

return SpawnerSystem()
