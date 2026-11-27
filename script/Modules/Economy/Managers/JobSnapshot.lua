local Registry = require("Core.ECS.Registry")
local Items = require("Shared.Registries.Items")
local EntityPosition = require("Shared.Helpers.EntityPosition")
local EstimateRequest = require("Shared.Helpers.EstimateRequest")
local OrderGoals = require("Shared.Helpers.OrderGoals")
local PlanFlight = require("Shared.Helpers.PlanFlight")
local NavConfig = require("Config.Navigation.NavConfig")
local JobPlanner = require("Shared.Economy.JobPlanner")
local PhysicsComponents = require("Modules.Physics.Components")
local ConstructComponents = require("Modules.Constructs.Components")
local FlightProfile = require("Modules.Constructs.Components.FlightProfileComponent")
local Economy = require("Modules.Economy.Components")
local InventoryManager = require("Modules.Economy.Managers.InventoryManager")
local AccountManager = require("Modules.Economy.Managers.AccountManager")
local FactionManager = require("Modules.Factions.Managers.FactionManager")

-- Design: doc/script/economy.md

---Builds what NPC captains think from (`JobPlanner`): the job board's offers, the stations and rocks
---they lead to, the claims of the agents, the factions' hostilities, the navigation scene, and each
---thinking captain's ship. Flat arrays of numbers (`JobPlanner.Layout`), so the snapshot crosses to a
---worker as a payload.
---@class JobSnapshot
local JobSnapshot = {}

local L, Flag = JobPlanner.Layout, JobPlanner.Flag

---Appends a record's fields, in the layout's order, to `flat`.
---@param flat number[]
---@param layout table
---@param fields table<string, number>
local function push(flat, layout, fields)
    local base = #flat
    for name, offset in pairs(layout) do
        if name ~= "Stride" then flat[base + offset + 1] = fields[name] or 0 end
    end
end

---@param entity Entity
---@return integer 0 for none
local function factionId(entity)
    local faction = FactionManager:factionOf(entity)
    return faction and faction.id or 0
end

---Where a ship stops at the entity and how big it is.
---@param entity Entity
---@return table fields X, Y, Z, Safe, Arrival, Radius
local function placeOf(entity)
    local goal = OrderGoals.moveTo(entity)
    local rbCmp = entity:get(PhysicsComponents.RigidBody)
    local body = rbCmp and rbCmp:getRigidBody()
    return {
        Id = entity.id, X = goal.targetPos.x, Y = goal.targetPos.y, Z = goal.targetPos.z,
        Safe = goal.safeRange or 0, Arrival = goal.arrivalRadius or 0, Radius = body and body:getBoundingRadius() or 0,
    }
end

---The snapshot for the captains of `ships`.
---@param ships { ship: Entity, agent: EconomyAgentComponent }[]
---@param board JobBoardSystem
---@param claims table<string, number> The agents' claims now
---@param params table The planner's numbers (`now`, `dockOverheadS`, `minProfit`, `miningRate`...)
---@return table snapshot
function JobSnapshot.build(ships, board, claims, params)
    local s = EstimateRequest.scene(nil)
    s.params = params
    params.detourFactor = NavConfig.estimateDetourFactor
    s.claims = claims

    -- The stations of the markets, and the rocks of the mining offers.
    local stations, seenStation = {}, {}
    local function addStation(entity)
        if seenStation[entity.id] or not Registry:hasEntity(entity) then return end
        seenStation[entity.id] = true
        local place = placeOf(entity)
        place.Faction = factionId(entity)
        place.Ports = require("Modules.Docking.Systems.DockingSystem"):countPorts(entity)
        push(stations, L.Station, place)
    end
    for entity in Registry:view(Economy.Marketplace) do
        local rbCmp = entity:get(PhysicsComponents.RigidBody)
        if rbCmp and rbCmp:getRigidBody() then addStation(entity) end
    end
    s.stations = stations

    local items, masses = {}, {}
    local function addItem(itemId)
        if items[itemId] then return end
        items[itemId] = true
        local definition = Items:getDefinition(itemId)
        push(masses, L.ItemMass, { Item = itemId, Mass = definition and definition.mass or 1 })
    end

    local bids = {}
    for itemId, list in pairs(board.bids or {}) do
        addItem(itemId)
        for _, bid in ipairs(list) do
            if Registry:hasEntity(bid.station) then
                push(bids, L.Bid, { Station = bid.station.id, Item = itemId, Price = bid.price, Demand = bid.demand })
            end
        end
    end
    s.bids = bids

    local trades = {}
    for _, offer in ipairs(board:getTrades()) do
        if Registry:hasEntity(offer.source) and Registry:hasEntity(offer.destination) then
            addItem(offer.itemId)
            push(trades, L.Trade, { Source = offer.source.id, Destination = offer.destination.id, Item = offer.itemId,
                Buy = offer.buyPrice, Sell = offer.sellPrice, Supply = offer.supply, Demand = offer.demand })
        end
    end
    s.trades = trades

    local mining, rocks, seenRock = {}, {}, {}
    for _, offer in ipairs(board:getMining()) do
        if Registry:hasEntity(offer.rock) and Registry:hasEntity(offer.station) then
            addItem(offer.itemId)
            if not seenRock[offer.rock.id] then
                seenRock[offer.rock.id] = true
                push(rocks, L.Rock, placeOf(offer.rock))
            end
            push(mining, L.Mining, { Rock = offer.rock.id, Station = offer.station.id, Item = offer.itemId,
                Sell = offer.sellPrice, Left = offer.left, Demand = offer.demand, UseRate = offer.useRate or 0 })
        end
    end
    s.mining, s.rocks = mining, rocks

    local patrols = {}
    for _, offer in ipairs(board:getPatrols()) do
        if Registry:hasEntity(offer.station) then
            push(patrols, L.Patrol, { Station = offer.station.id, Payout = offer.payout, Duration = offer.duration, Slots = offer.slots })
        end
    end
    s.patrols = patrols

    -- The factions: hostile pairs, the marauders and their prey.
    local hostile, marauders, victims = {}, {}, {}
    local factions = FactionManager:all()
    for i, a in ipairs(factions) do
        for j = i + 1, #factions do
            if FactionManager:isHostile(a, factions[j]) then push(hostile, L.Hostile, { A = a.id, B = factions[j].id }) end
        end
        if FactionManager:isMarauders(a) then
            marauders[#marauders + 1] = a.id
            push(victims, L.Victims, { Faction = a.id, Count = board:countRaidTargets(a) })
        end
    end
    s.hostile, s.marauders, s.victims = hostile, marauders, victims

    -- The thinking captains' ships.
    local agents, cargo, badRocks = {}, {}, {}
    local driveOn = Config.game.autoPilot.enableTravelDrive ~= false
    for index, entry in ipairs(ships) do
        local ship, agent = entry.ship, entry.agent
        local pos = EntityPosition.Get(ship)
        local profile = FlightProfile.ProfileOf(ship)
        local inventory = ship:get(Economy.Inventory)
        local free = InventoryManager:getFreeMass(inventory)
        local capacity = inventory:getCapacity()
        local own = FactionManager:factionOf(ship)
        local flags = 0
        if ship:get(Economy.MiningLaser) then flags = flags + Flag.Laser end
        if ship:get(ConstructComponents.WeaponControl) then flags = flags + Flag.Armed end
        if FactionManager:isMarauders(own) then flags = flags + Flag.Marauder end
        if capacity == nil or capacity > 0 then flags = flags + Flag.Hold end
        if require("Modules.Docking.Systems.DockingSystem"):phaseOf(ship) == nil then flags = flags + Flag.InSpace end
        local driven = driveOn and ship:get(ConstructComponents.TravelDrive) ~= nil
        push(agents, L.Agent, {
            Id = ship.id, X = pos.x, Y = pos.y, Z = pos.z, MaxSpeed = profile.maxSpeed, Drag = profile.linearDrag,
            OpenCap = driven and Config.game.travelDrive.maxSpeedByZone.openSpace or 0,
            RampDown = driven and Config.game.travelDrive.rampDownSpeed or 0,
            DriveMin = PlanFlight.driveMinLength(profile), FreeMass = free == math.huge and -1 or free,
            Balance = AccountManager:balanceOf(ship), Faction = own and own.id or 0, Flags = flags,
        })
        for itemId in pairs(inventory:getInventory()) do
            local count = InventoryManager:count(inventory, itemId)
            if count > 0 then
                addItem(itemId)
                push(cargo, L.Cargo, { Agent = index, Item = itemId, Count = count })
            end
        end
        for rockId, untilTime in pairs(agent.badRocks or {}) do
            push(badRocks, L.BadRock, { Agent = index, Rock = rockId, Until = untilTime })
        end
    end
    s.agents, s.cargo, s.badRocks, s.itemMasses = agents, cargo, badRocks, masses
    return s
end

return JobSnapshot
