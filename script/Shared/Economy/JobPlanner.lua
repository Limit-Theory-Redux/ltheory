--- The economic thinking of NPC captains: for each captain that needs work, the best job its ship can
--- do from a snapshot of the job board, by profit per second. Pure numbers in and out (no entities,
--- no globals), so it runs on a worker (`JobPlannerWorker.lua`) and inline alike. The main thread
--- builds the snapshot (`Modules/Economy/Managers/JobSnapshot.lua`) and applies the answers.
---
--- The snapshot is flat arrays of numbers, `Layout` giving each record's fields (a payload carries
--- no arrays of tables), plus `claims` (what the other agents hold, by claim key) and `params`.
--- Flight times are navigation estimates of the scene (`NavEstimate`), as `Nav.estimate` gives them.
--- Design: doc/script/economy.md
local NavEstimate = require("Shared.Helpers.NavEstimate")

local floor, min, max, sqrt = math.floor, math.min, math.max, math.sqrt

local JobPlanner = {}

-- The fields of each record, 0-based, and its stride.
JobPlanner.Layout = {
    -- A captain's ship: id, position, flight numbers (open cap 0: no drive), the shortest drive leg,
    -- its hold's free mass (-1: no limit), its owner's balance, its faction (0: none) and `Flag`s.
    Agent = { Id = 0, X = 1, Y = 2, Z = 3, MaxSpeed = 4, Drag = 5, OpenCap = 6, RampDown = 7, DriveMin = 8,
        FreeMass = 9, Balance = 10, Faction = 11, Flags = 12, Stride = 13 },
    -- A station: where a ship stops near it (stand-off and arrival), its radius, faction, free ports.
    Station = { Id = 0, X = 1, Y = 2, Z = 3, Safe = 4, Arrival = 5, Radius = 6, Faction = 7, Ports = 8, Stride = 9 },
    Rock = { Id = 0, X = 1, Y = 2, Z = 3, Safe = 4, Arrival = 5, Radius = 6, Stride = 7 },
    Bid = { Station = 0, Item = 1, Price = 2, Demand = 3, Stride = 4 },
    Trade = { Source = 0, Destination = 1, Item = 2, Buy = 3, Sell = 4, Supply = 5, Demand = 6, Stride = 7 },
    Mining = { Rock = 0, Station = 1, Item = 2, Sell = 3, Left = 4, Demand = 5, UseRate = 6, Stride = 7 },
    Patrol = { Station = 0, Payout = 1, Duration = 2, Slots = 3, Stride = 4 },
    -- What an agent carries, and the rocks it has given up on until a time: by the agent's index (1-based).
    Cargo = { Agent = 0, Item = 1, Count = 2, Stride = 3 },
    BadRock = { Agent = 0, Rock = 1, Until = 2, Stride = 3 },
    ItemMass = { Item = 0, Mass = 1, Stride = 2 },
    -- Pairs of hostile factions; marauder factions; raid prey of each marauder faction.
    Hostile = { A = 0, B = 1, Stride = 2 },
    Victims = { Faction = 0, Count = 1, Stride = 2 },
    -- An answer: the agent's id, the job's kind and what it needs.
    Result = { Agent = 0, Kind = 1, Item = 2, Quantity = 3, Source = 4, Destination = 5, Buy = 6, Sell = 7,
        Payout = 8, Duration = 9, Score = 10, Stride = 11 },
}

JobPlanner.Kind = { None = 0, Sale = 1, Trade = 2, Mining = 3, Patrol = 4, Raid = 5, Park = 6 }
JobPlanner.Flag = { Laser = 1, Armed = 2, Marauder = 4, Hold = 8, InSpace = 16 }

local L, Kind, Flag = JobPlanner.Layout, JobPlanner.Kind, JobPlanner.Flag

---@param kind string "ask", "bid" or "rock"
---@param entityId integer
---@param itemId integer
---@return string
function JobPlanner.claimKey(kind, entityId, itemId)
    return kind .. ":" .. entityId .. ":" .. itemId
end
local claimKey = JobPlanner.claimKey

---@param flags integer
---@param flag integer
---@return boolean
local function has(flags, flag)
    return floor(flags / flag) % 2 == 1
end

---Records of a flat array, as tables of their fields.
---@param flat number[]|nil
---@param layout table
---@return table[]
local function records(flat, layout)
    local list, stride = {}, layout.Stride
    flat = flat or {}
    for base = 1, #flat, stride do
        local record = {}
        for name, offset in pairs(layout) do
            if name ~= "Stride" then record[name] = flat[base + offset] end
        end
        list[#list + 1] = record
    end
    return list
end

---Where a ship from `start` stops at a target: its stand-off point on the line to it.
---@return number, number, number
local function standOff(sx, sy, sz, target)
    local dx, dy, dz = sx - target.X, sy - target.Y, sz - target.Z
    local dist = sqrt(dx * dx + dy * dy + dz * dz)
    if dist <= target.Safe then return sx, sy, sz end
    local back = target.Safe / dist
    return target.X + dx * back, target.Y + dy * back, target.Z + dz * back
end

---Flight times from a point to each target, in the targets' order: the scene's estimates, or the
---straight line at top speed for a target the estimate can't reach.
---@param scene table
---@param ship table A `NavEstimateShip`
---@param driveMin number
---@param sx number
---@param targets table[]
---@return number[]
local function timesTo(scene, ship, driveMin, sx, sy, sz, targets)
    local goals, Goal = {}, NavEstimate.Goal
    for i, target in ipairs(targets) do
        local base = (i - 1) * NavEstimate.GOAL_STRIDE
        local gx, gy, gz = standOff(sx, sy, sz, target)
        goals[base + Goal.X + 1], goals[base + Goal.Y + 1], goals[base + Goal.Z + 1] = gx, gy, gz
        goals[base + Goal.ArrivalRadius + 1] = target.Arrival
        goals[base + Goal.DriveMinLength + 1] = driveMin
    end
    local results = NavEstimate.many(scene, ship, { sx, sy, sz }, goals, #targets)
    local times = {}
    for i, target in ipairs(targets) do
        local result = results[i]
        if result.reachable and result.time then
            times[i] = result.time
        else
            local dx, dy, dz = target.X - sx, target.Y - sy, target.Z - sz
            times[i] = sqrt(dx * dx + dy * dy + dz * dz) / max(ship.maxSpeed, 1e-6)
        end
    end
    return times
end

---Plans a job for every agent of the snapshot, in order: each agent's job counts against what the
---next ones may take (it adds to the claims).
---@param s table The snapshot
---@param checkpoint fun()|nil Called after each agent (inline, `FrameScheduler.checkpoint`, so the
---thinking spreads over frames); none on a worker
---@return number[] results `Layout.Result.Stride` numbers per agent
function JobPlanner.planAll(s, checkpoint)
    local p = s.params or {}
    local scene = {
        bodies = s.bodies or {}, bodyCount = s.bodyCount or 0, zones = s.zones or {}, zoneCount = s.zoneCount or 0,
    }
    local claims = {}
    for key, value in pairs(s.claims or {}) do claims[key] = value end
    local function claimed(key) return claims[key] or 0 end
    local function addClaim(key, quantity) claims[key] = (claims[key] or 0) + quantity end

    -- Factions: hostile pairs; an agent or a station of no faction (0) is hostile to marauders only.
    local hostile, marauders = {}, {}
    for _, pair in ipairs(records(s.hostile, L.Hostile)) do
        hostile[pair.A] = hostile[pair.A] or {}
        hostile[pair.B] = hostile[pair.B] or {}
        hostile[pair.A][pair.B], hostile[pair.B][pair.A] = true, true
    end
    for _, id in ipairs(s.marauders or {}) do marauders[id] = true end
    local function isHostile(a, b)
        if a == b then return false end
        if a == 0 or b == 0 then return marauders[a == 0 and b or a] == true end
        return hostile[a] ~= nil and hostile[a][b] == true
    end
    local victims = {}
    for _, row in ipairs(records(s.victims, L.Victims)) do victims[row.Faction] = row.Count end

    local stations, rocks = records(s.stations, L.Station), records(s.rocks, L.Rock)
    local stationIndex, rockIndex = {}, {}
    for i, station in ipairs(stations) do stationIndex[station.Id] = i end
    for i, rock in ipairs(rocks) do rockIndex[rock.Id] = i end
    -- Every place an agent flies to: the stations, then the rocks.
    local places = {}
    for _, station in ipairs(stations) do places[#places + 1] = station end
    for _, rock in ipairs(rocks) do places[#places + 1] = rock end
    local function placeOfStation(id) return stationIndex[id] end
    local function placeOfRock(id) return rockIndex[id] and #stations + rockIndex[id] end

    local masses = {}
    for _, row in ipairs(records(s.itemMasses, L.ItemMass)) do masses[row.Item] = row.Mass end
    local bids = {}
    for _, bid in ipairs(records(s.bids, L.Bid)) do
        bids[bid.Item] = bids[bid.Item] or {}
        table.insert(bids[bid.Item], bid)
    end
    local trades, mining, patrols = records(s.trades, L.Trade), records(s.mining, L.Mining), records(s.patrols, L.Patrol)
    local cargoOf, badRocksOf = {}, {}
    for _, row in ipairs(records(s.cargo, L.Cargo)) do
        cargoOf[row.Agent] = cargoOf[row.Agent] or {}
        cargoOf[row.Agent][row.Item] = row.Count
    end
    for _, row in ipairs(records(s.badRocks, L.BadRock)) do
        badRocksOf[row.Agent] = badRocksOf[row.Agent] or {}
        badRocksOf[row.Agent][row.Rock] = row.Until
    end

    local now, dock, minProfit = p.now or 0, p.dockOverheadS or 0, p.minProfit or 0
    local miningRate, miningRunS = max(p.miningRate or 0.1, 1e-6), p.miningRunS or 180
    local maxMiners, raidS, raidValue = p.maxMinersPerAsteroid or 1, p.raidS or 60, p.raidValue or 0

    -- Legs from a place (its departure point, above it) to each station, by flight numbers: shared by
    -- the agents of one profile.
    local legCache = {}
    local function legs(ship, driveMin, key, fromPlace)
        local byProfile = legCache[key]
        if not byProfile then byProfile = {} legCache[key] = byProfile end
        local times = byProfile[fromPlace]
        if not times then
            local from = places[fromPlace]
            times = timesTo(scene, ship, driveMin, from.X, from.Y + from.Radius * 2, from.Z, stations)
            byProfile[fromPlace] = times
        end
        return times
    end

    local results, R = {}, L.Result
    local agents = records(s.agents, L.Agent)
    for index, a in ipairs(agents) do
        local ship = { maxSpeed = a.MaxSpeed, linearDrag = a.Drag, detourFactor = p.detourFactor or 1 }
        if a.OpenCap > 0 then ship.openCap, ship.rampDownSpeed = a.OpenCap, a.RampDown end
        local profileKey = table.concat({ a.MaxSpeed, a.Drag, a.OpenCap, a.RampDown, a.DriveMin }, ":")
        local fromHere = timesTo(scene, ship, a.DriveMin, a.X, a.Y, a.Z, places)
        local cargo, bad = cargoOf[index] or {}, badRocksOf[index] or {}
        local function room(item)
            local mass = masses[item] or 1
            if a.FreeMass < 0 or mass <= 0 then return math.huge end
            return floor(a.FreeMass / mass)
        end
        local best = nil
        local function consider(job)
            if job and (not best or job.score > best.score) then best = job end
        end

        -- A sale of what it carries, where it is bid for best.
        for item, held in pairs(cargo) do
            if held > 0 then
                for _, bid in ipairs(bids[item] or {}) do
                    local place = placeOfStation(bid.Station)
                    local open = bid.Demand - claimed(claimKey("bid", bid.Station, item))
                    if place and open > 0 and not isHostile(a.Faction, stations[place].Faction) then
                        local quantity = min(held, open)
                        consider({ kind = Kind.Sale, item = item, quantity = quantity, destination = bid.Station,
                            sell = bid.Price, score = bid.Price * quantity / (fromHere[place] + dock) })
                    end
                end
            end
        end

        -- A trade route, within its hold and credits.
        if has(a.Flags, Flag.Hold) then
            for _, offer in ipairs(trades) do
                local from, to = placeOfStation(offer.Source), placeOfStation(offer.Destination)
                if from and to and not isHostile(a.Faction, stations[from].Faction)
                    and not isHostile(a.Faction, stations[to].Faction) then
                    local supply = offer.Supply - claimed(claimKey("ask", offer.Source, offer.Item))
                    local demand = offer.Demand - claimed(claimKey("bid", offer.Destination, offer.Item))
                    local quantity = min(supply, demand, room(offer.Item), floor(a.Balance / max(offer.Buy, 1e-6)))
                    local profit = (offer.Sell - offer.Buy) * quantity
                    if quantity > 0 and profit >= minProfit then
                        local time = fromHere[from] + legs(ship, a.DriveMin, profileKey, from)[to] + 2 * dock
                        consider({ kind = Kind.Trade, item = offer.Item, quantity = quantity, source = offer.Source,
                            destination = offer.Destination, buy = offer.Buy, sell = offer.Sell,
                            score = profit / max(1, time) })
                    end
                end
            end
        end

        -- A mining run, of the length miners make, counting what the station uses meanwhile as demand.
        if has(a.Flags, Flag.Laser) then
            for _, offer in ipairs(mining) do
                local rockPlace, station = placeOfRock(offer.Rock), placeOfStation(offer.Station)
                local avoidUntil = bad[offer.Rock]
                if rockPlace and station and claimed("miners:" .. offer.Rock) < maxMiners
                    and not (avoidUntil and now < avoidUntil) and not isHostile(a.Faction, stations[station].Faction) then
                    local left = offer.Left - claimed(claimKey("rock", offer.Rock, offer.Item))
                    local load = min(room(offer.Item), max(1, floor(miningRate * miningRunS)))
                    local flightS = fromHere[rockPlace] + legs(ship, a.DriveMin, profileKey, rockPlace)[station]
                    local runS = flightS + load / miningRate
                    local demand = offer.Demand + floor(offer.UseRate * runS) - claimed(claimKey("bid", offer.Station, offer.Item))
                    local amount = min(left, demand, load)
                    local value = offer.Sell * amount
                    if amount > 0 and value >= minProfit then
                        consider({ kind = Kind.Mining, item = offer.Item, quantity = amount, source = offer.Rock,
                            destination = offer.Station, sell = offer.Sell,
                            score = value / max(1, flightS + amount / miningRate + dock) })
                    end
                end
            end
        end

        -- Fighting: a raid for a marauder, while there is prey; else a patrol at its faction's station.
        if has(a.Flags, Flag.Armed) then
            if has(a.Flags, Flag.Marauder) then
                local prey = victims[a.Faction] or 0
                if prey > 0 then
                    consider({ kind = Kind.Raid, duration = raidS, score = raidValue * min(1, prey / 3) / raidS })
                end
            elseif a.Faction ~= 0 then
                for _, offer in ipairs(patrols) do
                    local place = placeOfStation(offer.Station)
                    if place and stations[place].Faction == a.Faction and claimed("patrol:" .. offer.Station) < offer.Slots then
                        consider({ kind = Kind.Patrol, destination = offer.Station, payout = offer.Payout,
                            duration = offer.Duration, score = offer.Payout / (fromHere[place] + offer.Duration) })
                    end
                end
            end
        end

        -- Nothing pays: a ship out in space docks at the nearest station with a free port to wait.
        if not best and has(a.Flags, Flag.InSpace) then
            local nearest, nearestTime = nil, math.huge
            for place, station in ipairs(stations) do
                if station.Ports > 0 and fromHere[place] < nearestTime then nearest, nearestTime = station, fromHere[place] end
            end
            if nearest then best = { kind = Kind.Park, destination = nearest.Id, score = 0 } end
        end

        -- What it takes counts against the next agents.
        if best then
            if best.kind == Kind.Sale then
                addClaim(claimKey("bid", best.destination, best.item), best.quantity)
            elseif best.kind == Kind.Trade then
                addClaim(claimKey("ask", best.source, best.item), best.quantity)
                addClaim(claimKey("bid", best.destination, best.item), best.quantity)
            elseif best.kind == Kind.Mining then
                addClaim(claimKey("rock", best.source, best.item), best.quantity)
                addClaim("miners:" .. best.source, 1)
                addClaim(claimKey("bid", best.destination, best.item), best.quantity)
            elseif best.kind == Kind.Patrol then
                addClaim("patrol:" .. best.destination, 1)
            end
        end
        local base = (index - 1) * R.Stride
        local job = best or { kind = Kind.None, score = 0 }
        results[base + R.Agent + 1] = a.Id
        results[base + R.Kind + 1] = job.kind
        results[base + R.Item + 1] = job.item or 0
        results[base + R.Quantity + 1] = job.quantity or 0
        results[base + R.Source + 1] = job.source or 0
        results[base + R.Destination + 1] = job.destination or 0
        results[base + R.Buy + 1] = job.buy or 0
        results[base + R.Sell + 1] = job.sell or 0
        results[base + R.Payout + 1] = job.payout or 0
        results[base + R.Duration + 1] = job.duration or 0
        results[base + R.Score + 1] = job.score or 0
        if checkpoint then checkpoint() end
    end
    return results
end

return JobPlanner
