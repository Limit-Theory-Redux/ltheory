--[[
Sensor setups test: starts the Sensor Testbed and goes through every setup
(Config/Scenarios/SensorTestbedSetups.lua) the way Regenerate (B) does. In
each one it switches through the three hulls and checks the readings that
show what the setup is for, such as how far each hull hears, classes and
identifies the same ships, which hiding places work, and whether five fighters
merge into one blob. On top of that it fires a pulse and runs a scan in
"Passive against active", and checks who hears the player's ship in "Being
heard", listening and scanning. After the last setup, Regenerate must start
the first one again.

Run it from the repository root after a build:

    cargo run -p ltr -- SensorSetupsTest

Exit code 0 and "SensorSetupsTest: passed" = every setup built, ran and read
as expected. Failed checks are listed and stop the game with exit code 1, as
does a script error (`Core.Call`).

It needs a GPU and a window, so CI does not run it.
]]

local Game = require('States.App.LTheoryRedux')
local Scenarios = require('Config.Scenarios')
local Setups = require('Config.Scenarios.SensorTestbedSetups')
local SensorComponents = require('Modules.Sensors.Components')
local SensorModel = require('Modules.Sensors.SensorModel')

Game.autostartScenario = Scenarios.find('SensorTestbed')
Game.autostartSeed = 1

local SETTLE_FRAMES = 3      -- after a change, before reading
local SCAN_FRAME_LIMIT = 2000 -- frames a scan may take before it fails

-- Expected tiers per setup and hull, by label. "!why" checks a reason too.
local EXPECT = {
    hulls = {
        fighter = {
            ["Fighter at 1 km"] = 1, ["Freighter at 1 km"] = 2, ["Capital ship at 1 km"] = 3, ["Station at 1 km"] = 3,
            ["Fighter at 3 km"] = 1, ["Freighter at 3 km"] = 1, ["Capital ship at 3 km"] = 3, ["Station at 3 km"] = 3,
            ["Fighter at 10 km"] = 1, ["Freighter at 10 km"] = 1, ["Capital ship at 10 km"] = 2, ["Station at 10 km"] = 2,
            ["Fighter at 30 km"] = 0, ["Freighter at 30 km"] = 1, ["Capital ship at 30 km"] = 1, ["Station at 30 km"] = 2,
            ["Fighter at 100 km"] = 0, ["Freighter at 100 km"] = 0, ["Capital ship at 100 km"] = 1, ["Station at 100 km"] = 1,
        },
        frigate = {
            ["Fighter at 1 km"] = 2, ["Freighter at 1 km"] = 3, ["Capital ship at 1 km"] = 3, ["Station at 1 km"] = 3,
            ["Fighter at 3 km"] = 2, ["Freighter at 3 km"] = 2, ["Capital ship at 3 km"] = 3, ["Station at 3 km"] = 3,
            ["Fighter at 10 km"] = 1, ["Freighter at 10 km"] = 1, ["Capital ship at 10 km"] = 3, ["Station at 10 km"] = 3,
            ["Fighter at 30 km"] = 0, ["Freighter at 30 km"] = 1, ["Capital ship at 30 km"] = 2, ["Station at 30 km"] = 3,
            ["Fighter at 100 km"] = 0, ["Freighter at 100 km"] = 0, ["Capital ship at 100 km"] = 1, ["Station at 100 km"] = 2,
        },
        capital = {
            ["Fighter at 1 km"] = 3, ["Freighter at 1 km"] = 3, ["Capital ship at 1 km"] = 3, ["Station at 1 km"] = 3,
            ["Fighter at 3 km"] = 2, ["Freighter at 3 km"] = 3, ["Capital ship at 3 km"] = 3, ["Station at 3 km"] = 3,
            ["Fighter at 10 km"] = 2, ["Freighter at 10 km"] = 2, ["Capital ship at 10 km"] = 3, ["Station at 10 km"] = 3,
            ["Fighter at 30 km"] = 1, ["Freighter at 30 km"] = 2, ["Capital ship at 30 km"] = 3, ["Station at 30 km"] = 3,
            ["Fighter at 100 km"] = 0, ["Freighter at 100 km"] = 1, ["Capital ship at 100 km"] = 2, ["Station at 100 km"] = 3,
        },
    },
    hiding = {
        fighter = {
            ["In the open"] = 1, ["Next to a rock"] = "0!lost in the background",
            ["Behind a rock"] = "0!an asteroid blocks EM and heat", ["Cloaked"] = 1, ["Running cold"] = 0,
            ["Firing"] = 0, ["Firing, in the star's glare"] = 0, ["In front of the planet"] = 0,
            ["Behind the planet"] = "0!behind the planet",
        },
        frigate = {
            ["In the open"] = 1, ["Next to a rock"] = "0!lost in the background",
            ["Behind a rock"] = "0!an asteroid blocks EM and heat", ["Cloaked"] = 1, ["Running cold"] = 0,
            ["Firing"] = 1, ["Firing, in the star's glare"] = 0, ["In front of the planet"] = 0,
            ["Behind the planet"] = "0!behind the planet",
        },
        capital = {
            ["In the open"] = 2, ["Next to a rock"] = 2, ["Behind a rock"] = "0!an asteroid blocks EM and heat",
            ["Cloaked"] = 2, ["Running cold"] = 2, ["Firing"] = 1, ["Behind the planet"] = "0!behind the planet",
        },
    },
    active = {
        frigate = { ["Freighter"] = 2, ["Wreck"] = 0, ["Station"] = 2, ["Fighter"] = 2 },
    },
    merging = {
        frigate = { ["Fighter 1"] = 1, ["Fighter 2"] = 1, ["Fighter 3"] = 1, ["Fighter 4"] = 1, ["Fighter 5"] = 1 },
        capital = { ["Fighter 1"] = 2, ["Fighter 2"] = 2, ["Fighter 3"] = 2, ["Fighter 4"] = 2, ["Fighter 5"] = 2 },
    },
}

-- Who hears the player's frigate in "Being heard", listening and scanning
local HEARD = {
    listening = { ["Frigate at 20 km"] = true, ["Frigate at 40 km"] = true, ["Frigate at 80 km"] = false,
        ["Frigate at 160 km"] = false, ["Frigate at 400 km"] = false },
    scanning = { ["Frigate at 20 km"] = true, ["Frigate at 40 km"] = true, ["Frigate at 80 km"] = true,
        ["Frigate at 160 km"] = true, ["Frigate at 400 km"] = false },
}

-- Contacts that stand for several signals, per setup and hull: { signals = how many contacts }
local MERGED = {
    merging = { fighter = { [5] = 1 }, frigate = {}, capital = {} },
}

local failures = {}
local function check(ok, fmt, ...)
    if not ok then failures[#failures + 1] = string.format(fmt, ...) end
end

--- The player's readings now, by label: { tier, why, count }.
local function readingsByLabel(game)
    local sensor = game.playerShip:get(SensorComponents.Sensor)
    local byLabel = {}
    for _, o in ipairs(sensor:getObjects()) do
        local r = sensor:getReadings()[o]
        if r then byLabel[o.label] = { tier = r.tier, why = r.why, count = r.count, object = o } end
    end
    return byLabel
end

local function viewOf(game, label)
    local sensor = game.playerShip:get(SensorComponents.Sensor)
    for _, o in ipairs(sensor:getObjects()) do
        if o.label == label then return o end
    end
end

--- Whether each listener hears the player's ship now, by label.
local function heardBy(game)
    local result = {}
    for _, object in ipairs(game.scenarioData.objects) do
        local listener = object.entity:get(SensorComponents.Sensor)
        if listener then
            local heard = false
            for _, view in ipairs(listener:getObjects()) do
                local r = listener:getReadings()[view]
                if view.entity.id == game.playerShip.id and r and r.tier >= 1 then heard = true end
            end
            result[object.spec.label] = heard
        end
    end
    return result
end

local function checkMerged(game, setup, hull)
    local expected = MERGED[setup.id] and MERGED[setup.id][hull]
    if not expected then return end
    local found = {}
    for _, r in pairs(readingsByLabel(game)) do
        if r.tier >= 1 and r.count > 1 then found[r.count] = (found[r.count] or 0) + 1 end
    end
    for count, n in pairs(found) do
        check(expected[count] == n, "%s, %s: %d contacts of %d signals, expected %d", setup.name, hull, n, count,
            expected[count] or 0)
    end
    for count, n in pairs(expected) do
        check(found[count] == n, "%s, %s: %d contacts of %d signals, expected %d", setup.name, hull, found[count] or 0,
            count, n)
    end
end

local function checkExpectations(game, setup, hull)
    checkMerged(game, setup, hull)
    local expected = EXPECT[setup.id] and EXPECT[setup.id][hull]
    if not expected then return end
    local readings = readingsByLabel(game)
    for label, want in pairs(expected) do
        local wantTier, wantWhy = want, nil
        if type(want) == "string" then
            wantTier, wantWhy = want:match("^(%d)!(.+)$")
            wantTier = tonumber(wantTier)
        end
        local r = readings[label]
        if not r then
            failures[#failures + 1] = string.format("%s, %s: %s isn't among the sensor's objects", setup.name, hull, label)
        else
            check(r.tier == wantTier, "%s, %s: %s is T%d (%s), expected T%d", setup.name, hull, label, r.tier, r.why, wantTier)
            check(not wantWhy or r.why == wantWhy, "%s, %s: %s is '%s', expected '%s'", setup.name, hull, label, r.why,
                tostring(wantWhy))
        end
    end
end

-- The steps, run one per frame once their wait is over. A step returns
-- false to run again next frame.
local steps = {}
local function step(wait, fn) steps[#steps + 1] = { wait = wait, fn = fn } end

local scanHeld = false

local function setHull(game, hull)
    game.scenarioData.hull = hull
    game.playerShip:get(SensorComponents.Sensor):setDials(SensorModel.hulls[hull])
end

for index, setup in ipairs(Setups) do
    step(SETTLE_FRAMES, function(game)
        Log.Info('SensorSetupsTest: checking %s (%d/%d)', setup.name, index, #Setups)
        check(game.scenarioData.setupIndex == index, "%s: the testbed is on setup %s", setup.name,
            tostring(game.scenarioData.setupIndex))
    end)
    for _, hull in ipairs(SensorModel.hullOrder) do
        step(0, function(game) setHull(game, hull) end)
        step(SETTLE_FRAMES, function(game) checkExpectations(game, setup, hull) end)
    end

    if setup.id == "active" then
        step(0, function(game) setHull(game, "frigate") end)
        step(SETTLE_FRAMES, function(game) game.playerShip:get(SensorComponents.Sensor):requestPulse() end)
        step(SETTLE_FRAMES, function(game)
            local r = readingsByLabel(game)["Wreck"]
            check(r and r.tier >= 1, "%s: after a pulse the wreck is T%s (%s)", setup.name, r and r.tier, r and r.why)
            local sensor = game.playerShip:get(SensorComponents.Sensor)
            sensor:setTarget(viewOf(game, "Freighter"))
            scanHeld = true
            game.scenarioData.scanFrames = 0
        end)
        step(0, function(game) -- scanning the freighter until it reads its identity
            local sensor = game.playerShip:get(SensorComponents.Sensor)
            local data = game.scenarioData
            data.scanFrames = data.scanFrames + 1
            local read = sensor:getState().reads[viewOf(game, "Freighter")] or 0
            if read >= 3 or data.scanFrames > SCAN_FRAME_LIMIT then
                check(read >= 3, "%s: scanning the freighter read T%d in %d frames", setup.name, read, data.scanFrames)
                scanHeld = false
                return true
            end
            return false
        end)
    end

    if setup.id == "heard" then
        step(0, function(game) setHull(game, "frigate") end)
        step(SETTLE_FRAMES, function(game)
            local heard = heardBy(game)
            for label, want in pairs(HEARD.listening) do
                check(heard[label] == want, "%s, listening: %s %s you", setup.name, label, heard[label] and "hears" or "doesn't hear")
            end
            scanHeld = true
        end)
        step(SETTLE_FRAMES, function(game)
            local heard = heardBy(game)
            for label, want in pairs(HEARD.scanning) do
                check(heard[label] == want, "%s, scanning: %s %s you", setup.name, label, heard[label] and "hears" or "doesn't hear")
            end
            scanHeld = false
        end)
    end

    step(0, function(game) game:regenerateScenario() end)
end
step(SETTLE_FRAMES, function(game)
    check(game.scenarioData.setupIndex == 1, "after the last setup, Regenerate went to setup %s",
        tostring(game.scenarioData.setupIndex))
end)

local current, waited = 1, 0
local gameOnInit = Game.onInit
function Game:onInit()
    gameOnInit(self)
    -- The scenario sets the scan from the X key every frame; hold it here
    local scenarioInput = self.scenario.onInput
    self.scenario.onInput = function(game, dt)
        scenarioInput(game, dt)
        local sensor = game.playerShip:get(SensorComponents.Sensor)
        if sensor and scanHeld then sensor:setScanning(true) end
    end

    EventBus:subscribe(Event.PreRender, nil, function()
        local s = steps[current]
        if not s then return end
        if waited < s.wait then
            waited = waited + 1
            return
        end
        if s.fn(self) ~= false then
            current, waited = current + 1, 0
        end
        if current > #steps then
            if #failures > 0 then
                error("SensorSetupsTest: " .. #failures .. " failed\n" .. table.concat(failures, "\n"))
            end
            Log.Info('SensorSetupsTest: passed, %d setups', #Setups)
            Engine:exit()
        end
    end)
end

return Game
