-- The sensor model: what a ship's sensors make of the objects around it.
--
-- One sensor system with two modes. Passive listening is all round and always
-- on: an object is heard (T1) once its strongest signal beats the noise,
-- classed (T2) once it fills one resolution cell, and identified by sight (T3)
-- once it fills five cells and its reflected light alone beats the noise.
-- Active sensing shouts: the aimed scan reads T3 and then T4 from its echo,
-- and the all-round pulse finds whatever echoes back strongly enough. Both are
-- heard by everyone in range.
--
-- An object comes down to three facts: what it gives off (EM from its power
-- draw, and heat), what it reflects (light, and a share of a scan or a pulse)
-- and its size. A sensor has a few dials: sensitivity, resolution, a heat
-- band, power, scan time, and the ship's own power draw.
--
-- Units: metres, megawatts, seconds, degrees. Strengths are given at 10 km and
-- fall off with distance squared; a signal counts once signal / noise (SNR)
-- reaches 1. Positions are relative to the sensor. The model works on plain
-- tables and holds no engine types, so it runs without a scene.
--
-- The constants are tuned so a frigate against a 20 m ship drawing 50 MW, in
-- open space, gives T1 at 50 km, T2 at 10 km, T3 at 2 km and T4 at 200 m (the
-- 1.0 spec's sensor table). The other values are placeholders to tune.

local SensorModel = {}

local DEG = math.pi / 180

---@class SensorConstants
local M = {
    noise = 0.2,            -- the noise floor: a signal is heard once it beats this (SNR 1)
    reference = 1e4,        -- strengths are given at this distance (10 km)
    sizeReference = 20,     -- light, pulse echoes and background are reckoned per 20 m
    emPerMw = 0.1,          -- EM given off = 0.1 x power draw
    heatPerMw = 0.05,       -- heat given off = 0.05 x power draw x (1 + 3 x how hot it runs)
    hotHeat = 3,
    light = 0.04,           -- light reflected = 0.04 x reflects x (size / 20 m)^2
    classCells = 1,         -- T2: the object fills one resolution cell
    identifyCells = 5,      -- T3: it fills five, and its light alone beats the noise
    scanFactor = 0.016,
    scanT3Snr = 1000,       -- scan echo SNR that reads T3
    scanT4Snr = 1e5,        -- and T4
    pulseFactor = 0.25,
    pulseEchoTime = 20,     -- seconds a pulse echo stays on the contacts
    heatUp = 1 / 8,         -- own heat per second while scanning
    heatDown = 1 / 20,      -- and cooling per second after
    bloom = 0.6,            -- own heat from which scanning shows as a thermal bloom
    shipHeatUp = 0.2,       -- a ship's heat per second while thrusting, firing or cloaked
    shipHeatDown = 0.1,     -- and cooling per second otherwise
    cloakReflects = 0.1,    -- a cloak keeps this share of what a ship reflects, and hides its light
    glare = 8,              -- degrees around a star where its glare hides heat and light
    glareMinDistance = 2000,
    rockDistance = 1000,    -- closer than this to an asteroid, a target shares its background
    -- Background per resolution cell, per 20 m of cell (the suggested resolution rule)
    background = { nebula = 3, debris = 3, radiation = 10, rock = 3, planet = 5 },
}
SensorModel.constants = M

--- Kinds of object, and how much of a scan, a pulse or starlight they reflect.
SensorModel.kinds = {
    star = { name = "Star", reflects = 0 },
    planet = { name = "Planet", reflects = 0.3 },
    station = { name = "Station", reflects = 0.6 },
    ship = { name = "Ship", reflects = 0.5 },
    asteroid = { name = "Asteroid", reflects = 0.1 },
    wreck = { name = "Wreck", reflects = 0.5 },
}

---@class SensorDials
---@field sensitivity number how faint a signal it still hears (x4 hears twice as far)
---@field resolution number degrees per resolution cell
---@field heatBand boolean hears heat as well as power draw
---@field power number how hard the active modes shout
---@field scanTime number seconds to read one tier
---@field ownMw number the ship's own power draw, MW: others hear it as EM

--- Sensor presets per hull: a bigger hull carries a bigger array, so finer
--- resolution, more power and more sensitivity, but it is louder too.
---@type table<string, SensorDials>
SensorModel.hulls = {
    fighter = { sensitivity = 0.5, resolution = 0.5, heatBand = false, power = 200, scanTime = 5, ownMw = 10 },
    frigate = { sensitivity = 1, resolution = 0.115, heatBand = true, power = 1000, scanTime = 3, ownMw = 50 },
    capital = { sensitivity = 4, resolution = 0.02, heatBand = true, power = 5000, scanTime = 4, ownMw = 500 },
}
SensorModel.hullOrder = { "fighter", "frigate", "capital" }
SensorModel.hullNames = { fighter = "Fighter", frigate = "Frigate", capital = "Capital ship" }

--- What surrounds the sensor: open space, or something that smears signals.
SensorModel.surroundingsOrder = { "open", "nebula", "debris", "radiation" }
SensorModel.surroundingsNames = { open = "Open space", nebula = "Nebula", debris = "Debris field", radiation = "Radiation belt" }

--- Resolution rules. "background": resolution decides how much background
--- shares the target's cell (a coarse sensor loses small ships in clutter
--- only). "dim": only min(1, cells) of the signal counts (a coarse sensor
--- loses small ships even in empty space).
SensorModel.resolutionRules = { "background", "dim" }

SensorModel.activities = { "idle", "thrust", "firing", "cloak" }
SensorModel.activityNames = { idle = "Idle", thrust = "Thrusting", firing = "Firing", cloak = "Cloaked" }

---@class SensorVec
---@field x number
---@field y number
---@field z number

---@class SensorObject
---@field kind string a key of `SensorModel.kinds`
---@field label string
---@field pos SensorVec metres from the sensor
---@field radius number metres
---@field mw number|nil power draw, MW
---@field activity string|nil ships: idle, thrust, firing or cloak
---@field hot number|nil ships: how hot it runs, 0 to 1 (see `SensorModel.stepObjectHeat`)
---@field fixedEm number|nil EM given off, instead of its power draw's
---@field fixedHeat number|nil heat given off, instead of its power draw's
---@field glow number|nil light of its own (a star)
---@field name string|nil what a T3 read shows, "[Manufacturer] [Model] (Year)" for vessels
---@field details string|nil a second T3 line (faction, condition ...)
---@field shield number|nil its shields, 0 to 1, as a T3 read shows them
---@field standing string|nil hostile, neutral or friendly, as a T3 read shows it
---@field contents string[]|nil what a T4 read shows (cargo, logs ...)

---@class SensorState
---@field dials SensorDials
---@field objects SensorObject[] everything it could sense, positions relative to it
---@field surroundings string a key of `SensorModel.surroundingsNames`
---@field resolutionRule string "background" or "dim"
---@field merge boolean unresolved signals in one cell show as one blob
---@field echoes table<SensorObject, number> seconds of pulse echo left
---@field reads table<SensorObject, number> the best tier a scan has read
---@field progress table<SensorObject, number> the running scan's progress, 0 to 1
---@field target SensorObject|nil what the scan aims at
---@field scanning boolean
---@field heat number own heat from scanning, 0 to 1
---@field pulseAge number seconds since the last pulse
---@field pulseHeardBy number ships and stations within earshot of the last pulse

--- A sensor with these dials and nothing around it yet.
---@param dials SensorDials
---@return SensorState
function SensorModel.newState(dials)
    return {
        dials = dials,
        objects = {},
        surroundings = "open",
        resolutionRule = "background",
        merge = true,
        echoes = {},
        reads = {},
        progress = {},
        target = nil,
        scanning = false,
        heat = 0,
        pulseAge = math.huge,
        pulseHeardBy = 0,
    }
end

--- A copy of a dials table.
---@param dials SensorDials
---@return SensorDials
function SensorModel.copyDials(dials)
    local copy = {}
    for key, value in pairs(dials) do copy[key] = value end
    return copy
end

local function clamp(x, lo, hi)
    return math.min(hi, math.max(lo, x))
end

local function length(v)
    return math.sqrt(v.x * v.x + v.y * v.y + v.z * v.z)
end

--- The angle between two directions from the sensor, in radians. atan2 of
--- the cross and dot products stays exact for small angles, where acos does not.
local function angleBetween(a, b)
    local cx = a.y * b.z - a.z * b.y
    local cy = a.z * b.x - a.x * b.z
    local cz = a.x * b.y - a.y * b.x
    return math.atan2(math.sqrt(cx * cx + cy * cy + cz * cz), a.x * b.x + a.y * b.y + a.z * b.z)
end

local function separation(a, b)
    local dx, dy, dz = a.pos.x - b.pos.x, a.pos.y - b.pos.y, a.pos.z - b.pos.z
    return math.sqrt(dx * dx + dy * dy + dz * dz)
end

--- The distance from the sensor to an object, in metres.
---@param o SensorObject
---@return number
function SensorModel.distance(o)
    return length(o.pos)
end

--- The angle between two objects as the sensor sees them, in degrees.
---@param a SensorObject
---@param b SensorObject
---@return number
function SensorModel.angleBetween(a, b)
    return angleBetween(a.pos, b.pos) / DEG
end

--- A number of cells as the model's reasons show it: one decimal below 10.
---@param cells number
---@return string
function SensorModel.formatCells(cells)
    if cells < 10 then return string.format("%.1f", cells) end
    return tostring(math.floor(cells + 0.5))
end

--- Size classes: I support and drones, II industrial, III capital, IV station;
--- 0 for anything that is not a vessel.
---@param o SensorObject
---@return integer
function SensorModel.shipClass(o)
    if o.kind == "station" then return 4 end
    if o.kind == "wreck" then return 2 end
    if o.kind ~= "ship" then return 0 end
    if o.radius < 10 then return 1 end
    if o.radius < 60 then return 2 end
    return 3
end

---@class SensorEmissions
---@field em number
---@field heat number
---@field light number reflected light, plus its own glow
---@field reflects number the share of a scan or pulse it sends back
---@field cloaked boolean

--- What an object gives off and reflects.
---@param o SensorObject
---@return SensorEmissions
function SensorModel.giveOff(o)
    local cloaked = o.kind == "ship" and o.activity == "cloak"
    local mw = o.mw or 0
    local kindReflects = SensorModel.kinds[o.kind].reflects
    local em = o.fixedEm or M.emPerMw * mw
    local heat = o.fixedHeat or M.heatPerMw * mw * (1 + M.hotHeat * (o.hot or 0))
    local lit = cloaked and 0 or M.light * kindReflects * (o.radius / M.sizeReference) ^ 2
    return {
        em = em,
        heat = heat,
        light = (o.glow or 0) + lit,
        reflects = kindReflects * (cloaked and M.cloakReflects or 1),
        cloaked = cloaked,
    }
end

---@class SensorSurroundings
---@field em number what reaches the sensor of each channel, 0 or 1
---@field heat number
---@field light number
---@field by string what blocks a channel, "" for nothing
---@field blocked boolean a planet is in the way: nothing gets through
---@field background number background in the target's cell, per 20 m of cell
---@field backgroundWhy string[]
---@field cap integer the highest tier anything here can reach

local OPEN = { em = 1, heat = 1, light = 1, by = "", blocked = false, background = 0, backgroundWhy = {}, cap = 4 }

--- What is in the way of an object: planets block everything behind them,
--- asteroids block EM and heat, a star's glare hides heat and light, nebulae
--- block light. Nebulae, debris fields, radiation belts, a rock next to the
--- target and a planet behind it add background to its cell. Radiation belts
--- cap everything at T1.
---@param state SensorState
---@param o SensorObject
---@return SensorSurroundings
function SensorModel.surroundings(state, o)
    local e = { em = 1, heat = 1, light = 1, by = "", blocked = false, background = 0, backgroundWhy = {}, cap = 4 }
    local d = length(o.pos)
    local star
    for _, b in ipairs(state.objects) do
        if b ~= o and (b.kind == "planet" or b.kind == "asteroid") then
            local bd = length(b.pos)
            if bd < d then
                -- the closest approach of the line of sight to b's centre
                local a = angleBetween(o.pos, b.pos)
                if a < math.pi / 2 and bd * math.sin(a) <= b.radius then
                    if b.kind == "planet" then
                        return { em = 0, heat = 0, light = 0, by = "behind the planet", blocked = true,
                            background = 0, backgroundWhy = {}, cap = 0 }
                    end
                    e.em, e.heat = 0, 0
                    e.by = "an asteroid blocks EM and heat"
                end
            end
        end
        if not star and b.kind == "star" then star = b end
    end
    if star and o ~= star and d > M.glareMinDistance and angleBetween(o.pos, star.pos) < M.glare * DEG then
        e.heat, e.light = 0, 0
        if e.by == "" then e.by = "the star's glare hides heat and light" end
    end

    local bg = M.background
    if state.surroundings == "nebula" then
        e.light = 0
        e.background = e.background + bg.nebula
        e.backgroundWhy[#e.backgroundWhy + 1] = "nebula " .. bg.nebula
    elseif state.surroundings == "debris" then
        e.background = e.background + bg.debris
        e.backgroundWhy[#e.backgroundWhy + 1] = "debris " .. bg.debris
    elseif state.surroundings == "radiation" then
        e.background = e.background + bg.radiation
        e.backgroundWhy[#e.backgroundWhy + 1] = "radiation " .. bg.radiation
        e.cap = 1
    end
    for _, b in ipairs(state.objects) do
        if b ~= o then
            if b.kind == "asteroid" and o.kind ~= "asteroid" and separation(o, b) < M.rockDistance then
                e.background = e.background + bg.rock
                e.backgroundWhy[#e.backgroundWhy + 1] = "next to a rock " .. bg.rock
            end
            if b.kind == "planet" then
                local bd = length(b.pos)
                if bd > d and angleBetween(o.pos, b.pos) < math.asin(math.min(1, b.radius / bd)) then
                    e.background = e.background + bg.planet
                    e.backgroundWhy[#e.backgroundWhy + 1] = "planet behind " .. bg.planet
                end
            end
        end
    end
    return e
end

---@class SensorPassive
---@field emissions SensorEmissions
---@field surroundings SensorSurroundings
---@field distance number metres (at least 1)
---@field cell number metres across one resolution cell at that distance
---@field cells number how many cells the object fills
---@field near number (10 km / distance)^2
---@field channels { EM: number, heat: number, light: number } what reaches the sensor, at 10 km
---@field via string the strongest channel heard: "EM", "heat" or "light"
---@field best number its strength
---@field signal number
---@field noise number
---@field snr number
---@field lightSnr number SNR of the light alone
---@field dimmed number the share of the signal that counts ("dim" rule)
---@field backgroundFactor number how much the background raises the noise ("background" rule)

--- What the sensor hears of an object without doing anything.
---@param state SensorState
---@param o SensorObject
---@param dials SensorDials|nil default: the state's
---@param open boolean|nil as if in open space, with nothing around it
---@return SensorPassive
function SensorModel.passive(state, o, dials, open)
    dials = dials or state.dials
    local g = SensorModel.giveOff(o)
    local e = open and OPEN or SensorModel.surroundings(state, o)
    local d = math.max(1, length(o.pos))
    local cell = d * dials.resolution * DEG
    local cells = o.radius / cell
    local near = (M.reference / d) ^ 2
    local channels = {
        EM = g.em * e.em,
        heat = dials.heatBand and g.heat * e.heat or 0,
        light = g.light * e.light,
    }
    local via, best = "EM", channels.EM
    if channels.heat > best then via, best = "heat", channels.heat end
    if channels.light > best then via, best = "light", channels.light end

    local noise, dimmed, backgroundFactor = M.noise, 1, 1
    if state.resolutionRule == "dim" then
        dimmed = math.min(1, cells)
    else
        backgroundFactor = 1 + e.background * (cell / M.sizeReference) ^ 2
        noise = noise * backgroundFactor
    end
    local signal = dials.sensitivity * best * near * dimmed
    return {
        emissions = g,
        surroundings = e,
        distance = d,
        cell = cell,
        cells = cells,
        near = near,
        channels = channels,
        via = via,
        best = best,
        signal = signal,
        noise = noise,
        snr = signal / noise,
        lightSnr = dials.sensitivity * channels.light * near * dimmed / noise,
        dimmed = dimmed,
        backgroundFactor = backgroundFactor,
    }
end

---@class SensorScanEcho
---@field fit number the share of the beam that lands on the target
---@field echo number
---@field snr number
---@field level integer the tier it reads: 4, 3, or 0 for too weak

--- The aimed scan: a beam with the sensor's own angular width. All of it lands
--- while the target fills the beam, so the echo falls with distance squared;
--- before that, with distance to the fourth. A scan is radar, so whatever
--- blocks EM blocks it too.
---@param state SensorState
---@param p SensorPassive the target's passive reading
---@param dials SensorDials|nil default: the state's
---@return SensorScanEcho
function SensorModel.scanEcho(state, p, dials)
    dials = dials or state.dials
    local fit = math.min(1, p.cells) ^ 2
    local echo = M.scanFactor * dials.power * p.emissions.reflects * dials.sensitivity * fit
        * (M.reference / p.distance) ^ 2 * p.surroundings.em
    local snr = echo / p.noise
    local level = 0
    if snr >= M.scanT4Snr then level = 4 elseif snr >= M.scanT3Snr then level = 3 end
    return { fit = fit, echo = echo, snr = snr, level = level }
end

--- The all-round pulse: spread over every direction, so its echo falls with
--- distance to the fourth from the start.
---@param state SensorState
---@param o SensorObject
---@param p SensorPassive
---@param dials SensorDials|nil default: the state's
---@return { echo: number, snr: number }
function SensorModel.pulseEcho(state, o, p, dials)
    dials = dials or state.dials
    local echo = M.pulseFactor * dials.power * p.emissions.reflects * (o.radius / M.sizeReference) ^ 2
        * dials.sensitivity * (M.reference / p.distance) ^ 4 * p.surroundings.em * p.dimmed
    return { echo = echo, snr = echo / p.noise }
end

--- How far a sensor of sensitivity 1 hears a strength, in open space.
---@param strength number at 10 km
---@return number metres
function SensorModel.heardAt(strength)
    return M.reference * math.sqrt(strength / M.noise)
end

--- How far others hear a power draw as EM (a ship listening, or the power of
--- a scan or a pulse).
---@param mw number
---@return number metres
function SensorModel.heardWithin(mw)
    return SensorModel.heardAt(M.emPerMw * mw)
end

---@class SensorResult
---@field tier integer 0 (not seen) to 4
---@field why string
---@field count integer signals merged into this contact (1 for one object)
---@field passive SensorPassive

--- The passive tier of an object, with the reason for it.
---@param state SensorState
---@param o SensorObject
---@param p SensorPassive
---@return SensorResult
function SensorModel.passiveTier(state, o, p)
    if p.surroundings.blocked then
        return { tier = 0, why = "behind the planet", count = 1, passive = p }
    end
    local pulsed = (state.echoes[o] or 0) > 0
    if p.snr < 1 and not pulsed then
        local lost = p.backgroundFactor >= 1.5 and p.signal >= M.noise
        local why = lost and "lost in the background" or (p.surroundings.by ~= "" and p.surroundings.by or "too faint")
        return { tier = 0, why = why, count = 1, passive = p }
    end
    local how = p.snr >= 1 and p.via or "pulse echo"
    local tier, why = 1, how .. ", unresolved"
    if p.cells >= M.classCells then
        tier, why = 2, how .. ", " .. SensorModel.formatCells(p.cells) .. " cells"
    end
    if p.cells >= M.identifyCells and p.lightSnr >= 1 then
        tier, why = 3, "seen, " .. SensorModel.formatCells(p.cells) .. " cells"
    end
    if tier > p.surroundings.cap then
        tier, why = p.surroundings.cap, "radiation: only fuzzy"
    end
    return { tier = tier, why = why, count = 1, passive = p }
end

--- Unresolved signals that fall in the same cell show as one blob, their
--- signal added up: the strongest one stands for the group.
local function mergeUnresolved(state, results)
    local candidates = {}
    for _, o in ipairs(state.objects) do
        local r = results[o]
        local p = r.passive
        if not p.surroundings.blocked and p.cells < M.classCells and p.snr > 0.02 and r.tier <= 1 then
            candidates[#candidates + 1] = o
        end
    end
    local groups = {}
    for _, o in ipairs(candidates) do
        local home
        for _, group in ipairs(groups) do
            for _, q in ipairs(group) do
                if angleBetween(q.pos, o.pos) / DEG <= state.dials.resolution then
                    home = group
                    break
                end
            end
            if home then break end
        end
        if home then home[#home + 1] = o else groups[#groups + 1] = { o } end
    end
    for _, group in ipairs(groups) do
        if #group >= 2 then
            local sum, lead, heard = 0, group[1], false
            for _, o in ipairs(group) do
                local r = results[o]
                sum = sum + r.passive.snr
                if r.passive.snr > results[lead].passive.snr then lead = o end
                if r.tier >= 1 then heard = true end
            end
            heard = heard or sum >= 1
            for _, o in ipairs(group) do
                local r = results[o]
                if o == lead then
                    if heard then
                        r.tier, r.count, r.why = 1, #group, string.format("%d signals in one cell", #group)
                    end
                else
                    r.tier, r.why = 0, "merged into the " .. string.lower(lead.label)
                end
            end
        end
    end
end

--- Everything the sensor makes of its objects now: passive tiers, merged
--- blobs, and what scans have already read.
---@param state SensorState
---@return table<SensorObject, SensorResult>
function SensorModel.evaluate(state)
    local results = {}
    for _, o in ipairs(state.objects) do
        results[o] = SensorModel.passiveTier(state, o, SensorModel.passive(state, o))
    end
    if state.merge then
        mergeUnresolved(state, results)
    end
    for _, o in ipairs(state.objects) do
        local r, read = results[o], state.reads[o]
        if r.tier > 0 and read and read > r.tier then
            r.tier = read
            r.why = read == 4 and "full scan read" or "scan identified it"
        end
    end
    return results
end

--- The largest distance at which `test` still holds, for tests that get
--- harder with distance: 0 if not even at 1 m, math.huge if still at 10^9 m.
---@param test fun(d: number): boolean
---@return number metres
function SensorModel.maxRange(test)
    if not test(1) then return 0 end
    local lo, hi = 0, 9
    if test(10 ^ hi) then return math.huge end
    for _ = 1, 36 do
        local mid = 0.5 * (lo + hi)
        if test(10 ^ mid) then lo = mid else hi = mid end
    end
    return 10 ^ lo
end

---@class SensorRanges
---@field detect number T1, heard
---@field classify number T2
---@field identify number T3 by sight
---@field scan3 number a scan reads T3
---@field scan4 number a scan reads T4
---@field pulse number a pulse finds it

--- Where an object would reach each tier, in open space, for a set of dials.
---@param state SensorState
---@param o SensorObject
---@param dials SensorDials|nil default: the state's
---@return SensorRanges
function SensorModel.ranges(state, o, dials)
    dials = dials or state.dials
    -- The object at distance d: its own pos, everything else read through from
    -- the object (which may itself read through from another table).
    local probe = setmetatable({ pos = { x = 0, y = 0, z = 0 } }, { __index = o })
    local function at(d)
        probe.pos.z = d
        return probe, SensorModel.passive(state, probe, dials, true)
    end
    return {
        detect = SensorModel.maxRange(function(d)
            local _, p = at(d)
            return p.snr >= 1
        end),
        classify = SensorModel.maxRange(function(d)
            local _, p = at(d)
            return p.snr >= 1 and p.cells >= M.classCells
        end),
        identify = SensorModel.maxRange(function(d)
            local _, p = at(d)
            return p.snr >= 1 and p.cells >= M.identifyCells and p.lightSnr >= 1
        end),
        scan3 = SensorModel.maxRange(function(d)
            local _, p = at(d)
            return SensorModel.scanEcho(state, p, dials).snr >= M.scanT3Snr
        end),
        scan4 = SensorModel.maxRange(function(d)
            local _, p = at(d)
            return SensorModel.scanEcho(state, p, dials).snr >= M.scanT4Snr
        end),
        pulse = SensorModel.maxRange(function(d)
            local q, p = at(d)
            return SensorModel.pulseEcho(state, q, p, dials).snr >= 1
        end),
    }
end

--- A ship heats up while thrusting, firing or cloaked, and cools down
--- otherwise. Other kinds keep their heat.
---@param o SensorObject
---@param dt number seconds
function SensorModel.stepObjectHeat(o, dt)
    if o.kind ~= "ship" then return end
    local a = o.activity
    local rate = (a == "thrust" or a == "firing" or a == "cloak") and M.shipHeatUp or -M.shipHeatDown
    o.hot = clamp((o.hot or 0) + rate * dt, 0, 1)
end

--- What a ship's own sensor adds to its signature: others hear its power
--- draw, plus the scan's power while it scans (everyone in earshot hears a
--- scan), and the heat scanning builds.
---@param state SensorState
---@return { mw: number, hot: number }
function SensorModel.ownEmission(state)
    local dials = state.dials
    return { mw = dials.ownMw + (state.scanning and dials.power or 0), hot = state.heat }
end

--- An all-round pulse: whatever echoes back strongly enough shows on the
--- contacts for a while, and every ship and station within earshot hears
--- where the sensor is.
---@param state SensorState
function SensorModel.firePulse(state)
    state.pulseAge = 0
    state.pulseHeardBy = 0
    local heard = SensorModel.heardWithin(state.dials.power)
    for _, o in ipairs(state.objects) do
        local p = SensorModel.passive(state, o)
        if not p.surroundings.blocked then
            if SensorModel.pulseEcho(state, o, p).snr >= 1 then
                state.echoes[o] = M.pulseEchoTime
            end
            if (o.kind == "ship" or o.kind == "station") and p.distance <= heard then
                state.pulseHeardBy = state.pulseHeardBy + 1
            end
        end
    end
end

--- Moves the sensor on by `dt`: pulse echoes fade, own heat builds while
--- scanning and cools after, and a scan on a contact reads its next tier once
--- it has run for the scan time and its echo is strong enough.
---@param state SensorState
---@param results table<SensorObject, SensorResult> this frame's `evaluate`
---@param dt number seconds
function SensorModel.step(state, results, dt)
    state.pulseAge = state.pulseAge + dt
    for o, left in pairs(state.echoes) do
        state.echoes[o] = left > dt and left - dt or nil
    end
    state.heat = clamp(state.heat + (state.scanning and M.heatUp or -M.heatDown) * dt, 0, 1)

    local o = state.target
    local r = o and results[o]
    if state.scanning and r and r.tier >= 1 then
        local level = SensorModel.scanEcho(state, r.passive).level
        local read = state.reads[o] or 0
        if level > read then
            local progress = (state.progress[o] or 0) + dt / state.dials.scanTime
            if progress >= 1 then
                state.reads[o] = level
                progress = 0
            end
            state.progress[o] = progress
        end
    end
end

--- Forgets everything an object left behind: echoes, reads, scan progress.
---@param state SensorState
---@param o SensorObject
function SensorModel.forget(state, o)
    state.echoes[o], state.reads[o], state.progress[o] = nil, nil, nil
    if state.target == o then state.target = nil end
end

return SensorModel
