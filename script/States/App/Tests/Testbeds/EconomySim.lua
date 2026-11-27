-- Economy simulation testbed: the game's star system and its planned economy (as Testbeds/EconomyGame),
-- run as fast as the machine allows for simulated hours, with no 3D rendering: a top-down map of the
-- system (planets, stations by faction, ships by what they do) and charts of the economy over time
-- (average prices, stock, agents by state, credits), from an `EconomySampler`. The simulation steps at
-- a fixed 0.05 s (`Application.fixedSimStepS`) as many times a frame as keep a frame near
-- `FRAME_TARGET_S`.
--
--   LTHEORY_BACKGROUND=1 LTHEORY_ECONOMY_HOURS=4 LTHEORY_ECONOMY_LOG=economy.csv \
--       ./bin/ltr.exe Testbeds/EconomySim
--
-- Environment (all optional; and those of Testbeds/EconomyGame: SEED, SHIPS):
--   LTHEORY_ECONOMY_HOURS  simulated hours to run, then quit with a summary (default: until closed)
--   LTHEORY_ECONOMY_LOG    a CSV file for every sample (`time,kind,name,key,value`) and the summary
--   LTHEORY_ECONOMY_SAMPLE simulated seconds between samples (default 60)
-- Design: doc/script/economy-sim.md

local Registry          = require('Core.ECS.Registry')
local DrawEx            = require('UI.DrawEx')
local PhysicsComponents = require('Modules.Physics.Components')
local CelestialComponents = require('Modules.CelestialObjects.Components')
local Economy           = require('Modules.Economy.Components')
local Factions          = require('Modules.Factions.Components')
local FactionManager    = require('Modules.Factions.Managers.FactionManager')
local PeopleManager     = require('Modules.People.Managers.PeopleManager')
local MarketplaceSystem = require('Modules.Economy.Systems.MarketplaceSystem')
local SpawnerSystem     = require('Modules.Spawning.Systems.SpawnerSystem')
local EconomySampler    = require('Modules.Economy.Managers.EconomySampler')
local EconomyGame       = require('States.App.Tests.Testbeds.EconomyGame')
local MiningEffectsSystem = require('Modules.Economy.Systems.MiningEffectsSystem')

---@class EconomySimTestbed: EconomyGameTestbed
local EconomySim = Subclass("EconomySimTestbed", EconomyGame)

local SIM_STEP_S = 0.05
-- Real seconds a frame aims at: the steps a frame adapt to it.
local FRAME_TARGET_S = 0.15
local MAX_STEPS = 2000
local HOURS = tonumber(os.getenv('LTHEORY_ECONOMY_HOURS'))
local FONT = 'Unageo-Medium'

-- Ship colours by what the agent does.
local STATE_COLORS = {
    Idle = Color(0.5, 0.5, 0.5, 1), Waiting = Color(0.5, 0.5, 0.5, 1), Undocking = Color(0.4, 0.6, 1, 1),
    DockingToBuy = Color(0.3, 0.9, 0.4, 1), Buying = Color(0.3, 0.9, 0.4, 1),
    DockingToSell = Color(0.6, 1, 0.5, 1), Selling = Color(0.6, 1, 0.5, 1),
    FlyingToMine = Color(1, 0.65, 0.2, 1), Mining = Color(1, 0.55, 0.1, 1), Fighting = Color(1, 0.25, 0.2, 1),
}
-- Line colours for chart series, in turn.
local SERIES_COLORS = {
    Color(0.35, 0.7, 1, 1), Color(1, 0.7, 0.25, 1), Color(0.45, 0.95, 0.5, 1), Color(0.9, 0.45, 1, 1),
    Color(1, 0.4, 0.4, 1), Color(0.4, 1, 0.95, 1), Color(1, 1, 0.5, 1), Color(0.7, 0.7, 0.7, 1),
}

local stateNames = nil
---@param state integer
---@return string
local function stateName(state)
    if not stateNames then
        stateNames = {}
        for name, value in pairs(Enums.EconomyAgentState) do stateNames[value] = name end
    end
    return stateNames[state] or "Idle"
end

function EconomySim:onInit()
    EconomyGame.onInit(self)
    -- Nothing is drawn in 3D: effects nobody sees aren't made.
    MiningEffectsSystem.enabled = false
    self.fixedSimStepS = SIM_STEP_S
    self.simStepsPerFrame = 20
    self.sampler = EconomySampler({
        intervalS = tonumber(os.getenv('LTHEORY_ECONOMY_SAMPLE')) or 60,
        csvPath = os.getenv('LTHEORY_ECONOMY_LOG'),
    })
    self.realClock = TimeStamp.Now()
    self.lastFrameAt = nil
    self.simAtLastFrame = 0
    self.speed = 0
end

---@param data EventData
function EconomySim:onPreSim(data)
    EconomyGame.onPreSim(self, data)
    if not self.playerShip then return end
    local now = MarketplaceSystem:now()
    local samples = self.sampler.samples
    self.sampler:update(now)
    if self.sampler.samples ~= samples then
        Log.Info("EconomySim: %.0f s simulated, x%.1f real time, %d steps a frame", now, self.speed, self.simStepsPerFrame)
    end
    if HOURS and now >= HOURS * 3600 and not self.finished then
        self.finished = true
        self.sampler:finish()
        self:quit()
    end
end

---The steps a frame follow the frame's real time: more while frames are quick, fewer when slow.
function EconomySim:adaptSteps()
    local now = self.realClock:getElapsed()
    if self.lastFrameAt then
        local frame = now - self.lastFrameAt
        if frame < FRAME_TARGET_S * 0.8 then
            self.simStepsPerFrame = math.min(MAX_STEPS, math.ceil(self.simStepsPerFrame * 1.2) + 1)
        elseif frame > FRAME_TARGET_S * 1.25 then
            self.simStepsPerFrame = math.max(1, math.floor(self.simStepsPerFrame * 0.8))
        end
        local sim = MarketplaceSystem:now()
        if frame > 0 then self.speed = (sim - self.simAtLastFrame) / frame end
        self.simAtLastFrame = sim
    end
    self.lastFrameAt = now
end

---@param data EventData
function EconomySim:onRender(data)
    -- Starting up (the menu, the universe being made): the game draws as usual.
    if not self.playerShip then return EconomyGame.onRender(self, data) end
    self:adaptSteps()
    local w, h = Window:width(), Window:height()
    self:immediateUI(function()
        DrawEx.SimpleRect(0, 0, w, h, Color(0.03, 0.035, 0.05, 1))
        local mapW = math.floor(w * 0.58)
        self:drawMap(10, 40, mapW - 20, h - 50)
        self:drawCharts(mapW, 40, w - mapW - 10, h - 50)
        local now = MarketplaceSystem:now()
        DrawEx.TextAdditive(FONT, string.format(
            "Economy simulation  |  %02d:%02d:%02d simulated  |  x%.0f real time  |  %d steps a frame  |  %d agents",
            math.floor(now / 3600), math.floor(now / 60) % 60, math.floor(now) % 60, self.speed, self.simStepsPerFrame,
            #SpawnerSystem:getAgents()), 14, 14, 12, w - 28, 20, 0.9, 0.9, 0.9, 1, 0.0, 0.5)
    end)
end

---The top-down map: x and z of the system, fitted to the box.
function EconomySim:drawMap(x, y, sx, sy)
    DrawEx.RectOutline(x, y, sx, sy, Color(0.3, 0.35, 0.45, 1))
    local points = {}
    local function add(entity)
        local rbCmp = entity:get(PhysicsComponents.RigidBody)
        local body = rbCmp and rbCmp:getRigidBody()
        if body then
            local p = body:getPos()
            points[#points + 1] = p
            return p
        end
    end
    local stations, ships, planets = {}, {}, {}
    for _, station in ipairs(SpawnerSystem:getStations()) do
        if Registry:hasEntity(station.entity) then
            local p = add(station.entity)
            if p then stations[#stations + 1] = { p = p, station = station } end
        end
    end
    for _, ship in ipairs(SpawnerSystem:getAgents()) do
        if Registry:hasEntity(ship.entity) then
            local p = add(ship.entity)
            if p then ships[#ships + 1] = { p = p, ship = ship } end
        end
    end
    for entity in Registry:view(CelestialComponents.Planet) do
        local p = add(entity)
        if p then planets[#planets + 1] = p end
    end
    if #points == 0 then return end
    local minX, maxX, minZ, maxZ = math.huge, -math.huge, math.huge, -math.huge
    for _, p in ipairs(points) do
        minX, maxX, minZ, maxZ = math.min(minX, p.x), math.max(maxX, p.x), math.min(minZ, p.z), math.max(maxZ, p.z)
    end
    local spanX, spanZ = math.max(maxX - minX, 1e-6), math.max(maxZ - minZ, 1e-6)
    local scale = math.min((sx - 40) / spanX, (sy - 40) / spanZ)
    local cx, cz = (minX + maxX) / 2, (minZ + maxZ) / 2
    local function screen(p)
        return x + sx / 2 + (p.x - cx) * scale, y + sy / 2 + (p.z - cz) * scale
    end

    for _, p in ipairs(planets) do
        local px, py = screen(p)
        DrawEx.Ring(px, py, 10, Color(0.3, 0.5, 0.8, 1), false)
    end
    for _, entry in ipairs(ships) do
        local agent = PeopleManager:commandOf(entry.ship.entity, Economy.EconomyAgent)
        local color = agent and STATE_COLORS[stateName(agent:getState())] or STATE_COLORS.Idle
        local px, py = screen(entry.p)
        DrawEx.SimpleRect(px - 1.5, py - 1.5, 3, 3, color)
    end
    for _, entry in ipairs(stations) do
        local px, py = screen(entry.p)
        local faction = FactionManager:factionOf(entry.station.entity)
        local color = faction and faction:get(Factions.Faction):getColor() or Color(0.8, 0.8, 0.8, 1)
        DrawEx.SimpleRect(px - 5, py - 5, 10, 10, color)
        DrawEx.TextAdditive(FONT, entry.station.name, 11, px + 8, py - 7, 160, 14, 0.85, 0.85, 0.85, 1, 0.0, 0.5)
    end
end

---A chart of the series named with `prefix`, over the sampler's kept samples.
function EconomySim:drawChart(title, prefix, x, y, sx, sy)
    DrawEx.RectOutline(x, y, sx, sy, Color(0.3, 0.35, 0.45, 1))
    DrawEx.TextAdditive(FONT, title, 12, x + 6, y + 2, sx, 16, 0.85, 0.85, 0.85, 1, 0.0, 0.5)
    local names = {}
    for name in pairs(self.sampler.series) do
        if name:sub(1, #prefix) == prefix then names[#names + 1] = name end
    end
    table.sort(names)
    local top = 0
    for _, name in ipairs(names) do
        for _, value in ipairs(self.sampler.series[name]) do top = math.max(top, value) end
    end
    if top <= 0 then return end
    local plotX, plotY, plotW, plotH = x + 6, y + 20, sx - 12, sy - 26
    for index, name in ipairs(names) do
        local list = self.sampler.series[name]
        local color = SERIES_COLORS[(index - 1) % #SERIES_COLORS + 1]
        local count = #list
        for i = 2, count do
            local x1 = plotX + (i - 2) / math.max(1, count - 1) * plotW
            local x2 = plotX + (i - 1) / math.max(1, count - 1) * plotW
            DrawEx.SimpleLine(x1, plotY + plotH * (1 - list[i - 1] / top), x2, plotY + plotH * (1 - list[i] / top), color, 1)
        end
        DrawEx.TextAdditive(FONT, string.format("%s %.0f", name:sub(#prefix + 1), list[count] or 0), 10,
            plotX + plotW - 140, plotY + (index - 1) * 12, 140, 12, color.r, color.g, color.b, 1, 0.0, 0.5)
    end
    DrawEx.TextAdditive(FONT, string.format("%.0f", top), 10, plotX, plotY, 80, 12, 0.6, 0.6, 0.6, 1, 0.0, 0.5)
end

function EconomySim:drawCharts(x, y, sx, sy)
    local h = math.floor((sy - 30) / 4)
    self:drawChart("Average price", "price:", x, y, sx, h)
    self:drawChart("Stock at stations", "stock:", x, y + (h + 10), sx, h)
    self:drawChart("Agents by state", "state:", x, y + 2 * (h + 10), sx, h)
    self:drawChart("Credits", "credits:", x, y + 3 * (h + 10), sx, h)
end

return EconomySim
