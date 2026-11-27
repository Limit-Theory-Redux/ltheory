--[[
Sensor model test: checks Modules/Sensors/SensorModel against the numbers it
is tuned to and against reference readings of a fixed scene.

- The hull table: for a 20 m ship drawing 50 MW in open space, the distances
  at which each hull hears it (T1), classes it (T2), sees who it is (T3), reads
  it with a scan (T3, T4) and finds it with a pulse, and how far the hull's
  pulse and its own power draw are heard. The frigate gives the 1.0 spec's
  sensor table (T1 50 km, T2 10 km, T3 2 km, T4 200 m).
- The reference scene (a star, a planet, a station, a freighter, a fighter
  parked next to an asteroid, a second asteroid and a cold wreck): every
  object's tier, its reason and its SNR, for each hull in each surrounding,
  for both resolution rules, without merging, and with the fighter hot from
  firing or cloaked; and every object's scan and pulse echo per hull.
- Behaviour over time: a pulse makes the wreck in a nebula a contact, a scan
  reads its next tier after the scan time and heats the sensor, a ship heats
  up while it fires.

Run it from the repository root:

    cargo run -p ltr -- SensorModelTest

Exit code 0 and "SensorModelTest: passed" = passed. A failure stops the game
with its error and traceback and exit code 1 (`Core.Call`).
]]

local Application = require('States.Application')
local SensorModel = require('Modules.Sensors.SensorModel')
local SensorText = require('Modules.Sensors.SensorText')

---@class SensorModelTest: Application
local SensorModelTest = Subclass("SensorModelTest", Application)

local DEG = math.pi / 180

-- Reference distances: hears it, class, sees who, scan T3, scan T4, pulse
-- finds it, pulse heard, you're heard.
local HULL_TABLE = {
    fighter = { "35 km", "2.3 km", "458 m", "632 m", "63 m", "28 km", "100 km", "22 km" },
    frigate = { "50 km", "10.0 km", "2.0 km", "2.0 km", "200 m", "50 km", "224 km", "50 km" },
    capital = { "100 km", "57 km", "6.3 km", "8.9 km", "894 m", "106 km", "500 km", "158 km" },
}

-- The reference scene, top down: distance (m) and bearing (degrees) from the
-- sensor, in its plane.
local function referenceScene()
    local function at(dist, bearing)
        return { x = dist * math.sin(bearing * DEG), y = 0, z = dist * math.cos(bearing * DEG) }
    end
    return {
        { kind = "star", label = "Star", pos = at(6e7, 38), radius = 1e6, fixedEm = 1e7, fixedHeat = 1e10, glow = 3e10 },
        { kind = "planet", label = "Planet", pos = at(1.5e5, 3), radius = 5e4, fixedHeat = 50 },
        { kind = "station", label = "Station", pos = at(1.2e4, 150), radius = 316, mw = 500 },
        { kind = "ship", label = "Freighter", pos = at(4e3, -28), radius = 20, mw = 50, activity = "idle", hot = 0 },
        { kind = "ship", label = "Fighter", pos = at(1.06e4, 73.5), radius = 8, mw = 8, activity = "idle", hot = 0 },
        { kind = "asteroid", label = "Asteroid", pos = at(6e3, 64), radius = 500 },
        { kind = "asteroid", label = "Asteroid", pos = at(1.1e4, 71), radius = 500 },
        { kind = "wreck", label = "Wreck", pos = at(400, 4), radius = 30 },
    }
end

-- Reference readings of the scene, object by object: tier (x merged
-- signals), reason, passive SNR.
local SCENE_READINGS = {
    {
        "fighter", "open", nil,
        "Star=2 (light, 1.9 cells, snr 2.08e+3); Planet=3 (seen, 38 cells, snr 833); Station=2 (EM, 3.0 cells, snr 86.8); Freighter=1 (EM, unresolved, snr 78.1); Fighter=0 (lost in the background, snr 0.0273); Asteroid=3 (seen, 9.5 cells, snr 17.4); Asteroid=3 (seen, 5.2 cells, snr 5.17); Wreck=3 (seen, 8.6 cells, snr 61.0)",
    },
    {
        "fighter", "nebula", nil,
        "Star=0 (too faint, snr 3.38e-10); Planet=0 (too faint, snr 0.00); Station=2 (EM, 3.0 cells, snr 1.04); Freighter=1 (EM, unresolved, snr 7.71); Fighter=0 (lost in the background, snr 0.0138); Asteroid=0 (too faint, snr 0.00); Asteroid=0 (too faint, snr 0.00); Wreck=0 (too faint, snr 0.00)",
    },
    {
        "fighter", "debris", nil,
        "Star=0 (lost in the background, snr 0.00000101); Planet=0 (lost in the background, snr 0.0648); Station=2 (EM, 3.0 cells, snr 1.04); Freighter=1 (EM, unresolved, snr 7.71); Fighter=0 (lost in the background, snr 0.0138); Asteroid=0 (lost in the background, snr 0.805); Asteroid=0 (lost in the background, snr 0.0737); Wreck=3 (seen, 8.6 cells, snr 56.5)",
    },
    {
        "fighter", "radiation", nil,
        "Star=0 (lost in the background, snr 3.04e-7); Planet=0 (lost in the background, snr 0.0195); Station=0 (lost in the background, snr 0.315); Freighter=1 (EM, unresolved, snr 2.48); Fighter=0 (lost in the background, snr 0.00638); Asteroid=0 (lost in the background, snr 0.250); Asteroid=0 (lost in the background, snr 0.0223); Wreck=1 (radiation: only fuzzy, snr 48.3)",
    },
    {
        "frigate", "open", nil,
        "Star=3 (seen, 8.3 cells, snr 4.17e+3); Planet=3 (seen, 166 cells, snr 1.67e+3); Station=3 (seen, 13 cells, snr 174); Freighter=2 (EM, 2.5 cells, snr 156); Fighter=0 (lost in the background, snr 0.810); Asteroid=3 (seen, 42 cells, snr 34.7); Asteroid=3 (seen, 23 cells, snr 10.3); Wreck=3 (seen, 37 cells, snr 140)",
    },
    {
        "frigate", "nebula", nil,
        "Star=0 (lost in the background, snr 0.0000128); Planet=0 (lost in the background, snr 0.00163); Station=2 (EM, 13 cells, snr 32.4); Freighter=2 (EM, 2.5 cells, snr 105); Fighter=0 (lost in the background, snr 0.457); Asteroid=0 (too faint, snr 0.00); Asteroid=0 (too faint, snr 0.00); Wreck=0 (too faint, snr 0.00)",
    },
    {
        "frigate", "debris", nil,
        "Star=0 (lost in the background, snr 0.0000383); Planet=3 (seen, 166 cells, snr 2.45); Station=3 (seen, 13 cells, snr 32.4); Freighter=2 (EM, 2.5 cells, snr 105); Fighter=0 (lost in the background, snr 0.457); Asteroid=3 (seen, 42 cells, snr 16.6); Asteroid=3 (seen, 23 cells, snr 2.22); Wreck=3 (seen, 37 cells, snr 139)",
    },
    {
        "frigate", "radiation", nil,
        "Star=0 (lost in the background, snr 0.0000115); Planet=0 (lost in the background, snr 0.735); Station=1 (radiation: only fuzzy, snr 11.2); Freighter=1 (radiation: only fuzzy, snr 59.8); Fighter=0 (lost in the background, snr 0.227); Asteroid=1 (radiation: only fuzzy, snr 7.51); Asteroid=0 (lost in the background, snr 0.783); Wreck=1 (radiation: only fuzzy, snr 137)",
    },
    {
        "capital", "open", nil,
        "Star=3 (seen, 48 cells, snr 1.67e+4); Planet=3 (seen, 955 cells, snr 6.67e+3); Station=3 (seen, 75 cells, snr 694); Freighter=3 (seen, 14 cells, snr 625); Fighter=2 (EM, 2.2 cells, snr 12.9); Asteroid=3 (seen, 239 cells, snr 139); Asteroid=3 (seen, 130 cells, snr 41.3); Wreck=3 (seen, 215 cells, snr 562)",
    },
    {
        "capital", "nebula", nil,
        "Star=0 (lost in the background, snr 0.00169); Planet=0 (lost in the background, snr 0.206); Station=2 (EM, 75 cells, snr 614); Freighter=2 (EM, 14 cells, snr 616); Fighter=2 (EM, 2.2 cells, snr 11.8); Asteroid=0 (too faint, snr 0.00); Asteroid=0 (too faint, snr 0.00); Wreck=0 (too faint, snr 0.00)",
    },
    {
        "capital", "debris", nil,
        "Star=0 (lost in the background, snr 0.00507); Planet=3 (seen, 955 cells, snr 309); Station=3 (seen, 75 cells, snr 614); Freighter=3 (seen, 14 cells, snr 616); Fighter=2 (EM, 2.2 cells, snr 11.8); Asteroid=3 (seen, 239 cells, snr 134); Asteroid=3 (seen, 130 cells, snr 37.2); Wreck=3 (seen, 215 cells, snr 562)",
    },
    {
        "capital", "radiation", nil,
        "Star=0 (lost in the background, snr 0.00152); Planet=1 (radiation: only fuzzy, snr 95.9); Station=1 (radiation: only fuzzy, snr 483); Freighter=1 (radiation: only fuzzy, snr 596); Fighter=1 (radiation: only fuzzy, snr 9.85); Asteroid=1 (radiation: only fuzzy, snr 125); Asteroid=1 (radiation: only fuzzy, snr 30.2); Wreck=1 (radiation: only fuzzy, snr 562)",
    },
    {
        "frigate", "open", { resolutionRule = "dim" },
        "Star=3 (seen, 8.3 cells, snr 4.17e+3); Planet=3 (seen, 166 cells, snr 1.67e+3); Station=3 (seen, 13 cells, snr 174); Freighter=2 (EM, 2.5 cells, snr 156); Fighter=1 (EM, unresolved, snr 1.34); Asteroid=3 (seen, 42 cells, snr 34.7); Asteroid=3 (seen, 23 cells, snr 10.3); Wreck=3 (seen, 37 cells, snr 141)",
    },
    {
        "fighter", "open", { resolutionRule = "dim" },
        "Star=2 (light, 1.9 cells, snr 2.08e+3); Planet=3 (seen, 38 cells, snr 833); Station=2 (EM, 3.0 cells, snr 86.8); Freighter=1 (EM, unresolved, snr 44.8); Fighter=0 (too faint, snr 0.154); Asteroid=3 (seen, 9.5 cells, snr 17.4); Asteroid=3 (seen, 5.2 cells, snr 5.17); Wreck=3 (seen, 8.6 cells, snr 70.3)",
    },
    {
        "frigate", "open", { merge = false },
        "Star=3 (seen, 8.3 cells, snr 4.17e+3); Planet=3 (seen, 166 cells, snr 1.67e+3); Station=3 (seen, 13 cells, snr 174); Freighter=2 (EM, 2.5 cells, snr 156); Fighter=0 (lost in the background, snr 0.810); Asteroid=3 (seen, 42 cells, snr 34.7); Asteroid=3 (seen, 23 cells, snr 10.3); Wreck=3 (seen, 37 cells, snr 140)",
    },
    {
        "capital", "open", { fighter = { activity = "cloak", hot = 1 } },
        "Star=3 (seen, 48 cells, snr 1.67e+4); Planet=3 (seen, 955 cells, snr 6.67e+3); Station=3 (seen, 75 cells, snr 694); Freighter=3 (seen, 14 cells, snr 625); Fighter=2 (heat, 2.2 cells, snr 25.8); Asteroid=3 (seen, 239 cells, snr 139); Asteroid=3 (seen, 130 cells, snr 41.3); Wreck=3 (seen, 215 cells, snr 562)",
    },
    {
        "frigate", "open", { fighter = { activity = "firing", hot = 1 } },
        "Star=3 (seen, 8.3 cells, snr 4.17e+3); Planet=3 (seen, 166 cells, snr 1.67e+3); Station=3 (seen, 13 cells, snr 174); Freighter=2 (EM, 2.5 cells, snr 156); Fighter=1 (heat, unresolved, snr 1.62); Asteroid=3 (seen, 42 cells, snr 34.7); Asteroid=3 (seen, 23 cells, snr 10.3); Wreck=3 (seen, 37 cells, snr 140)",
    },
}

-- Reference scan level (echo SNR) and pulse echo SNR per object, in open space.
local ECHOES = {
    fighter = "Star scan=0 (0.00) pulse=0.00; Planet scan=0 (0.0107) pulse=4.63e+3; Station scan=0 (3.33) pulse=9.03e+3; Freighter scan=0 (8.21) pulse=2.44e+3; Fighter scan=0 (0.000409) pulse=0.122; Asteroid scan=0 (2.22) pulse=6.03e+4; Asteroid scan=0 (0.661) pulse=5.34e+3; Wreck scan=3 (2.17e+3) pulse=4.77e+7",
    frigate = "Star scan=0 (0.00) pulse=0.00; Planet scan=0 (0.107) pulse=4.63e+4; Station scan=0 (33.3) pulse=9.03e+4; Freighter scan=0 (250) pulse=2.44e+4; Fighter scan=0 (1.15) pulse=18.0; Asteroid scan=0 (22.2) pulse=6.03e+5; Asteroid scan=0 (6.61) pulse=5.34e+4; Wreck scan=3 (2.48e+4) pulse=5.45e+8",
    capital = "Star scan=0 (0.00) pulse=0.00; Planet scan=0 (2.13) pulse=9.26e+5; Station scan=0 (667) pulse=1.81e+6; Freighter scan=3 (5.00e+3) pulse=4.88e+5; Fighter scan=0 (646) pulse=1.44e+3; Asteroid scan=0 (444) pulse=1.21e+7; Asteroid scan=0 (132) pulse=1.07e+6; Wreck scan=4 (5.00e+5) pulse=1.10e+10",
}

local failures = {}

local function check(ok, fmt, ...)
    if not ok then failures[#failures + 1] = string.format(fmt, ...) end
end

-- Reference values have three significant figures; "0.00" is exactly zero.
local function close(actual, reference)
    if reference == 0 then return math.abs(actual) < 1e-12 end
    return math.abs(actual - reference) <= 0.006 * math.abs(reference)
end

local function split(line)
    local parts = {}
    for part in (line .. "; "):gmatch("(.-); ") do parts[#parts + 1] = part end
    return parts
end

local function newState(hull, surroundings)
    local state = SensorModel.newState(SensorModel.copyDials(SensorModel.hulls[hull]))
    state.objects = referenceScene()
    state.surroundings = surroundings or "open"
    return state
end

local function checkHullTable()
    local reference = { kind = "ship", label = "Reference", pos = { x = 0, y = 0, z = 1e4 }, radius = 20, mw = 50, activity = "idle", hot = 0 }
    for _, hull in ipairs(SensorModel.hullOrder) do
        local dials = SensorModel.hulls[hull]
        local state = SensorModel.newState(dials)
        local g = SensorModel.ranges(state, reference)
        local row = { g.detect, g.classify, g.identify, g.scan3, g.scan4, g.pulse,
            SensorModel.heardWithin(dials.power), SensorModel.heardWithin(dials.ownMw) }
        for i, d in ipairs(row) do
            local got, want = SensorText.distance(d), HULL_TABLE[hull][i]
            check(got == want, "hull table, %s column %d: %s, expected %s", hull, i, got, want)
        end
    end
end

local function checkScene()
    for _, case in ipairs(SCENE_READINGS) do
        local hull, surroundings, options, line = case[1], case[2], case[3] or {}, case[4]
        local state = newState(hull, surroundings)
        if options.resolutionRule then state.resolutionRule = options.resolutionRule end
        if options.merge ~= nil then state.merge = options.merge end
        if options.fighter then
            for key, value in pairs(options.fighter) do state.objects[5][key] = value end
        end
        local results = SensorModel.evaluate(state)
        local name = string.format("%s in %s%s", hull, surroundings,
            options.resolutionRule and (", " .. options.resolutionRule) or
            (options.merge == false and ", no merging") or
            (options.fighter and (", fighter " .. options.fighter.activity) or ""))
        for i, entry in ipairs(split(line)) do
            local label, tier, count, why, snr = entry:match("^(%a+)=(%d)x?(%d*) %((.*), snr ([^%)]+)%)$")
            local o = state.objects[i]
            local r = results[o]
            check(o.label == label, "%s: object %d is %s, expected %s", name, i, o.label, tostring(label))
            check(r.tier == tonumber(tier), "%s, %s: T%d, expected T%s", name, label, r.tier, tier)
            check(r.count == (tonumber(count) or 1), "%s, %s: %d signals, expected %s", name, label, r.count, count)
            check(r.why == why, "%s, %s: '%s', expected '%s'", name, label, r.why, why)
            check(close(r.passive.snr, tonumber(snr)), "%s, %s: SNR %g, expected %s", name, label, r.passive.snr, snr)
        end
    end
end

local function checkEchoes()
    for _, hull in ipairs(SensorModel.hullOrder) do
        local state = newState(hull)
        for i, entry in ipairs(split(ECHOES[hull])) do
            local label, level, scanSnr, pulseSnr = entry:match("^(%a+) scan=(%d) %(([^%)]+)%) pulse=(.+)$")
            local o = state.objects[i]
            local p = SensorModel.passive(state, o)
            local scan = SensorModel.scanEcho(state, p)
            local pulse = SensorModel.pulseEcho(state, o, p)
            check(scan.level == tonumber(level), "%s, %s: scan reads %d, expected %s", hull, label, scan.level, level)
            check(close(scan.snr, tonumber(scanSnr)), "%s, %s: scan SNR %g, expected %s", hull, label, scan.snr, scanSnr)
            check(close(pulse.snr, tonumber(pulseSnr)), "%s, %s: pulse SNR %g, expected %s", hull, label, pulse.snr, pulseSnr)
        end
    end
end

local function checkOverTime()
    -- In a nebula the frigate can't hear the cold wreck; a pulse finds it
    local state = newState("frigate", "nebula")
    local wreck = state.objects[8]
    check(SensorModel.evaluate(state)[wreck].tier == 0, "nebula: the wreck shows before a pulse")
    SensorModel.firePulse(state)
    local r = SensorModel.evaluate(state)[wreck]
    check(r.tier == 2 and r.why == "pulse echo, 37 cells", "nebula: after a pulse the wreck is T%d '%s', expected T2 'pulse echo, 37 cells'", r.tier, r.why)
    check(state.pulseHeardBy == 3, "nebula: %d ships and stations heard the pulse, expected 3", state.pulseHeardBy)
    -- The echo fades after its time
    local steps = math.ceil(SensorModel.constants.pulseEchoTime / 0.5) + 1
    for _ = 1, steps do SensorModel.step(state, SensorModel.evaluate(state), 0.5) end
    check(SensorModel.evaluate(state)[wreck].tier == 0, "nebula: the pulse echo didn't fade")

    -- The capital ship's scan reads the wreck's T4 after one scan time, and heats it up
    state = newState("capital")
    wreck = state.objects[8]
    state.target, state.scanning = wreck, true
    local dt, t = 0.1, 0
    while (state.reads[wreck] or 0) < 4 and t < 10 do
        SensorModel.step(state, SensorModel.evaluate(state), dt)
        t = t + dt
    end
    check(state.reads[wreck] == 4, "scan: read T%d, expected T4", state.reads[wreck] or 0)
    check(math.abs(t - state.dials.scanTime) < 2 * dt, "scan: took %.1f s, expected %.1f s", t, state.dials.scanTime)
    r = SensorModel.evaluate(state)[wreck]
    check(r.tier == 4 and r.why == "full scan read", "scan: the wreck shows T%d '%s'", r.tier, r.why)
    check(close(state.heat, SensorModel.constants.heatUp * t), "scan: heat %.3f after %.1f s", state.heat, t)

    -- A ship firing heats up, and cools down after
    local fighter = { kind = "ship", label = "Fighter", pos = { x = 0, y = 0, z = 1e4 }, radius = 8, mw = 8, activity = "firing", hot = 0 }
    for _ = 1, 12 do SensorModel.stepObjectHeat(fighter, 0.5) end
    check(fighter.hot == 1, "heat: a firing ship at %.2f after 6 s, expected 1", fighter.hot)
    fighter.activity = "idle"
    SensorModel.stepObjectHeat(fighter, 2)
    check(close(fighter.hot, 0.8), "heat: an idle ship at %.2f after cooling 2 s, expected 0.8", fighter.hot)
end

function SensorModelTest:onInit()
    checkHullTable()
    checkScene()
    checkEchoes()
    checkOverTime()
    if #failures > 0 then
        error("SensorModelTest: " .. #failures .. " failed\n" .. table.concat(failures, "\n"))
    end
    Log.Info('SensorModelTest: passed')
    self:quit()
end

return SensorModelTest
