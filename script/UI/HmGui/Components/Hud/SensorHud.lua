-- Sensor HUD: what a ship's sensor shows, with words only where nothing else
-- can say it (a target's name, its distance, what a full scan reads).
--
-- Colours: a contact is yellow while it's only heard (T1: unknown, possibly a
-- threat), and from T2 on, when it's classed, its standing: green owned, cyan
-- friendly, grey neutral (and wrecks), red hostile. White is yours: your scan
-- and pulse, the ring on your target, the arrow toward it. The HUD itself is
-- blue. Contacts show as the game map icons (fighter, corvette, capital
-- ship, station, asteroid, planet, star; res/tex2d/map-icon).
--
-- The radar, at the bottom centre: a small disc tilted well away from the
-- viewer with the ship in the middle, nose up, distance on a log scale (50 m
-- at the centre, 100 km at the rim), a faint wedge for what's in view. A
-- contact's foot sits on the disc at its bearing, on a stick up to it above
-- the ship's plane or a dashed one down below it; it shows as a dot (a ship's
-- as big as its class; a station a ring with a dot, a planet a ring), a
-- yellow blob at T1. Dashed rings show where the target would turn T2, T3 and
-- T4 in open space; a ring round the disc bulges where signals come from. How
-- far the ship is heard is a dashed ring (a white one further out while it
-- scans or pulses); its heat an arc round it; ripples over a contact mean it
-- heard the last pulse. What the sensor knew of a contact stays: while it
-- senses less, the dot fades and an arc round it closes as the knowledge
-- ages; lost, a dashed ring stays where it was last seen.
--
-- Focused (`hud:setFocused`, `hud:toggleFocus`), the radar grows and tilts
-- toward top-down, and its contacts show as their map icons; given a
-- `focusKey`, a small button beside the rim shows that key, lit while it's
-- focused. It moves between the two over a fraction of a second.
--
-- In the view: each contact's icon where it is on the screen, the target in
-- brackets with its distance (and its name once it's identified). Off screen:
-- the screen's edge glows toward unknown and hostile contacts, a chevron with
-- its icon points to each classed contact, and a white arrow with its
-- distance to the target.
--
-- The target panel, at the bottom right: a hologram circle that fills in as
-- the sensor learns more, like the HUD vision's target panel. Static without a
-- contact, a yellow blob at T1, its icon at T2, a wireframe of it with its
-- shields (the ring's glow), power and activity at T3, brighter at T4 with
-- what the full scan read beside it. A scan's progress runs round the rim; the
-- echo fills a meter under it with a notch for T3 and one for T4; chevrons ask
-- to get closer when it's too weak to read more; ripples show the target hears
-- the scan.
--
--   local hud = UIComponent.SensorHud { visible = function() return shown end }
--   -- every frame, before rendering:
--   hud:update(SensorPicture.build(sensor, forward, right, up), dt)
--   -- on the focus key:
--   hud:toggleFocus()
--
-- Places and sizes are pixels on a 1920 x 1080 screen and scale with the
-- window height. The radar keeps its place relative to the bottom centre of
-- the screen, the target panel relative to the bottom right corner.

local SensorModel = require("Modules.Sensors.SensorModel")
local SensorText = require("Modules.Sensors.SensorText")
local CameraManager = require("Modules.Cameras.Managers.CameraManager")
local PhysicsComponents = require("Modules.Physics.Components")
local UniverseScaleConfig = require("Config.Gen.UniverseScaleConfig")

---@class UIComponentSensorHud: UIComponent
---@field state UIComponentState
---@field update fun(self: UIComponentSensorHud, picture: SensorPictureData|nil, dt: number|nil) takes this frame's picture; nil hides the HUD
---@field render fun(self: UIComponentSensorHud) renders the markers, the radar and the target panel
---@field setFocused fun(self: UIComponentSensorHud, focused: boolean) the radar focused (big, near top-down, icons) or compact
---@field toggleFocus fun(self: UIComponentSensorHud) focuses the radar, or puts it back
---@field isFocused fun(self: UIComponentSensorHud): boolean whether the radar is focused (or on its way)

---@class UIComponentSensorHudConstructor
---@field visible boolean|(fun(): boolean)|nil
---@field scale number|(fun(): number)|nil sizes relative to 1080p; default window height / 1080
---@field focusKey string|nil the focus key's name, shown on the button beside the radar; nil: no button

local DEG = math.pi / 180

-- Geometry, in pixels on a 1920 x 1080 screen.
-- The radar compact and focused: its centre, radius and tilt (0 = top-down).
-- Focused, its signal ring (bulges and all) stays clear of the crosshair
-- above it and of the screen's bottom edge below it.
local RADAR = {
    compact = { x = 960, y = 978, r = 80, tilt = 62 * DEG },
    focused = { x = 960, y = 812, r = 230, tilt = 32 * DEG },
    depth = 7, -- perspective, in radii
}
-- Focusing: one table, as `render` is close to LuaJIT's limit of 60
-- upvalues per function.
local FOCUS = {
    time = 0.3, -- seconds the radar takes to focus or go back
    -- focused, how much bigger the rings and blobs round its contacts are, and
    -- how much smaller (in radar radii) your ship, your heat and the signal
    -- ring's distance from the rim, so they keep about their size on screen
    grow = 2,
    shrink = 0.55,
    button = { gap = 34, r = 11 }, -- the focus button: right of the rim, its radius
    keycap = { w = 18, h = 16 },   -- the key's cap under the button
    rimLabelLowest = 1070,         -- the rim's distance label's lowest baseline, clear of the screen's bottom
}
-- Focused, a contact's map icon on the radar (longer side, 1080p pixels)
local RADAR_ICON = { ship = { 16, 19, 23 }, station = 21, asteroid = 12, planet = 24, star = 20 }
local PANEL = { x = 1720, y = 858, r = 128 } -- the HUD vision's target panel
local RADAR_MIN, RADAR_MAX = 50, 1e5 -- metres at the radar's centre and at its rim
local SIGNAL_RING = 1.1              -- the signal ring's radius, in radar radii
local SIGNAL_BULGE = 0.3             -- how far a strong signal bulges it out
local HEAT_RING = 0.17               -- the heat arc round the ship, in radar radii
local TIER_RING_ALPHA = { [2] = 0.35, [3] = 0.45, [4] = 0.55 } -- brighter toward the middle
local PULSE_RING_TIME = 1.2          -- seconds the pulse ring takes to reach the rim and fade
local PULSE_SHOUT_TIME = 4           -- seconds the pulse's footprint stays up
local ECHO_FADE_TIME = 5             -- a blob seen by its pulse echo alone fades over its last seconds
local STALE_FULL = 120               -- seconds until a remembered contact's age arc closes
local GHOST_FADE = 120               -- seconds a lost ship's ghost takes to fade to its faintest
-- Strokes per closed curve on the radar; each is bent to follow the curve
local SEGMENTS = { rim = 24, tier = 20, signal = 60, heat = 16, disc = 24 }
local SHIP_DOT = { 2.3, 3, 3.8 } -- a ship's dot on the radar by its class (I, II, III), radius
local RADAR_BLOB, MERGED_RADAR_BLOB = 8, 11
-- In the view: icons; none once the object itself is this big on screen
-- (1080p pixels, radius); on screen means this far inside the window's edge
local MARKER_ICON, MARKER_BLOB, MARKER_MAX_SIZE, ON_SCREEN_MARGIN = 18, 12, 24, 20
local BRACKET = { half = 30, arm = 10 }
-- Off screen: the edge glow's length along the edge (each way), and how far
-- inside the edge the edge markers and the target arrow sit
local EDGE = { glow = 210, marker = 34, arrow = 64, icon = 14 }
local PANEL_ICON, PANEL_BLOB, MERGED_PANEL_BLOB = 84, 36, 46
local ECHO_METER = { from = 158.4, sweep = 43.2, gap = 16 } -- the echo meter's arc under the panel (degrees, 0 = up)
local FONT, FONT_MIN = "UbuntuBold", 9

local function rgb(r, g, b) return { r / 255, g / 255, b / 255 } end

--- Colours and looks; public as `UIComponent.SensorHud.style`.
local STYLE = {
    hud = rgb(26, 153, 255),
    cyan = rgb(51, 230, 255),
    pale = rgb(190, 225, 255),
    core = rgb(240, 250, 255),
    white = rgb(235, 245, 255),   -- yours: your scan and pulse, your target
    unknown = rgb(255, 216, 77),  -- heard, not yet classed: possibly a threat
    standing = {
        owned = rgb(90, 230, 140),
        friendly = rgb(51, 230, 255),
        neutral = rgb(170, 178, 190),
        hostile = rgb(255, 69, 69),
        wreck = rgb(170, 178, 190),
    },
    halo = Color(0, 0.02, 0.04, 0.6),
    -- neon: the rims, rings and the hologram's outline, with a soft glow;
    -- line: thin rings and outlines; cue: the edge glow, a lit line with a
    -- wide halo into the screen. Neon and line are alpha blended: a curve is
    -- many strokes whose round ends overlap, and added light would make
    -- bright beads there.
    looks = {
        neon = { core = 0.9, coreWidth = 0.7, glow = 0.3, glowWidth = 3, shade = 1 },
        line = { core = 1, coreWidth = 0.6, glow = 0, shade = 1 },
        cue = { core = 0.8, coreWidth = 1, glow = 0.8, glowWidth = 44, shade = 0.35 },
    },
}
local SCALED_LOOK_PARAMS = { coreWidth = true, glowWidth = true, haloWidth = true }
local NO_OUTLINE = Color(0, 0, 0, 0)

-- The map icons: white textures with the shape in the alpha channel, made
-- from the game map icons' SVG by tools/ui-icons. An icon's longer side fills
-- ICON_FILL of its texture; `<name>-glow` is it blurred, filling GLOW_FILL.
local ICON_FILL, GLOW_FILL = 120 / 128, 32 / 64
local SHIP_ICONS = { "fighter", "corvette", "capital-ship" } -- by class: I, II, III
local iconTextures = {}

--- An icon's texture, loaded once. `Tex2D.Load` gives a single level, so the
--- icon is copied into a texture with a mip chain: drawn far smaller than
--- its texture, it stays smooth instead of shimmering.
local function iconTexture(name)
    local tex = iconTextures[name]
    if not tex then
        local src = Tex2D.Load("map-icon/" .. name)
        local size = src:getSize()
        tex = Tex2D.Create(size.x, size.y, TexFormat.RGBA8, { mips = true })
        tex:setDataBytes(src:getDataBytes(PixelFormat.RGBA, DataFormat.U8), PixelFormat.RGBA, DataFormat.U8)
        tex:genMipmap()
        iconTextures[name] = tex
    end
    return tex
end

local hudCount = 0 -- each HUD defines its looks under its own names

local function clamp(x, lo, hi) return math.min(hi, math.max(lo, x)) end

local function wrapDegrees(a) return (a + 540) % 360 - 180 end

local function color(c, alpha) return Color(c[1], c[2], c[3], math.max(0, alpha or 1)) end

--- Where a distance sits on the radar, 0 (centre) to 1 (rim).
local function radarRadius(d)
    local lo, hi = math.log10(RADAR_MIN), math.log10(RADAR_MAX)
    return clamp((math.log10(math.max(1, d)) - lo) / (hi - lo), 0, 1)
end

local function lerp(a, b, t) return a + (b - a) * t end

-- The radar's place, radius and tilt this frame: compact, focused or on its
-- way between them (focusRadar).
local radar = { x = RADAR.compact.x, y = RADAR.compact.y, r = RADAR.compact.r, tilt = RADAR.compact.tilt }

--- Puts the radar `focus` of the way (0 to 1, eased in and out) from compact
--- to focused. Returns the eased focus, how much bigger the marks round its
--- contacts are, and how much smaller your ship and heat are in radar radii.
local function focusRadar(focus)
    local t = focus * focus * (3 - 2 * focus)
    for _, key in ipairs({ "x", "y", "r", "tilt" }) do
        radar[key] = lerp(RADAR.compact[key], RADAR.focused[key], t)
    end
    return t, lerp(1, FOCUS.grow, t), lerp(1, FOCUS.shrink, t)
end

--- A point in the radar's space (x right, y ahead, h up from the disc, in
--- radar radii) in screen pixels at 1080p, through the disc's tilt and
--- perspective.
local function onRadar(x, y, h)
    h = h or 0
    local ct, st = math.cos(radar.tilt), math.sin(radar.tilt)
    local Y, Z = y * ct + h * st, -y * st + h * ct
    local p = RADAR.depth / (RADAR.depth - Z)
    return radar.x + radar.r * x * p, radar.y - radar.r * Y * p
end

--- A contact's colour: unknown (yellow) until it's classed, then its standing.
local function contactColor(o, tier)
    if tier <= 1 then return STYLE.unknown end
    if o.kind == "wreck" then return STYLE.standing.wreck end
    return STYLE.standing[o.standing or "neutral"] or STYLE.standing.neutral
end

--- A contact's map icon.
local function iconName(o)
    if o.kind == "station" or o.kind == "asteroid" or o.kind == "star" then return o.kind end
    if o.kind == "planet" then return "planet-rocky" end
    return SHIP_ICONS[SensorModel.shipClass(o)] or SHIP_ICONS[1] -- ships and wrecks by class
end

--- A contact's icon size on the focused radar.
local function radarIconSize(o)
    local size = RADAR_ICON[o.kind]
    if type(size) == "number" then return size end
    return RADAR_ICON.ship[SensorModel.shipClass(o)] or RADAR_ICON.ship[1] -- ships and wrecks by class
end

--- The hologram stand-in: a wireframe per kind, in radii of the panel's
--- figure (y down), with a few lines inside. `round` draws a circle first.
local SILHOUETTES = {
    station = { lines = { { 0, -0.9, 0, 0.9 }, { -0.78, -0.45, 0.78, 0.45 }, { -0.78, 0.45, 0.78, -0.45 } } },
    asteroid = {
        outline = { { -0.2, -0.85 }, { 0.45, -0.7 }, { 0.85, -0.15 }, { 0.7, 0.5 }, { 0.15, 0.85 }, { -0.55, 0.7 }, { -0.85, 0.1 }, { -0.7, -0.5 } },
        lines = { { -0.3, -0.2, -0.1, -0.1 }, { 0.2, 0.3, 0.4, 0.35 } },
    },
    planet = { round = 0.85, lines = { { -0.85, 0, 0.85, 0 }, { -0.7, -0.45, 0.7, -0.45 }, { -0.7, 0.45, 0.7, 0.45 } } },
    star = { round = 0.55, rays = true },
    wreck = {
        outline = { { -0.15, -0.95 }, { 0.25, -0.95 }, { 0.3, -0.1 }, { 0.05, 0.05 }, { 0.35, 0.2 }, { 0.3, 0.95 }, { -0.2, 0.95 }, { -0.25, 0.25 }, { 0, 0.1 }, { -0.2, -0.05 } },
        lines = { { -0.5, -0.4, -0.25, -0.35 }, { 0.4, 0.5, 0.6, 0.6 } },
    },
    class1 = {
        outline = { { 0, -0.95 }, { 0.25, -0.2 }, { 0.85, 0.55 }, { 0.2, 0.45 }, { 0, 0.75 }, { -0.2, 0.45 }, { -0.85, 0.55 }, { -0.25, -0.2 } },
        lines = { { 0, -0.6, 0, 0.5 } },
    },
    class2 = {
        outline = { { -0.18, -0.95 }, { 0.18, -0.95 }, { 0.25, 0.85 }, { -0.25, 0.85 } },
        lines = { { -0.55, -0.35, -0.25, -0.35 }, { 0.25, -0.35, 0.55, -0.35 }, { -0.55, 0.25, -0.25, 0.25 }, { 0.25, 0.25, 0.55, 0.25 } },
    },
    class3 = {
        outline = { { 0, -0.95 }, { 0.35, -0.6 }, { 0.4, 0.7 }, { 0.2, 0.95 }, { -0.2, 0.95 }, { -0.4, 0.7 }, { -0.35, -0.6 } },
        lines = { { -0.35, -0.2, 0.35, -0.2 }, { -0.4, 0.3, 0.4, 0.3 } },
    },
}
SILHOUETTES.station.outline = {} -- a hexagon, its spokes corner to corner
for i = 0, 5 do
    local a = (i * 60 + 30) * DEG
    SILHOUETTES.station.outline[i + 1] = { 0.9 * math.cos(a), 0.9 * math.sin(a) }
end

--- What a full-scan line is about, as an icon; the words before its colon
--- are left out then.
local function contentIcon(line)
    local key = line:match("^([^:]*):")
    if not key then return nil end
    if key:find("CARGO") then return "crate" end
    if key:find("LOG") or key:find("BLACKBOX") then return "page" end
    if key:find("BLUEPRINT") then return "grid" end
    if key:find("LOADOUT") then return "aim" end
    if key:find("TRADE") then return "bars" end
    return nil
end

---@param _ table
---@param args UIComponentSensorHudConstructor
---@return UIComponentSensorHud|nil
local function new(_, args)
    if not args then return end

    local hud = {}
    hud.state = UICore.ComponentState {
        visible = args.visible,
        scale = args.scale or function() return Window:height() / 1080 end,
    }

    hudCount = hudCount + 1
    local lookPrefix = string.format("sensorHud%d.", hudCount)
    local lookNames, definedScale = {}, nil

    local function defineLooks(scale)
        if definedScale == scale then return end
        definedScale = scale
        for key, values in pairs(UIComponent.SensorHud.style.looks) do
            local scaled = {}
            for param, value in pairs(values) do
                scaled[param] = SCALED_LOOK_PARAMS[param] and value * scale or value
            end
            lookNames[key] = lookPrefix .. key
            UICore.Look.define(lookNames[key], scaled)
        end
    end

    ---@type SensorPictureData|nil
    local picture
    local clock = 0
    -- the radar's focus: `at` runs from 0 (compact) to 1 (focused) toward `on`
    local focus = { on = false, at = 0, key = args.focusKey }

    function hud:update(newPicture, dt)
        picture = newPicture
        clock = clock + (dt or 0)
        focus.at = clamp(focus.at + (focus.on and 1 or -1) * (dt or 0) / FOCUS.time, 0, 1)
    end

    function hud:setFocused(on) focus.on = on == true end

    function hud:toggleFocus() focus.on = not focus.on end

    function hud:isFocused() return focus.on end

    function hud:render()
        if not picture or not self.state.visible() then return end

        local style = UIComponent.SensorHud.style
        local s = self.state.scale()
        defineLooks(s)
        local W, H = Window:width(), Window:height()

        -- The radar's focus, eased (fe: 0 compact, 1 focused); marks round
        -- its contacts grow with it, your ship and heat shrink in radii.
        local fe, grow, shrink = focusRadar(focus.at)

        -- 1080p places to window pixels: the radar from the bottom centre,
        -- the target panel from the bottom right, the view from the top left.
        local ax, ay -- this anchor's offsets
        local function anchorBottomCentre() ax, ay = W / 2 - 960 * s, H - 1080 * s end
        local function anchorBottomRight() ax, ay = W - 1920 * s, H - 1080 * s end
        local function anchorWindow() ax, ay = 0, 0 end
        local function X(x) return ax + x * s end
        local function Y(y) return ay + y * s end

        -- Shapes, at 1080p places.
        --- A stroke from one place to another, bent `bend` degrees on the
        --- way (positive = clockwise). Its box holds the stroke with room to
        --- spare: the bulge of the bend (the arc's sagitta) and its width.
        local function line(c, alpha, width, x0, y0, x1, y1, look, bend, dash)
            local pad = width
            local half = math.abs(bend or 0) * DEG / 2
            if half > 1e-4 then
                local chord = math.sqrt((x1 - x0) ^ 2 + (y1 - y0) ^ 2)
                pad = pad + chord / (2 * math.sin(half)) * (1 - math.cos(half))
            end
            local left, top = X(math.min(x0, x1) - pad), Y(math.min(y0, y1) - pad)
            local w = (math.abs(x1 - x0) + 2 * pad) * s
            local h = (math.abs(y1 - y0) + 2 * pad) * s
            local function fx(x) return (X(x) - left) / w end
            local function fy(y) return (Y(y) - top) / h end
            Gui:strokeBetween(color(c, alpha), fx(x0), fy(y0), fx(x1), fy(y1), bend or 0, width * s)
            Gui:setFixedSize(w, h)
            Gui:setPosition(left, top)
            Gui:setStrokeLook(lookNames[look or "line"])
            if dash then Gui:setStrokeDash(dash[1] * s, dash[2] * s, (dash[3] or 0) * s) else Gui:setStrokeRoundCaps(true) end
        end

        local function polyline(c, alpha, width, points, look, dash)
            for i = 1, #points - 1 do
                local a, b = points[i], points[i + 1]
                line(c, alpha, width, a[1], a[2], b[1], b[2], look, 0, dash)
            end
        end

        --- A curve on the radar's plane, from angle `from` to `to` (radians,
        --- 0 ahead, growing clockwise): radiusAt(angle) in radar radii. Each
        --- stroke is bent by the turn of the curve's direction between its
        --- ends, so a few dozen strokes follow the curve smoothly.
        local function radarCurve(c, alpha, width, radiusAt, segments, look, from, to, dash)
            from, to = from or 0, to or 2 * math.pi
            local function at(a)
                local r = radiusAt(a)
                return onRadar(r * math.sin(a), r * math.cos(a))
            end
            local step = (to - from) / segments
            local function heading(a) -- screen direction of the curve at a, degrees (y down: clockwise grows)
                local x0, y0 = at(a - 0.05 * step)
                local x1, y1 = at(a + 0.05 * step)
                return math.atan2(y1 - y0, x1 - x0) / DEG
            end
            local x0, y0 = at(from)
            local h0 = heading(from)
            for i = 1, segments do
                local a = from + i * step
                local x1, y1 = at(a)
                local h1 = heading(a)
                line(c, alpha, width, x0, y0, x1, y1, look, wrapDegrees(h1 - h0), dash)
                x0, y0, h0 = x1, y1, h1
            end
        end

        local function triangle(c, alpha, x0, y0, x1, y1, x2, y2)
            local left, top = X(math.min(x0, x1, x2)), Y(math.min(y0, y1, y2))
            local w = math.max(1, (math.max(x0, x1, x2) - math.min(x0, x1, x2)) * s)
            local h = math.max(1, (math.max(y0, y1, y2) - math.min(y0, y1, y2)) * s)
            local function fx(x) return (X(x) - left) / w end
            local function fy(y) return (Y(y) - top) / h end
            Gui:triangle(color(c, alpha), fx(x0), fy(y0), fx(x1), fy(y1), fx(x2), fy(y2), false)
            Gui:setFixedSize(w, h)
            Gui:setPosition(left, top)
        end

        --- A wedge of the radar's disc, from angle `from` to `to` (radians,
        --- 0 ahead, clockwise), out to radius r: a fan of triangles.
        local function radarFan(c, alpha, r, from, to, steps)
            local cx, cy = onRadar(0, 0)
            local px, py = onRadar(r * math.sin(from), r * math.cos(from))
            for i = 1, steps do
                local a = from + (to - from) * i / steps
                local qx, qy = onRadar(r * math.sin(a), r * math.cos(a))
                triangle(c, alpha, cx, cy, px, py, qx, qy)
                px, py = qx, qy
            end
        end

        local function rect(c, alpha, x, y, w, h)
            Gui:rect()
            Gui:setBackgroundColor(color(c, alpha))
            Gui:setFixedSize(math.max(1, w * s), math.max(1, h * s))
            Gui:setPosition(X(x), Y(y))
        end

        --- A round shape (`Gui:circle` or `ring`) of radius r around x, y.
        local function round(shape, c, alpha, x, y, r, glow)
            Gui[shape](Gui, color(c, alpha), glow == true)
            Gui:setFixedSize(2 * r * s, 2 * r * s)
            Gui:setPosition(X(x) - r * s, Y(y) - r * s)
        end

        --- An unresolved signal: a soft blob, brightest in the middle (two
        --- soft-edged discs and a glowing core on top of each other).
        local function blob(c, alpha, x, y, r)
            round("circle", c, 0.25 * alpha, x, y, r)
            round("circle", c, 0.4 * alpha, x, y, 0.55 * r)
            round("circle", c, 0.9 * alpha, x, y, 0.25 * r, true)
        end

        --- An arc of radius r round x, y, from `start` degrees (0 = up,
        --- clockwise) over `sweep` degrees.
        local function arc(c, alpha, width, x, y, r, start, sweep, look, dash)
            Gui:arc(color(c, alpha), width * s, start, sweep)
            Gui:setFixedSize(2 * r * s, 2 * r * s)
            Gui:setPosition(X(x) - r * s, Y(y) - r * s)
            Gui:setStrokeLook(lookNames[look or "line"])
            if dash then Gui:setStrokeDash(dash[1] * s, dash[2] * s, (dash[3] or 0) * s) end
        end

        local function font(size) return Cache.Font(FONT, math.max(FONT_MIN, math.floor(size * s + 0.5))) end

        --- Text with a dark halo, its baseline at y; align "left", "right" or "center".
        local function text(str, x, y, size, c, alpha, align)
            local w, h = 700 * s, size * 1.6 * s
            local left = X(x)
            if align == "right" then left = left - w elseif align == "center" then left = left - w / 2 end
            Gui:beginStackContainer()
            Gui:setClipping(false)
            Gui:setFixedSize(w, h)
            Gui:setPosition(left, Y(y) - size * 1.15 * s)
            local halo = style.halo -- fainter text, fainter halo
            Gui:textOutlined(str, font(size), color(c, alpha), NO_OUTLINE, Color(halo.r, halo.g, halo.b, halo.a * alpha))
            Gui:setAlignment(
                align == "right" and AlignHorizontal.Right or align == "center" and AlignHorizontal.Center or AlignHorizontal.Left,
                AlignVertical.Center)
            Gui:endContainer()
        end

        --- A text's width at 1080p.
        local function textWidth(str, size) return font(size):getSize2(str).x / s end

        --- A closed outline of glyph-radii points round x, y at `size`, filled
        --- (a fan from its centre) when `fill` > 0.
        local function shape(points, x, y, size, c, alpha, width, fill, look, dash)
            local pts = {}
            for _, p in ipairs(points) do pts[#pts + 1] = { x + p[1] * size, y + p[2] * size } end
            if fill and fill > 0 then
                for i = 1, #pts do
                    local a, b = pts[i], pts[i % #pts + 1]
                    triangle(c, fill, x, y, a[1], a[2], b[1], b[2])
                end
            end
            pts[#pts + 1] = pts[1]
            polyline(c, alpha, width, pts, look, dash)
        end

        --- A map icon centred on x, y, its longer side `size`, tinted `c`;
        --- `glow` (0 to 1) draws its soft glow under it.
        local function mapIcon(name, x, y, size, c, alpha, glow)
            local function image(tex, box, a)
                Gui:image(tex)
                Gui:setImageColor(color(c, a))
                Gui:setImageSmooth(true)
                Gui:setFixedSize(box * s, box * s)
                Gui:setPosition(X(x) - box * s / 2, Y(y) - box * s / 2)
            end
            if glow and glow > 0 then image(iconTexture(name .. "-glow"), size / GLOW_FILL, 0.8 * glow * alpha) end
            image(iconTexture(name), size / ICON_FILL, alpha)
        end

        --- Small icons in a box of size `z` round x, y.
        local function icon(kind, x, y, z, c, alpha)
            local w = 1.4
            if kind == "bolt" then
                polyline(c, alpha, w, { { x + 0.15 * z, y - z }, { x - 0.45 * z, y + 0.1 * z }, { x, y + 0.1 * z }, { x - 0.15 * z, y + z }, { x + 0.45 * z, y - 0.1 * z }, { x, y - 0.1 * z }, { x + 0.15 * z, y - z } })
            elseif kind == "crate" then
                shape({ { -0.8, -0.6 }, { 0.8, -0.6 }, { 0.8, 0.7 }, { -0.8, 0.7 } }, x, y, z, c, alpha, w)
                line(c, alpha, w, x - 0.8 * z, y - 0.15 * z, x + 0.8 * z, y - 0.15 * z)
                line(c, alpha, w, x, y - 0.6 * z, x, y + 0.7 * z)
            elseif kind == "page" then
                shape({ { -0.6, -0.85 }, { 0.6, -0.85 }, { 0.6, 0.85 }, { -0.6, 0.85 } }, x, y, z, c, alpha, w)
                for _, v in ipairs({ -0.4, 0, 0.4 }) do line(c, alpha, w, x - 0.35 * z, y + v * z, x + 0.35 * z, y + v * z) end
            elseif kind == "grid" then
                shape({ { -0.8, -0.8 }, { 0.8, -0.8 }, { 0.8, 0.8 }, { -0.8, 0.8 } }, x, y, z, c, alpha, w)
                for _, v in ipairs({ -0.27, 0.27 }) do
                    line(c, alpha, w, x + v * z, y - 0.8 * z, x + v * z, y + 0.8 * z)
                    line(c, alpha, w, x - 0.8 * z, y + v * z, x + 0.8 * z, y + v * z)
                end
            elseif kind == "aim" then
                round("ring", c, alpha, x, y, 0.6 * z)
                for _, d in ipairs({ { 0, -1 }, { 0, 1 }, { -1, 0 }, { 1, 0 } }) do
                    line(c, alpha, w, x + d[1] * 0.3 * z, y + d[2] * 0.3 * z, x + d[1] * z, y + d[2] * z)
                end
            elseif kind == "bars" then
                for i, hgt in ipairs({ 0.5, 1, 0.75 }) do rect(c, alpha, x + (i - 2) * 0.55 * z - 0.18 * z, y + 0.8 * z - hgt * 1.6 * z, 0.36 * z, hgt * 1.6 * z) end
            elseif kind == "flame" then
                line(c, alpha, w, x, y - z, x, y + z, "line", 140)
                line(c, alpha, w, x, y + z, x, y - z, "line", 140)
            elseif kind == "eyeOff" then
                line(c, alpha, w, x - z, y, x + z, y, "line", 80)
                line(c, alpha, w, x + z, y, x - z, y, "line", 80)
                line(c, alpha, w, x - 0.8 * z, y + 0.8 * z, x + 0.8 * z, y - 0.8 * z)
            elseif kind == "ping" then
                arc(c, alpha, w, x, y + z, 0.45 * z, -48, 96)
                arc(c, alpha, w, x, y + z, 0.85 * z, -48, 96)
            elseif kind == "chevrons" then
                polyline(c, alpha, w, { { x - 0.7 * z, y - 0.05 * z }, { x, y - 0.7 * z }, { x + 0.7 * z, y - 0.05 * z } })
                polyline(c, alpha, w, { { x - 0.7 * z, y + 0.6 * z }, { x, y - 0.05 * z }, { x + 0.7 * z, y + 0.6 * z } })
            end
        end

        --- A contact on the radar: a dot (a ship's as big as its class, a
        --- station a ring round a dot, a planet a ring), in its colour.
        --- `mode`: nil while it's as known as it ever was; "stale" when it
        --- knew more of it before (fainter); "ghost" when it's out of contact
        --- (a dashed ring).
        local function radarDot(o, x, y, tier, mode, alpha)
            local c = contactColor(o, tier)
            if mode == "ghost" then
                arc(c, 0.9 * alpha, 1.2, x, y, 3.5 * grow, 0, 360, "line", { 2, 2 })
                return
            end
            local a = (mode == "stale" and 0.55 or 0.95) * alpha
            if o.kind == "planet" then
                round("ring", c, a, x, y, 4.5)
            elseif o.kind == "station" then
                round("ring", c, a, x, y, 3.2)
                round("circle", c, a, x, y, 1.4)
            elseif o.kind == "star" then
                round("circle", c, a, x, y, 3, true)
            elseif o.kind == "asteroid" then
                round("circle", c, a, x, y, 1.8)
            else
                round("circle", c, (o.kind == "wreck" and 0.6 or 1) * a, x, y, SHIP_DOT[SensorModel.shipClass(o)] or SHIP_DOT[1])
            end
        end

        --- How strongly a lost contact's ghost still shows: a ship's fades as
        --- it could have moved on; anything else stays where it was.
        local function ghostAlpha(c)
            if c.object.kind ~= "ship" then return 0.6 end
            return math.max(0.25, 0.75 - 0.5 * c.lost / GHOST_FADE)
        end

        --- A tier's look for the echo meter's notches: T3 a filled triangle,
        --- T4 one with a bright core.
        local function tierIcon(tier, x, y)
            shape({ { 0, -1 }, { 0.92, 0.7 }, { -0.92, 0.7 } }, x, y, 5, style.pale, 0.95, 1.3, 0.8)
            if tier >= 4 then round("circle", style.core, 1, x, y + 0.5, 1.6, true) end
        end

        local function staticDots(x, y, r, n, alpha)
            for i = 1, n do
                local k = math.sin(i * 91.7 + math.floor(clock * 12) * 13.1) * 43758.5453
                local u, v = k - math.floor(k), math.abs(math.sin(k * 1.7)) % 1
                local rr, a = math.sqrt(u) * r * 0.85, v * 2 * math.pi
                rect(style.pale, alpha * (0.4 + 0.6 * v), x + math.cos(a) * rr, y + math.sin(a) * rr, 2.5, 2)
            end
        end

        local target = picture.target
        local targetObject = target and target.contact.object

        -- A full-window layer; everything in it is placed in window pixels.
        Gui:beginStackContainer()
        Gui:setClipping(false)

        ------------------------------------------------------------------
        -- Your view: each contact's icon where it is on the screen, the
        -- target in brackets. Off screen: the edge glows toward unknown and
        -- hostile contacts, a chevron with its icon points to each classed
        -- one, a white arrow to the target. Planets and stars need none.
        do
            local cam = CameraManager:getActiveCameraEntity()
            local camTransform = cam and cam:get(PhysicsComponents.Transform)
            if camTransform then
                anchorWindow() -- places below are window pixels / s
                local camPos = camTransform:getPos()
                local rot = CameraManager:getViewRot()
                local fwd, rgt, upv = rot:getForward(), rot:getRight(), rot:getUp()
                local tanHalf = math.tan(math.rad(CameraManager:getFov()) / 2)
                local G = UniverseScaleConfig.globalScale
                local cx, cy = W / 2 / s, H / 2 / s

                --- Toward a place on the screen from its middle: out to its
                --- projection, or (behind you) the side it is on; straight
                --- down when it's dead behind. The point `inset` inside the
                --- window's edge in that direction, and the direction.
                local function edgePoint(px, py, depth, side, lift, inset)
                    local ux, uy
                    if depth > 0 then ux, uy = px - cx, py - cy else ux, uy = side, -lift end
                    local len = math.sqrt(ux * ux + uy * uy)
                    if len < 1e-9 then ux, uy, len = 0, 1, 1 end
                    ux, uy = ux / len, uy / len
                    local t = math.min((cx - inset) / math.max(1e-9, math.abs(ux)),
                        (cy - inset) / math.max(1e-9, math.abs(uy)))
                    return cx + ux * t, cy + uy * t, ux, uy
                end

                --- The edge's glow toward x, y on it: a lit line along the
                --- edge, brightest at x, y, with a halo into the screen.
                local function edgeGlow(x, y, c, strength)
                    local vertical = math.abs(x - cx) > cx - 2
                    local steps = 9
                    for i = 1, steps do
                        local u0, u1 = -1 + 2 * (i - 1) / steps, -1 + 2 * i / steps
                        local um = (u0 + u1) / 2
                        local a = strength * math.exp(-(um / 0.5) ^ 2)
                        if vertical then
                            local ex = x < cx and 1.5 or 2 * cx - 1.5
                            line(c, a, 3, ex, clamp(y + u0 * EDGE.glow, 0, 2 * cy), ex, clamp(y + u1 * EDGE.glow, 0, 2 * cy), "cue")
                        else
                            local ey = y < cy and 1.5 or 2 * cy - 1.5
                            line(c, a, 3, clamp(x + u0 * EDGE.glow, 0, 2 * cx), ey, clamp(x + u1 * EDGE.glow, 0, 2 * cx), ey, "cue")
                        end
                    end
                end

                local offScreen = {} -- drawn after the glows, over them
                for _, c in ipairs(picture.objects) do
                    local o, r = c.object, c.reading
                    local isTarget = o == targetObject
                    -- sensed (a lost one only as the target's ghost); planets and stars only as the target
                    local sensed = c.world and (r.tier >= 1 or c.ghost) and (isTarget or not c.ghost)
                    if sensed and (isTarget or (o.kind ~= "planet" and o.kind ~= "star")) then
                        local dx, dy, dz = c.world.x - camPos.x, c.world.y - camPos.y, c.world.z - camPos.z
                        local depth = dx * fwd.x + dy * fwd.y + dz * fwd.z
                        local side = dx * rgt.x + dy * rgt.y + dz * rgt.z
                        local lift = dx * upv.x + dy * upv.y + dz * upv.z
                        local px, py, size
                        if depth > 0 then
                            px = cx + side / (depth * tanHalf) * cy -- the same scale both ways: square pixels
                            py = cy - lift / (depth * tanHalf) * cy
                            size = (o.radius or 0) * G / (depth * tanHalf) * cy
                        end
                        local onScreen = px and px > ON_SCREEN_MARGIN and px < 2 * cx - ON_SCREEN_MARGIN
                            and py > ON_SCREEN_MARGIN and py < 2 * cy - ON_SCREEN_MARGIN
                        local alpha = c.ghost and ghostAlpha(c) or 1
                        local shown = math.max(r.tier, c.known)
                        local col = contactColor(o, shown)
                        local small = onScreen and size < MARKER_MAX_SIZE and o.kind ~= "planet" and o.kind ~= "star"
                        if onScreen and isTarget then
                            -- brackets, its icon inside while it's small on screen, its
                            -- distance under them and its name over them once it's identified
                            local half, arm = math.max(BRACKET.half, size + 8), BRACKET.arm
                            for _, sgn in ipairs({ { -1, -1 }, { 1, -1 }, { 1, 1 }, { -1, 1 } }) do
                                local sx, sy = sgn[1], sgn[2]
                                polyline(style.pale, 0.85 * alpha, 1.4, {
                                    { px + sx * half, py + sy * (half - arm) },
                                    { px + sx * half, py + sy * half },
                                    { px + sx * (half - arm), py + sy * half },
                                }, "neon")
                            end
                            if small then
                                if shown <= 1 then
                                    blob(style.unknown, 0.9 * alpha, px, py, MARKER_BLOB)
                                else
                                    mapIcon(iconName(o), px, py, MARKER_ICON, col, (shown >= 3 and 1 or 0.8) * alpha,
                                        shown >= 3 and 1 or 0.5)
                                end
                            end
                            if shown >= 3 and o.name then
                                mapIcon(iconName(o), px - half + 5, py - half - 15, 11, col, 0.95 * alpha, 0.5)
                                text(o.name, px - half + 15, py - half - 10, 12, style.pale, 0.95 * alpha, "left")
                            end
                            text(SensorText.distance(c.distance), px, py + half + 17, 11, style.pale, 0.8 * alpha, "center")
                        elseif small then
                            if shown <= 1 then
                                blob(style.unknown, 0.9, px, py, MARKER_BLOB * 0.8)
                            else
                                local stale = c.stale and 0.6 or 1
                                mapIcon(iconName(o), px, py, MARKER_ICON, col, (shown >= 3 and 1 or 0.7) * stale,
                                    (shown >= 3 and 0.8 or 0.35) * stale)
                            end
                        elseif not onScreen then
                            offScreen[#offScreen + 1] = { c = c, o = o, shown = shown, col = col, alpha = alpha,
                                isTarget = isTarget, px = px, py = py, depth = depth, side = side, lift = lift }
                            -- toward an unknown or hostile contact, the edge glows: yellow
                            -- softly, red with a quicker pulse
                            local hostile = shown >= 2 and o.standing == "hostile" and o.kind ~= "wreck"
                            if not c.ghost and (shown <= 1 or hostile) then
                                local ex, ey = edgePoint(px, py, depth, side, lift, 0)
                                local pulse = hostile and 0.72 + 0.28 * math.sin(clock * 5) or 0.85 + 0.15 * math.sin(clock * 2.2)
                                edgeGlow(ex, ey, col, 0.95 * pulse)
                            end
                        end
                    end
                end

                for _, e in ipairs(offScreen) do
                    if e.isTarget then
                        -- the target: a white arrow at the edge, its distance beside it
                        local x, y, ux, uy = edgePoint(e.px, e.py, e.depth, e.side, e.lift, EDGE.arrow)
                        polyline(style.white, 0.95 * e.alpha, 1.5, {
                            { x + ux * 14, y + uy * 14 }, { x - ux * 6 - uy * 10, y - uy * 6 + ux * 10 },
                            { x - ux * 6 + uy * 10, y - uy * 6 - ux * 10 }, { x + ux * 14, y + uy * 14 },
                        }, "neon")
                        text(SensorText.distance(e.c.distance), x - ux * 32, y - uy * 32 + 4, 11, style.pale, 0.85 * e.alpha, "center")
                    elseif e.shown >= 2 and not e.c.ghost then
                        -- a classed contact: a chevron pointing out, its icon behind it
                        local x, y, ux, uy = edgePoint(e.px, e.py, e.depth, e.side, e.lift, EDGE.marker)
                        local tx, ty, k = x + ux * 10, y + uy * 10, 7
                        polyline(e.col, 0.9, 1.4, {
                            { tx - ux * k - uy * k, ty - uy * k + ux * k }, { tx, ty }, { tx - ux * k + uy * k, ty - uy * k - ux * k },
                        }, "neon")
                        mapIcon(iconName(e.o), x - ux * 12, y - uy * 12, EDGE.icon, e.col, e.c.stale and 0.55 or 0.9, 0.4)
                    end
                end
            end
        end

        ------------------------------------------------------------------
        -- The radar
        anchorBottomCentre()
        radarFan(style.hud, 0.04, 1, 0, 2 * math.pi, SEGMENTS.disc)
        radarCurve(style.hud, 0.9, 1.4, function() return 1 end, SEGMENTS.rim, "neon")
        radarCurve(style.hud, 0.26, 1, function() return 0.5 end, SEGMENTS.tier)

        do -- the cross through the disc
            local x0, y0 = onRadar(-1, 0)
            local x1, y1 = onRadar(1, 0)
            line(style.hud, 0.16, 1, x0, y0, x1, y1)
            x0, y0 = onRadar(0, -1)
            x1, y1 = onRadar(0, 1)
            line(style.hud, 0.16, 1, x0, y0, x1, y1)
        end

        do -- about what's in view: a wedge as wide as the camera's view, ahead
            local half = math.atan(math.tan(math.rad(CameraManager:getFov()) / 2) * W / H)
            radarFan(style.cyan, 0.06, 1, -half, half, 6)
        end

        do -- the ship, nose up
            local nx, ny = onRadar(0, 0.1 * shrink)
            local lx, ly = onRadar(-0.065 * shrink, -0.06 * shrink)
            local rx, ry = onRadar(0.065 * shrink, -0.06 * shrink)
            triangle(style.pale, 0.95, nx, ny, rx, ry, lx, ly)
        end

        if target then -- where the target would turn T2, T3 and T4 in open space (T1: the signal ring)
            local g = target.ranges
            for tier, d in pairs({ [2] = g.classify, [3] = g.identify, [4] = g.scan4 }) do
                if d > RADAR_MIN and d <= RADAR_MAX then
                    local r = radarRadius(d)
                    radarCurve(style.hud, TIER_RING_ALPHA[tier], 1, function() return r end, SEGMENTS.tier, "line", nil, nil, { 2, 5 })
                end
            end
        end

        do -- how far you're heard: your power draw, and further (white) while a scan or a pulse shouts
            local phase = -clock * 10
            local r = radarRadius(picture.heardWithin)
            radarCurve(style.pale, 0.4, 1.1, function() return r end, SEGMENTS.rim, "line", nil, nil, { 5, 5, phase })
            local shout = picture.scanning and 1 or math.max(0, math.min(1, PULSE_SHOUT_TIME - picture.pulseAge))
            if shout > 0 then
                local rs = radarRadius(picture.pulseHeardWithin)
                radarCurve(style.white, 0.75 * shout, 1.5, function() return rs end, SEGMENTS.rim, "line", nil, nil, { 5, 5, phase })
            end
        end

        if picture.heat > 0.005 then -- your heat, round your ship
            local col = picture.bloom and style.white or style.pale
            local glow = picture.bloom and (0.7 + 0.3 * math.sin(clock * 6)) or 0.8
            local hr = HEAT_RING * shrink
            radarCurve(col, 0.18, 2.4, function() return hr end, SEGMENTS.heat)
            radarCurve(col, glow, 2, function() return hr end, SEGMENTS.heat, "neon", -math.pi,
                -math.pi + picture.heat * 2 * math.pi)
        end

        if picture.pulseAge < PULSE_RING_TIME then
            local r = math.min(1, picture.pulseAge)
            radarCurve(style.white, 0.9 * (1 - picture.pulseAge / PULSE_RING_TIME), 1.4, function() return r end,
                SEGMENTS.rim, "neon")
        end

        do -- where signals come from: a bulge per heard object, as wide as the sensor is coarse
            local width = 2 + 12 * math.sqrt(picture.dials.resolution)
            local bumps = {}
            for _, c in ipairs(picture.objects) do
                local p = c.reading.passive
                if not p.surroundings.blocked and p.snr > 0.05 then
                    bumps[#bumps + 1] = { c.bearing, clamp(math.log10(1 + p.snr) / 4, 0, 1) }
                end
            end
            local ring, bulge = 1 + (SIGNAL_RING - 1) * shrink, SIGNAL_BULGE * shrink
            local function radiusAt(a)
                local deg, m = a / DEG, 0
                for _, b in ipairs(bumps) do
                    local x = wrapDegrees(deg - b[1]) / width
                    m = math.max(m, b[2] * math.exp(-x * x))
                end
                return ring + bulge * m
            end
            radarCurve(style.hud, 0.2, 1, function() return ring end, SEGMENTS.tier)
            radarCurve(style.cyan, 0.75, 1, radiusAt, SEGMENTS.signal, "neon")
        end

        -- Contacts on their sticks, who heard your pulse, the ring on the target and your scan.
        -- A contact sits where its direction, scaled to its distance on the log
        -- scale, puts it: its foot on the disc at its bearing, a stick up to it
        -- when it's above your plane, a dashed one down when it's below.
        for _, c in ipairs(picture.objects) do
            local r = c.reading
            if r.tier >= 1 or c.ghost then
                local shown = math.max(r.tier, c.known)
                local col = contactColor(c.object, shown)
                local alpha = c.ghost and ghostAlpha(c) or 1
                local rr, el, b = radarRadius(c.distance), c.elevation * DEG, c.bearing * DEG
                local flat, h = rr * math.cos(el), rr * math.sin(el)
                local fx, fy = onRadar(flat * math.sin(b), flat * math.cos(b))
                local x, y = onRadar(flat * math.sin(b), flat * math.cos(b), h)
                if math.abs(y - fy) > 2 then
                    line(col, (h < 0 and 0.4 or 0.55) * alpha, 1, fx, fy, x, y, "line", 0, h < 0 and { 2, 2 } or nil)
                end
                round("circle", col, 0.45 * alpha, fx, fy, 1.4)

                if c.ghost then
                    radarDot(c.object, x, y, shown, "ghost", alpha)
                elseif shown <= 1 then
                    -- heard: full; only a pulse echo: fading out over its last seconds
                    local fade = r.passive.snr >= 1 and 1 or clamp(c.echo / ECHO_FADE_TIME, 0.25, 1)
                    blob(style.unknown, fade, x + 1.5 * math.sin(clock * 1.7 + c.index * 2.1),
                        y + 1 * math.cos(clock * 1.3 + c.index),
                        (r.count > 1 and MERGED_RADAR_BLOB or RADAR_BLOB) * grow)
                else
                    -- a dot, or focused its map icon: one fades into the other on the way
                    local mode = c.stale and "stale" or nil
                    local dot, iconAlpha = 1 - fe, fe
                    if dot > 0 then radarDot(c.object, x, y, shown, mode, dot) end
                    if iconAlpha > 0 then
                        mapIcon(iconName(c.object), x, y, radarIconSize(c.object), col,
                            (mode and 0.55 or 0.95) * (c.object.kind == "wreck" and 0.6 or 1) * iconAlpha,
                            (mode and 0.2 or 0.5) * iconAlpha)
                    end
                end
                if c.stale or c.ghost then -- how old what it knows is: the arc closes as it ages
                    arc(style.pale, 0.6 * alpha, 1, x, y, 6 * grow, 0, 360 * math.min(1, c.age / STALE_FULL))
                end
                if c.heardPulse then
                    icon("ping", x, y - 12 * grow, 7 * math.sqrt(grow), style.white,
                        0.95 * math.min(1, PULSE_SHOUT_TIME - picture.pulseAge))
                end
                if c.object == targetObject then
                    arc(style.white, 0.9 * alpha, 1, x, y, 7.5 * grow, 0, 360, "neon")
                    if picture.scanning and r.tier >= 1 then
                        local cx, cy = onRadar(0, 0)
                        line(style.white, 0.8, 1.3, cx, cy, x, y, "line", 0, { 3, 5, -clock * 30 })
                    end
                end
            end
        end

        do -- the rim's distance under the disc (on the screen), and focused the middle ring's
            local lx, ly = onRadar(0, -1.18)
            text(SensorText.distance(RADAR_MAX), lx, math.min(ly + 13, FOCUS.rimLabelLowest), 10, style.hud, 0.85, "center")
            if fe > 0 then
                local mx, my = onRadar(0.5, 0)
                local middle = 10 ^ ((math.log10(RADAR_MIN) + math.log10(RADAR_MAX)) / 2)
                text(SensorText.distance(middle), mx + 4, my - 4, 10, style.hud, 0.75 * fe, "left")
            end
        end

        if focus.key then
            -- the focus button right of the rim, lit while focused: a small
            -- radar disc that flips up with the radar, a contact on its stick,
            -- and the key under it
            local rx = onRadar(1, 0)
            local _, nearY = onRadar(0, -1)
            local bx, by, br = rx + FOCUS.button.gap, nearY - 6, FOCUS.button.r
            local c = focus.on and style.white or style.hud
            round("circle", style.hud, focus.on and 0.25 or 0.08, bx, by, br)
            arc(c, focus.on and 0.95 or 0.6, 1.3, bx, by, br, 0, 360, "neon")
            local flat, disc = 2.6 + 2 * fe, {}
            for i = 0, 16 do
                local a = i / 16 * 2 * math.pi
                disc[#disc + 1] = { bx + 6 * math.cos(a), by + 2 + flat * math.sin(a) }
            end
            polyline(c, 0.9, 1.2, disc)
            line(c, 0.9, 1.2, bx + 2, by + 1, bx + 2, by - 5)
            round("circle", c, 0.95, bx + 2, by - 5.5, 1.5)
            local kw, kh = FOCUS.keycap.w, FOCUS.keycap.h -- the key, on a keycap under the button
            local ky = by + br + 6
            polyline(style.hud, 0.6, 1, { { bx - kw / 2, ky }, { bx + kw / 2, ky }, { bx + kw / 2, ky + kh },
                { bx - kw / 2, ky + kh }, { bx - kw / 2, ky } })
            text(focus.key, bx, ky + kh + 4, 18, style.pale, 0.9, "center") -- low: a backtick sits high on its line
        end

        ------------------------------------------------------------------
        -- The target panel
        anchorBottomRight()
        do
            local x, y, R = PANEL.x, PANEL.y, PANEL.r
            local contact = target and target.contact
            local tier = contact and contact.reading.tier or 0
            local o = contact and contact.object
            -- What it shows is the most the sensor has known of it. When it senses
            -- less now, what it is stays (shape, colour, name) but the details are
            -- as last seen: grey, with their age running round inside the ring.
            -- Lost, all of it is a memory: grey, with scan lines, a dashed ring.
            local known = contact and contact.known or 0
            local ghost = known >= 2 and tier < 1
            local stale = ghost or (contact ~= nil and contact.stale)
            local shown = math.max(tier, known)
            local read = target and math.max(target.read, shown >= 3 and 3 or 0) or 0
            round("circle", style.hud, 0.06, x, y, R)

            if shown < 1 then
                -- no contact (or no target): static, and a dashed ring that
                -- flashes while a scan has nothing to aim at
                local scanningNothing = picture.scanning and target ~= nil
                arc(scanningNothing and style.white or style.hud,
                    scanningNothing and (0.6 + 0.3 * math.sin(clock * 8)) or 0.35, 1.2, x, y, R, 0, 360, "line", { 3, 7 })
                staticDots(x, y, R, 40, 0.55)
            else
                local col = contactColor(o, shown)
                -- the ring: once its shields are known, it glows as strongly as they are
                local shield = read >= 3 and o.shield or nil
                if stale then
                    arc(style.pale, 0.5, 2, x, y, R - 7, 0, 360 * math.min(1, contact.age / STALE_FULL))
                end
                if ghost then
                    arc(style.pale, 0.6, 1.3, x, y, R, 0, 360, "line", { 5, 4 })
                elseif shield and not stale then
                    arc(style.cyan, 0.18 + 0.82 * shield ^ 1.1, 1.2 + 0.8 * shield, x, y, R, 0, 360, "neon")
                    Gui:setStrokeLookParam("glow", 0.25 + 0.6 * shield)
                    Gui:setStrokeLookParam("glowWidth", (4 + 14 * shield) * s)
                else
                    arc(style.hud, 0.6, 1.1, x, y, R, 0, 360, "neon")
                end

                -- what it is, as far as it's known
                if shown == 1 then
                    staticDots(x, y, R, 24, 0.3)
                    blob(style.unknown, 1, x + 3 * math.sin(clock * 1.7), y + 2 * math.cos(clock * 1.3),
                        contact.reading.count > 1 and MERGED_PANEL_BLOB or PANEL_BLOB)
                elseif read < 3 then
                    mapIcon(iconName(o), x, y, PANEL_ICON, col, ghost and 0.5 or stale and 0.65 or 0.95, stale and 0.3 or 0.9)
                else
                    -- the hologram's stand-in: a wireframe of what it is, in its colour, brighter at T4
                    local class = "class" .. math.max(1, SensorModel.shipClass(o))
                    local sil = SILHOUETTES[o.kind] or SILHOUETTES[class] or SILHOUETTES.class2
                    local hc = ghost and style.pale or col
                    local size, bright = R * 0.62, ghost and 0.5 or stale and 0.7 or read >= 4 and 1 or 0.85
                    if sil.round then
                        round("circle", hc, 0.16 * bright, x, y, sil.round * size)
                        round("ring", hc, bright, x, y, sil.round * size)
                    end
                    if sil.outline then shape(sil.outline, x, y, size, hc, bright, 1.6, 0.16 * bright, "neon") end
                    for _, l in ipairs(sil.lines or {}) do
                        line(hc, 0.6 * bright, 1.1, x + l[1] * size, y + l[2] * size, x + l[3] * size, y + l[4] * size)
                    end
                    if sil.rays then
                        for d = 0, 315, 45 do
                            local a = d * DEG
                            line(hc, 0.6 * bright, 1.1, x + math.cos(a) * 0.65 * size, y + math.sin(a) * 0.65 * size,
                                x + math.cos(a) * 0.95 * size, y + math.sin(a) * 0.95 * size)
                        end
                    end
                    if read >= 4 and not stale then round("circle", style.core, 0.9, x, y, 5, true) end

                    -- its power draw: five bars on a log scale (1 MW to 10 GW), or a struck-out bolt with none
                    local bx, by = x - 22, y + R - 26
                    if (o.mw or 0) > 0 then
                        icon("bolt", bx - 12, by - 4, 6, style.pale, stale and 0.5 or 0.9)
                        local lit = clamp(math.floor(math.log10(o.mw) * 1.25 + 0.5), 1, 5)
                        for i = 0, 4 do
                            rect(i < lit and (stale and style.pale or style.cyan) or style.hud,
                                i < lit and (stale and 0.45 or 0.9) or 0.2, bx + i * 9, by - 3 - i * 2, 6, 6 + i * 2)
                        end
                    elseif o.kind == "ship" or o.kind == "wreck" then
                        icon("bolt", bx - 12, by - 4, 6, style.pale, 0.55)
                        line(style.pale, 0.7, 1.4, bx - 19, by + 3, bx - 5, by - 11)
                    end
                    local act = o.kind == "ship" and ({ thrust = "flame", firing = "aim", cloak = "eyeOff" })[o.activity]
                    -- what it was doing: bright only while it's seen doing it
                    if act then icon(act, x + R * 0.62, y - R * 0.62, 8, stale and style.pale or style.white, stale and 0.5 or 0.95) end
                end
                if ghost then -- scan lines over the remembered picture
                    for k = -4, 4 do
                        local dy = k * R * 0.2
                        local half = math.sqrt(R * R - dy * dy) * 0.92
                        line(style.pale, 0.1, 1, x - half, y + dy, x + half, y + dy)
                    end
                end

                -- the name (once it's known) with its icon, and the distance, where
                -- the HUD vision's panel has its title
                if read >= 3 and o.name then
                    local w = textWidth(o.name, 13)
                    mapIcon(iconName(o), x - w / 2 - 12, y - R - 44, 13, col, ghost and 0.6 or 0.95, 0.5)
                    text(o.name, x, y - R - 40, 13, style.core, ghost and 0.6 or 0.95, "center")
                end
                text(SensorText.distance(contact.distance), x, y - R - 22, 12, style.pale, 0.8, "center")

                -- the scan: progress round the rim, the echo in a meter under it (a notch for T3 and one for T4)
                if target.reading then
                    arc(style.white, 0.95, 2.2, x, y, R + 7, 0, target.progress * 360, "neon")
                end
                local MR, m = R + ECHO_METER.gap, ECHO_METER
                local function at(snr) return m.from + m.sweep - m.sweep * clamp(math.log10(math.max(1, snr)) / 6, 0, 1) end
                arc(style.white, 0.14, 4, x, y, MR, m.from, m.sweep)
                local fillFrom = at(target.echo.snr)
                arc(style.white, target.echo.level > 0 and 0.9 or 0.55, 4, x, y, MR, fillFrom, m.from + m.sweep - fillFrom)
                for _, notch in ipairs({ { SensorModel.constants.scanT3Snr, 3 }, { SensorModel.constants.scanT4Snr, 4 } }) do
                    local a = at(notch[1]) * DEG
                    local sa, ca = math.sin(a), -math.cos(a)
                    line(style.pale, 0.8, 1.4, x + sa * (MR - 6), y + ca * (MR - 6), x + sa * (MR + 6), y + ca * (MR + 6))
                    tierIcon(notch[2], x + sa * (MR + 15), y + ca * (MR + 15))
                end
                -- too weak to read what's left: get closer
                if picture.scanning and not target.reading and target.read < 4 then
                    icon("chevrons", x, y + MR + 24, 9, style.white, 0.5 + 0.45 * math.sin(clock * 6))
                end
                -- the target hears the scan
                if picture.scanning and tier >= 1 and (o.kind == "ship" or o.kind == "station") then
                    for k = 0, 1 do
                        local ph = (clock * 0.8 + k * 0.5) % 1
                        arc(style.white, 0.5 * (1 - ph), 1.2, x, y, R + 22 + ph * 26, -54, 108)
                    end
                end

                -- what a full scan read, left of the panel: an icon each, and only the words that matter
                if target.read >= 4 then
                    for i, contentLine in ipairs(o.contents or {}) do
                        local kind = contentIcon(contentLine)
                        local words = kind and contentLine:gsub("^[^:]*:%s*", "") or contentLine
                        local ly, lx = y - 18 + (i - 1) * 24, x - R - 22
                        if kind then icon(kind, lx, ly - 4, 7, style.pale, stale and 0.6 or 0.95) end
                        text(words, lx - 14, ly, 13, style.pale, stale and 0.6 or 0.95, "right")
                    end
                end
            end
        end

        Gui:endContainer()
        Gui:setPosition(0, 0)
        Gui:setPercentSize(100, 100)
    end

    return hud
end

UIComponent.SensorHud = setmetatable({ style = STYLE }, { __call = new })

return UIComponent.SensorHud
