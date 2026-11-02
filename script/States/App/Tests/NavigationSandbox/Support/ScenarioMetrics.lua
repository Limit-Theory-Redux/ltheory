-- Records one sandbox scenario run: arrivals, collisions, deadlocks and frame times, each
-- logged as it happens, with a summary at the end. Thresholds are authored in metres.
-- Design: doc/script/navigation-sandbox.md
local ScaleConfig = require("Config.Gen.UniverseScaleConfig")

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

---@alias SandboxShipState "flying"|"arrived"|"deadlocked"

---@class SandboxShip
---@field entity Entity
---@field rb RigidBody
---@field target Position
---@field label string
---@field state SandboxShipState

---@class ScenarioMetrics
---@overload fun(name: string, ships: SandboxShip[]): ScenarioMetrics
local ScenarioMetrics = Class("ScenarioMetrics", function(self, name, ships)
    self.name = name
    self.ships = ships
    self.byId = {}
    for _, ship in ipairs(ships) do
        ship.state = "flying"
        ship.bestDistance = math.huge
        ship.progressTime = 0
        self.byId[ship.entity.id] = ship
    end
    self.touching = {} -- pair key -> true while the two bodies overlap
    self.arrivals, self.collisions, self.deadlocks = 0, 0, 0
    self.frames, self.frameMsSum, self.frameMsMax = 0, 0, 0
end)

---Game units back to metres, for log lines.
local function metres(gameUnits)
    return gameUnits / ScaleConfig:toGameUnits(1, "starSystem")
end

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

---@param ship SandboxShip
---@param simTime number Seconds of simulated time since the scenario started
function ScenarioMetrics:updateProgress(ship, simTime)
    local distance = ship.rb:getPos():distance(ship.target)
    if distance <= ARRIVAL_RANGE and ship.rb:getSpeed() <= ARRIVAL_SPEED then
        ship.state = "arrived"
        self.arrivals = self.arrivals + 1
        self:log("%s arrived at %.2f s", ship.label, simTime)
    elseif distance <= ship.bestDistance - PROGRESS_STEP then
        ship.bestDistance = distance
        ship.progressTime = simTime
    elseif simTime - ship.progressTime >= DEADLOCK_WINDOW_S then
        ship.state = "deadlocked"
        self.deadlocks = self.deadlocks + 1
        self:log("%s deadlocked at %.2f s, %.0f m from its target", ship.label, simTime, metres(distance))
    end
end

---Name of a body found by a query: its sandbox label, or its id.
function ScenarioMetrics:bodyLabel(id)
    local ship = self.byId[id]
    return ship and ship.label or string.format("body %d", id)
end

---Count a collision when a ship's bounding sphere starts to overlap another body.
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
                self.collisions = self.collisions + 1
                self:log("collision of %s and %s at %.2f s", ship.label, self:bodyLabel(other), simTime)
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

---@return integer unfinished Ships still flying
function ScenarioMetrics:unfinished()
    return #self.ships - self.arrivals - self.deadlocks
end

---Log the summary line.
---@param simTime number
function ScenarioMetrics:summarize(simTime)
    local measured = math.max(self.frames - WARMUP_FRAMES, 1)
    self:log("%d/%d arrived, %d collisions, %d deadlocked, %d unfinished, %.2f s simulated, "
        .. "frame %.2f ms mean, %.2f ms max over %d frames",
        self.arrivals, #self.ships, self.collisions, self.deadlocks, self:unfinished(), simTime,
        self.frameMsSum / measured, self.frameMsMax, measured)
end

return ScenarioMetrics
