-- The Sensor Testbed's setups: small scenes around the player's ship, each
-- showing one thing about sensors. Regenerate (B) moves to the next one.
--
-- A setup:
--   id, name, hint      what it is and what to try (shown in the testbed's text)
--   hull                the sensor it starts with (a key of SensorModel.hulls)
--   surroundings        the surroundings it starts in (default: open space)
--   target              the label of the object the scan starts on
--   objects             what it places around the ship (below)
--
-- An object has its sensor values (kind, label, radius in metres, mw,
-- activity, hot, and what a T3 and a T4 read show: name, details, shield,
-- standing, contents),
-- what builds its mesh (`mesh`: "ship" with `shipType` and `hull`, an index
-- into Config.game.shipHulls; "station"; "asteroid"), and where it goes:
--   dist, bearing       metres from the ship, degrees clockwise from its nose,
--                       in its plane
--   toward, dist        on the line from the ship to "planet" or "star",
--                       turned `offset` degrees round the ship's up axis
--   beyond, gap         behind "planet", on the far side from the ship, `gap`
--                       metres above its surface
-- `listener = true` gives it a sensor of its own (a frigate's).

-- What scans read of the kinds of ship the setups use, and their sizes
local FIGHTER = {
    kind = "ship", radius = 8, mw = 8, mesh = "ship", shipType = "Fighter", hull = 1,
    name = "VEGA DART (Y151)", details = "SHIELDS 100%", shield = 1, standing = "hostile",
    contents = { "LOADOUT: 2 PULSE CANNONS", "LOGS: PATROL ROUTE, 4 WAYPOINTS" },
}
local FREIGHTER = {
    kind = "ship", radius = 20, mw = 50, mesh = "ship", shipType = "Basic", hull = 3,
    name = "LIMTECH FREIGHTER (Y142)", details = "SHIELDS 78%", shield = 0.78,
    contents = { "CARGO: 40 T IRON ORE", "LOGS: 3 CONTRACTS, LAST DOCK 2 H AGO" },
}
local CAPITAL = {
    kind = "ship", radius = 150, mw = 500, mesh = "ship", shipType = "Basic", hull = 6,
    name = "ORION BULWARK (Y133)", details = "SHIELDS 92%", shield = 0.92,
    contents = { "HANGAR: 12 FIGHTERS", "LOGS: FLEET ORDERS, SEALED" },
}
local STATION = {
    kind = "station", radius = 316, mw = 500, mesh = "station",
    name = "COMMERCIAL NEXUS", details = "FACTION: LIMTECH",
    contents = { "TRADE DATA: 14 MARKETS", "DOCKING LOG: 31 SHIPS TODAY" },
}
local ASTEROID = {
    kind = "asteroid", radius = 500, mesh = "asteroid",
    name = "ASTEROID, 1 KM ACROSS", contents = { "IRON-RICH, YIELD HIGH", "NO SIGNS OF MINING" },
}
local WRECK = {
    kind = "wreck", radius = 30, mesh = "ship", shipType = "Basic", hull = 3,
    name = "DERELICT LIMTECH FALCON (Y98)", details = "NO POWER",
    contents = { "BLACKBOX: LAST ORDERS, 6 DAYS OLD", "BLUEPRINT FRAGMENT: 'FALCON' 4/6" },
}

--- `base` with `own` on top: one object of a kind.
local function like(base, own)
    local o = {}
    for key, value in pairs(base) do o[key] = value end
    for key, value in pairs(own) do o[key] = value end
    return o
end

local function ladder()
    local objects = {}
    local columns = {
        { base = FIGHTER, name = "Fighter", bearing = -45 },
        { base = FREIGHTER, name = "Freighter", bearing = -15 },
        { base = CAPITAL, name = "Capital ship", bearing = 15 },
        { base = STATION, name = "Station", bearing = 45 },
    }
    -- Each row a few degrees round from the last: unresolved signals in one
    -- direction merge, whatever their distance
    for row, km in ipairs({ 1, 3, 10, 30, 100 }) do
        for _, column in ipairs(columns) do
            objects[#objects + 1] = like(column.base, {
                label = string.format("%s at %d km", column.name, km), dist = km * 1000,
                bearing = column.bearing + (row - 3) * 3,
            })
        end
    end
    return objects
end

local function group()
    local objects = {}
    local spacing, dist = 60, 15000
    for i = -2, 2 do
        objects[#objects + 1] = like(FIGHTER, {
            label = string.format("Fighter %d", i + 3), dist = dist, bearing = math.deg(i * spacing / dist),
        })
    end
    return objects
end

local function listeners()
    local objects = {}
    local bearings = { -60, -30, 0, 30, 60 }
    -- A frigate is heard within 50 km while it listens, 229 km while it scans
    -- (and further as its heat builds); a capital ship within 158 and 524 km.
    -- They sit on the planet's side of the ship: seen from the other side, the
    -- planet behind the ship would hide it.
    for i, km in ipairs({ 20, 40, 80, 160, 400 }) do
        objects[#objects + 1] = like(FREIGHTER, {
            label = string.format("Frigate at %d km", km), toward = "planet", dist = km * 1000, offset = bearings[i],
            listener = true,
        })
    end
    return objects
end

-- A rock's radius in degrees at 11 km, so a ship can sit just beside it
local BESIDE_ROCK = math.deg(800 / 11000)

return {
    {
        id = "reference",
        name = "Reference scene",
        hint = "The sensor mock's scene. Tab picks a target, X scans it, Z pulses; 5 changes a ship's activity.",
        hull = "frigate",
        target = "Freighter",
        objects = {
            like(STATION, { label = "Station", dist = 1.2e4, bearing = 150 }),
            like(FREIGHTER, { label = "Freighter", dist = 4e3, bearing = -28 }),
            like(FIGHTER, { label = "Fighter", dist = 1.06e4, bearing = 73.5 }),
            like(ASTEROID, { label = "Asteroid", dist = 6e3, bearing = 64 }),
            like(ASTEROID, { label = "Asteroid", dist = 1.1e4, bearing = 71 }),
            like(WRECK, { label = "Wreck", dist = 400, bearing = 4 }),
        },
    },
    {
        id = "hulls",
        name = "Fighter, frigate, capital ship",
        hint = "The same ships at 1, 3, 10, 30 and 100 km (class I, II, III and a station, left to right). "
            .. "1 2 3 switch hulls: how far each hears, classes and identifies them.",
        hull = "frigate",
        target = "Freighter at 10 km",
        objects = ladder(),
    },
    {
        id = "hiding",
        name = "Hiding",
        hint = "Fighters hiding. Tab through them: the tuning panel (8) says why one is no contact. "
            .. "1 2 3 hull, 4 surroundings.",
        hull = "frigate",
        target = "Next to a rock",
        objects = {
            like(FIGHTER, { label = "In the open", dist = 11000, bearing = -70 }),
            like(ASTEROID, { label = "Rock", dist = 11000, bearing = -40 }),
            like(FIGHTER, { label = "Next to a rock", dist = 11000, bearing = -40 + BESIDE_ROCK }),
            like(ASTEROID, { label = "Rock", dist = 5000, bearing = 0 }),
            like(FIGHTER, { label = "Behind a rock", dist = 11000, bearing = 0 }),
            like(FIGHTER, { label = "Cloaked", dist = 11000, bearing = 40, activity = "cloak", hot = 1 }),
            like(FIGHTER, { label = "Running cold", dist = 11000, bearing = 70, mw = 1 }),
            like(FIGHTER, { label = "Firing", dist = 25000, bearing = 100, activity = "firing", hot = 1 }),
            like(FIGHTER, { label = "Firing, in the star's glare", toward = "star", dist = 25000,
                activity = "firing", hot = 1 }),
            like(FIGHTER, { label = "In front of the planet", toward = "planet", dist = 11000 }),
            like(FIGHTER, { label = "Behind the planet", beyond = "planet", gap = 1e6 }),
        },
    },
    {
        id = "active",
        name = "Passive against active",
        hint = "A nebula: no light gets through, so listening only classes ships by their EM. "
            .. "X scans to identify them, Z pulses to find the cold wreck; 4 back to open space.",
        hull = "frigate",
        surroundings = "nebula",
        target = "Freighter",
        objects = {
            like(FREIGHTER, { label = "Freighter", dist = 1500, bearing = -20 }),
            like(WRECK, { label = "Wreck", dist = 350, bearing = 15 }),
            like(STATION, { label = "Station", dist = 6000, bearing = 60 }),
            like(FIGHTER, { label = "Fighter", dist = 2500, bearing = -60 }),
        },
    },
    {
        id = "merging",
        name = "Resolution and merging",
        hint = "Five fighters 60 m apart at 15 km: one blob for a fighter's sensor, five for the frigate's, "
            .. "five classed for the capital ship's. 7 merging, 6 resolution rule.",
        hull = "fighter",
        target = "Fighter 3",
        objects = group(),
    },
    {
        id = "heard",
        name = "Being heard",
        hint = "Five frigates listening, 20 to 400 km away: the text shows which hear you. "
            .. "Hold X to scan or Z to pulse; 1 2 3 change how loud your ship is.",
        hull = "frigate",
        target = "Frigate at 20 km",
        objects = listeners(),
    },
}
