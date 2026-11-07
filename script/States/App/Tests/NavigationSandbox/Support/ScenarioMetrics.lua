-- Records one sandbox scenario run: arrivals, waypoints, collisions, deadlocks and the times
-- of the navigation step, the glue and the actuation, with events logged as they happen and a
-- summary at the end. Thresholds are authored in metres.
-- Design: doc/script/navigation-sandbox.md
local ScaleConfig   = require("Config.Gen.UniverseScaleConfig")
local ShipBehaviour = require("States.App.Tests.NavigationSandbox.Support.ShipBehaviour")
local Metres        = require("States.App.Tests.NavigationSandbox.Support.Metres")

-- A ship has arrived when it is this close to its target and this slow.
local ARRIVAL_RANGE_M   = 100
local ARRIVAL_SPEED_MS  = 50
-- A ship that has not come this much closer to its target for this long is deadlocked.
local PROGRESS_STEP_M   = 10
local DEADLOCK_WINDOW_S = 10
-- Frames left out of the frame-time figures at the start, which pay for spawning.
local WARMUP_FRAMES     = 1
local MS_PER_SECOND     = 1000

local ARRIVAL_RANGE = ScaleConfig:toGameUnits(ARRIVAL_RANGE_M, "starSystem")
local ARRIVAL_SPEED = ScaleConfig:toGameUnits(ARRIVAL_SPEED_MS, "starSystem")
local PROGRESS_STEP = ScaleConfig:toGameUnits(PROGRESS_STEP_M, "starSystem")

---What it takes to have arrived, for the ships judged by their behaviour.
---@type SandboxJudgingRules
local RULES = { range = ARRIVAL_RANGE, speed = ARRIVAL_SPEED }

---@alias SandboxShipState "flying"|"arrived"|"deadlocked"

---@class SandboxShip
---@field entity Entity
---@field rb RigidBody
---@field input ControlInputComponent
---@field agent integer Id in the `NavWorld`
---@field target Position The goal: the destination, or the current waypoint of a route
---@field route Position[]|nil Waypoints flown in a loop
---@field waypointIndex integer|nil Index into `route` of the current waypoint
---@field label string
---@field state SandboxShipState
---@field released boolean|nil Whether the ship's goal was cleared after it finished
---@field spec SandboxShipSpec
---@field behaviour SandboxBehaviour|nil Set when the spec has a behaviour
---@field kind "ship"

---@class SandboxObstacle
---@field entity Entity
---@field rb RigidBody
---@field radius number Game units
---@field circle SandboxObstacleCircle|nil
---@field label string
---@field kind "obstacle"

---@class SandboxObstacleCircle
---@field centre Position
---@field radius number Game units
---@field periodS number Seconds for one turn

---Mean and largest value of a series of times.
---@class TimingSeries
---@field sum number
---@field max number

---@class ScenarioMetrics
---@overload fun(name: string, ships: SandboxShip[], obstacles: SandboxObstacle[], avoidance: SandboxAvoidanceSpec|nil): ScenarioMetrics
local ScenarioMetrics = Class("ScenarioMetrics", function(self, name, ships, obstacles, avoidance)
    self.name = name
    self.ships = ships
    self.obstacles = obstacles
    self.avoidance = avoidance -- the scenario's tuning; nil for the navigation world's defaults
    self.byId = {} -- entity id -> the ship or obstacle, for labels and kinds
    for _, ship in ipairs(ships) do
        ship.state = "flying"
        ship.kind = "ship"
        ship.bestDistance = math.huge
        ship.progressTime = 0
        self.byId[ship.entity.id] = ship
    end
    for _, obstacle in ipairs(obstacles) do
        obstacle.kind = "obstacle"
        self.byId[obstacle.entity.id] = obstacle
    end
    self.touching = {} -- pair key -> true while the two bodies overlap
    self.arrivals, self.deadlocks, self.waypoints = 0, 0, 0
    self.passes, self.slowPasses = 0, 0 -- Seek points passed, and those passed too slowly
    self.obstacleCollisions, self.shipCollisions = 0, 0
    self.frames, self.frameMsSum, self.frameMsMax = 0, 0, 0
    self.timedFrames = 0
    self.timings = {
        step = { sum = 0, max = 0 },
        glue = { sum = 0, max = 0 },
        actuation = { sum = 0, max = 0 },
        total = { sum = 0, max = 0 },
    }
end)

local metres = Metres.toMetres

function ScenarioMetrics:log(fmt, ...)
    Log.Info("NavigationSandbox: %s: " .. fmt, self.name, ...)
end

---@param dt number Seconds of the frame
function ScenarioMetrics:recordFrame(dt)
    self.frames = self.frames + 1
    if self.frames <= WARMUP_FRAMES then return end
    local ms = dt * MS_PER_SECOND
    self.frameMsSum = self.frameMsSum + ms
    self.frameMsMax = math.max(self.frameMsMax, ms)
end

---@param series TimingSeries
---@param ms number
local function addTiming(series, ms)
    series.sum = series.sum + ms
    series.max = math.max(series.max, ms)
end

---Record the milliseconds one frame spent in the navigation step, in the glue between the
---step and the ships' control inputs, and in the actuation. The warmup frames are left out.
---@param stepMs number
---@param glueMs number
---@param actuationMs number
function ScenarioMetrics:recordTimings(stepMs, glueMs, actuationMs)
    if self.frames <= WARMUP_FRAMES then return end
    self.timedFrames = self.timedFrames + 1
    addTiming(self.timings.step, stepMs)
    addTiming(self.timings.glue, glueMs)
    addTiming(self.timings.actuation, actuationMs)
    addTiming(self.timings.total, stepMs + glueMs + actuationMs)
end

---@param series TimingSeries
---@return number mean Milliseconds per timed frame
function ScenarioMetrics:meanMs(series)
    return series.sum / math.max(self.timedFrames, 1)
end

---Move a route ship on to its next waypoint, which restarts its progress check.
---@param ship SandboxShip A ship with a route
---@param simTime number
function ScenarioMetrics:waypointReached(ship, simTime)
    ship.waypointIndex = ship.waypointIndex % #ship.route + 1
    ship.target = ship.route[ship.waypointIndex]
    ship.bestDistance = math.huge
    ship.progressTime = simTime
    self.waypoints = self.waypoints + 1
end

---Record that a Seek ship passed one of its points.
---@param ship SandboxShip
---@param index integer Index of the point in the behaviour's list
---@param speed number Game units per second
---@param fastEnough boolean Whether the ship was still at speed, not braking for the point
---@param simTime number
function ScenarioMetrics:seekPassed(ship, index, speed, fastEnough, simTime)
    self.passes = self.passes + 1
    if not fastEnough then self.slowPasses = self.slowPasses + 1 end
    self:log("%s passed point %d at %.2f s, %.0f m/s%s", ship.label, index, simTime,
        Metres.toMetres(speed), fastEnough and "" or " (too slow)")
end

---@class SandboxNeighbour
---@field obstacle SandboxObstacle
---@field gap number Game units from the ship's centre to the obstacle's surface
---@field centreDistance number Game units from the ship's centre to the obstacle's centre

---The obstacles whose surfaces are closest to the ship, nearest first.
---@param ship SandboxShip
---@param count integer At most this many
---@return SandboxNeighbour[]
function ScenarioMetrics:nearestObstacles(ship, count)
    local position = ship.rb:getPos()
    local found = {}
    for _, obstacle in ipairs(self.obstacles) do
        local centreDistance = position:distance(obstacle.rb:getPos())
        table.insert(found, {
            obstacle = obstacle,
            gap = centreDistance - obstacle.radius,
            centreDistance = centreDistance,
        })
    end
    table.sort(found, function(a, b) return a.gap < b.gap end)
    return { table.unpack(found, 1, count) }
end

---A deadlocked ship's two nearest obstacles for a log line: each one's label and surface gap,
---the gap between their surfaces, and, when the scenario sets the avoidance tuning, each one's
---centre distance as a multiple of its reach (1 to `aimFactor` is the aim band, where the
---detour finds no tangent).
---@param ship SandboxShip
---@return string
function ScenarioMetrics:describeNeighbours(ship)
    local near = self:nearestObstacles(ship, 2)
    if #near == 0 then return "no obstacles" end
    local shipRadius = ship.rb:getBoundingRadius()
    local parts = {}
    for _, neighbour in ipairs(near) do
        local text = string.format("%s %.0f m", neighbour.obstacle.label, metres(neighbour.gap))
        local tuning = self.avoidance
        if tuning then
            local reach = neighbour.obstacle.radius + shipRadius * (1 + tuning.clearanceRadii)
            text = text .. string.format(" (%.2f reach)", neighbour.centreDistance / reach)
        end
        table.insert(parts, text)
    end
    local text = "nearest " .. table.concat(parts, ", ")
    if #near == 2 then
        local between = near[1].obstacle.rb:getPos():distance(near[2].obstacle.rb:getPos())
            - near[1].obstacle.radius - near[2].obstacle.radius
        text = text .. string.format(", %.0f m between them", metres(between))
    end
    return text
end

---@param ship SandboxShip
---@param simTime number Seconds of simulated time since the scenario started
function ScenarioMetrics:updateProgress(ship, simTime)
    local distance = ship.rb:getPos():distance(ship.target)
    local behaviour = ship.behaviour
    local arrived
    if behaviour then
        arrived = ShipBehaviour.judge(ship, simTime, RULES)
    else
        arrived = not ship.route and distance <= ARRIVAL_RANGE and ship.rb:getSpeed() <= ARRIVAL_SPEED
    end
    if arrived then
        ship.state = "arrived"
        self.arrivals = self.arrivals + 1
        self:log("%s arrived at %.2f s", ship.label, simTime)
    elseif behaviour and ShipBehaviour.exemptFromDeadlock(ship) then
        return
    elseif distance <= ship.bestDistance - PROGRESS_STEP then
        ship.bestDistance = distance
        ship.progressTime = simTime
    elseif simTime - ship.progressTime >= DEADLOCK_WINDOW_S then
        ship.state = "deadlocked"
        self.deadlocks = self.deadlocks + 1
        self:log("%s deadlocked at %.2f s, %.0f m from its target, speed %.0f m/s, %s",
            ship.label, simTime, metres(distance), metres(ship.rb:getSpeed()), self:describeNeighbours(ship))
    end
end

---Name of a body found by a query: its sandbox label, or its id.
function ScenarioMetrics:bodyLabel(id)
    local known = self.byId[id]
    return known and known.label or string.format("body %d", id)
end

---Count a collision when a ship's bounding sphere starts to overlap another body. A body that
---is not a sandbox ship counts as an obstacle.
---@param world Physics
---@param simTime number
function ScenarioMetrics:updateCollisions(world, simTime)
    local touching = {}
    for _, ship in ipairs(self.ships) do
        local own = ship.entity.id
        local found = world:queryRadius(ship.rb:getPos(), ship.rb:getBoundingRadius(), ship.rb)
        for i = 0, found.len - 1 do
            local other = tonumber(found.ids[i])
            local key = own < other and (own .. ":" .. other) or (other .. ":" .. own)
            if not touching[key] and not self.touching[key] then
                local known = self.byId[other]
                local detail = ""
                if known and known.kind == "ship" then
                    self.shipCollisions = self.shipCollisions + 1
                else
                    self.obstacleCollisions = self.obstacleCollisions + 1
                    if known then
                        local toCentre = (known.rb:getPos() - ship.rb:getPos()):normalize()
                        local closing = (ship.rb:getVelocity() - known.rb:getVelocity()):dot(toCentre)
                        detail = string.format(", closing on it at %.0f m/s", metres(closing))
                    end
                end
                self:log("collision of %s and %s at %.2f s, ship speed %.0f m/s%s", ship.label,
                    self:bodyLabel(other), simTime, metres(ship.rb:getSpeed()), detail)
            end
            touching[key] = true
        end
    end
    self.touching = touching
end

---Update every flying ship after a physics step.
---@param world Physics
---@param simTime number
function ScenarioMetrics:update(world, simTime)
    for _, ship in ipairs(self.ships) do
        if ship.state == "flying" then self:updateProgress(ship, simTime) end
    end
    self:updateCollisions(world, simTime)
end

---@return boolean done Every ship has arrived or is deadlocked
function ScenarioMetrics:isDone()
    return self.arrivals + self.deadlocks == #self.ships
end

---@return integer unfinished Ships still flying that were to arrive: route ships fly on
function ScenarioMetrics:unfinished()
    local count = 0
    for _, ship in ipairs(self.ships) do
        if ship.state == "flying" and not ship.route then count = count + 1 end
    end
    return count
end

---@return string[] failures What fails the run: a collision, a deadlock, an unfinished ship
function ScenarioMetrics:failures()
    local failures = {}
    if self.obstacleCollisions > 0 then
        table.insert(failures, string.format("%d ship-obstacle collisions", self.obstacleCollisions))
    end
    if self.shipCollisions > 0 then
        table.insert(failures, string.format("%d ship-ship collisions", self.shipCollisions))
    end
    if self.deadlocks > 0 then
        table.insert(failures, string.format("%d deadlocked ships", self.deadlocks))
    end
    if self:unfinished() > 0 then
        table.insert(failures, string.format("%d ships still flying", self:unfinished()))
    end
    if self.slowPasses > 0 then
        table.insert(failures, string.format("%d seek points passed too slowly", self.slowPasses))
    end
    return failures
end

---Log the summary lines.
---@param simTime number
function ScenarioMetrics:summarize(simTime)
    local measured = math.max(self.frames - WARMUP_FRAMES, 1)
    self:log("%d/%d arrived, %d waypoints reached, %d seek points passed, %d ship-obstacle and %d ship-ship collisions, "
        .. "%d deadlocked, %d unfinished, %.2f s simulated, frame %.2f ms mean, %.2f ms max over %d frames",
        self.arrivals, #self.ships, self.waypoints, self.passes, self.obstacleCollisions, self.shipCollisions,
        self.deadlocks, self:unfinished(), simTime, self.frameMsSum / measured, self.frameMsMax, measured)
    for _, ship in ipairs(self.ships) do
        local maxError = ShipBehaviour.maxStationError(ship)
        if maxError then
            self:log("%s: largest distance from its station since it was reached: %.0f m", ship.label,
                Metres.toMetres(maxError))
        end
        local turnRate = ShipBehaviour.stationTurnRate(ship)
        if turnRate then
            self:log("%s: largest turn rate while held on station: %.3f rad/s", ship.label, turnRate)
        end
    end
    local t = self.timings
    self:log("navigation ms per frame, mean (max) over %d frames: step %.3f (%.3f), glue %.3f (%.3f), "
        .. "actuation %.3f (%.3f), total %.3f (%.3f)", self.timedFrames,
        self:meanMs(t.step), t.step.max, self:meanMs(t.glue), t.glue.max,
        self:meanMs(t.actuation), t.actuation.max, self:meanMs(t.total), t.total.max)
end

---The distance from its target, in game units, within which a ship can arrive.
ScenarioMetrics.ARRIVAL_RANGE = ARRIVAL_RANGE

---The rules for a ship judged by its behaviour.
ScenarioMetrics.RULES = RULES

return ScenarioMetrics
