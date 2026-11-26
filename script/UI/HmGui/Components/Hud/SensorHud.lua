-- Sensor HUD: what a ship's sensor shows, with words only where nothing else
-- can say it (a target's name, its distance, what a full scan reads).
--
-- The radar, at the bottom centre: a disc tilted away from the viewer with the
-- ship in the middle, nose up, distance on a log scale (50 m at the centre,
-- 100 km at the rim). A contact's shape says what it is, its fill how much is
-- known, its colour whose side it's on:
--   shape   ships and stations by their class icon (I circle, II triangle,
--           III triangle down, IV hexagon); asteroids a dot; planets a ring in
--           a ring; stars a four-pointed star; wrecks their class icon, broken
--   fill    T1 a soft blob (no shape yet), T2 an outline, T3 filled, T4
--           filled with a bright core
--   colour  pale until T3, then red for hostile
-- Rings show where the target would turn T1 to T4 in open space, each with
-- that tier's icon. A ring round the disc bulges where signals come from. How
-- far the ship is heard is a dashed ring (amber, further out, while it scans or
-- pulses); its heat an arc round it (amber from the thermal bloom); ripples over
-- a contact mean it heard the last pulse. Amber means "look here" and nothing
-- else.
--
-- The target panel, at the bottom right: a hologram circle that fills in as
-- the sensor learns more, like the HUD vision's target panel. Static without a
-- contact, a blob at T1, the class icon at T2, a wireframe of it with its
-- shields (the ring's glow), power and activity at T3, brighter at T4 with
-- what the full scan read beside it. A scan's progress runs round the rim; the
-- echo fills a meter under it with a notch for T3 and one for T4; chevrons ask
-- to get closer when it's too weak to read more; ripples show the target hears
-- the scan.
--
--   local hud = UIComponent.SensorHud { visible = function() return shown end }
--   -- every frame, before rendering:
--   hud:update(SensorPicture.build(sensor, forward, right), dt)
--
-- Places and sizes are pixels on a 1920 x 1080 screen and scale with the
-- window height. The radar keeps its place relative to the bottom centre of
-- the screen, the target panel relative to the bottom right corner.

local SensorModel = require("Modules.Sensors.SensorModel")
local SensorText = require("Modules.Sensors.SensorText")

---@class UIComponentSensorHud: UIComponent
---@field state UIComponentState
---@field update fun(self: UIComponentSensorHud, picture: SensorPictureData|nil, dt: number|nil) takes this frame's picture; nil hides the HUD
---@field render fun(self: UIComponentSensorHud) renders the radar and the target panel

---@class UIComponentSensorHudConstructor
---@field visible boolean|(fun(): boolean)|nil
---@field scale number|(fun(): number)|nil sizes relative to 1080p; default window height / 1080

local DEG = math.pi / 180

-- Geometry, in pixels on a 1920 x 1080 screen.
local RADAR = { x = 960, y = 930, r = 132, tilt = 40 * DEG, depth = 7 } -- depth: perspective, in radii
local PANEL = { x = 1720, y = 858, r = 128 }                            -- the HUD vision's target panel
local RADAR_MIN, RADAR_MAX = 50, 1e5 -- metres at the radar's centre and at its rim
local SIGNAL_RING = 1.1              -- the signal ring's radius, in radar radii
local SIGNAL_BULGE = 0.3             -- how far a strong signal bulges it out
local HEAT_RING = 0.17               -- the heat arc round the ship, in radar radii
local SWEEP_SPEED = 1.4              -- radians per second
local SWEEP_STEPS, SWEEP_STEP = 14, 0.06
local PULSE_RING_TIME = 1.2          -- seconds the pulse ring takes to reach the rim and fade
local PULSE_SHOUT_TIME = 4           -- seconds the pulse's footprint stays up
local ECHO_FADE_TIME = 5             -- a blob seen by its pulse echo alone fades over its last seconds
-- Strokes per closed curve on the radar; each is bent to follow the curve
local SEGMENTS = { rim = 24, tier = 20, signal = 60, heat = 16 }
local GLYPH_SIZE = 5.5
local BLOB_RADIUS, MERGED_BLOB_RADIUS = 13, 17
local ECHO_METER = { from = 158.4, sweep = 43.2, gap = 16 } -- the echo meter's arc under the panel (degrees, 0 = up)
local FONT, FONT_MIN = "UbuntuBold", 9

local function rgb(r, g, b) return { r / 255, g / 255, b / 255 } end

--- Colours ("HUD blue" and friends) and looks; public as `UIComponent.SensorHud.style`.
local STYLE = {
    hud = rgb(26, 153, 255),
    cyan = rgb(51, 230, 255),
    pale = rgb(190, 225, 255),
    core = rgb(240, 250, 255),
    amber = rgb(255, 170, 51),
    hostile = rgb(255, 77, 77),
    halo = Color(0, 0.02, 0.04, 0.6),
    -- neon: the rims, rings and the hologram's outline, with a soft glow;
    -- line: thin rings, icons and glyph outlines. Both are alpha blended: a
    -- curve is many strokes whose round ends overlap, and added light would
    -- make bright beads there.
    looks = {
        neon = { core = 0.9, coreWidth = 0.7, glow = 0.3, glowWidth = 3, shade = 1 },
        line = { core = 1, coreWidth = 0.6, glow = 0, shade = 1 },
    },
}
local SCALED_LOOK_PARAMS = { coreWidth = true, glowWidth = true, haloWidth = true }
local NO_OUTLINE = Color(0, 0, 0, 0)

local hudCount = 0 -- each HUD defines its looks under its own names

local function clamp(x, lo, hi) return math.min(hi, math.max(lo, x)) end

local function wrapDegrees(a) return (a + 540) % 360 - 180 end

local function color(c, alpha) return Color(c[1], c[2], c[3], math.max(0, alpha or 1)) end

--- Where a distance sits on the radar, 0 (centre) to 1 (rim).
local function radarRadius(d)
    local lo, hi = math.log10(RADAR_MIN), math.log10(RADAR_MAX)
    return clamp((math.log10(math.max(1, d)) - lo) / (hi - lo), 0, 1)
end

--- A point on the radar's plane (x right, y ahead, in radar radii) in screen
--- pixels at 1080p, through the disc's tilt and perspective.
local function onRadar(x, y)
    local ct, st = math.cos(RADAR.tilt), math.sin(RADAR.tilt)
    local Y, Z = y * ct, -y * st
    local p = RADAR.depth / (RADAR.depth - Z)
    return RADAR.x + RADAR.r * x * p, RADAR.y - RADAR.r * Y * p
end

local function polar(r, bearing)
    return onRadar(r * math.sin(bearing * DEG), r * math.cos(bearing * DEG))
end

--- What a contact is, as a shape: "class1" to "class4", "asteroid", "planet"
--- or "star"; and whether it's broken (a wreck).
local function glyphKind(o)
    if o.kind == "asteroid" or o.kind == "planet" or o.kind == "star" then return o.kind, false end
    return "class" .. math.max(1, SensorModel.shipClass(o)), o.kind == "wreck"
end

--- Outlines in glyph radii (y down) for the shapes with corners.
local GLYPH_SHAPES = {
    class2 = { { 0, -1 }, { 0.95, 0.75 }, { -0.95, 0.75 } },
    class3 = { { 0, 1 }, { 0.95, -0.75 }, { -0.95, -0.75 } },
    class4 = (function()
        local hexagon = {}
        for i = 0, 5 do
            local a = (i * 60 + 30) * DEG
            hexagon[#hexagon + 1] = { math.cos(a), math.sin(a) }
        end
        return hexagon
    end)(),
    star = { { 0, -1.3 }, { 0.3, -0.3 }, { 1.3, 0 }, { 0.3, 0.3 }, { 0, 1.3 }, { -0.3, 0.3 }, { -1.3, 0 }, { -0.3, -0.3 } },
}

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
for i, p in ipairs(GLYPH_SHAPES.class4) do SILHOUETTES.station.outline[i] = { 0.9 * p[1], 0.9 * p[2] } end

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

    function hud:update(newPicture, dt)
        picture = newPicture
        clock = clock + (dt or 0)
    end

    function hud:render()
        if not picture or not self.state.visible() then return end

        local style = UIComponent.SensorHud.style
        local s = self.state.scale()
        defineLooks(s)
        local W, H = Window:width(), Window:height()

        -- 1080p places to window pixels: the radar from the bottom centre,
        -- the target panel from the bottom right.
        local ax, ay -- this anchor's offsets
        local function anchorBottomCentre() ax, ay = W / 2 - 960 * s, H - 1080 * s end
        local function anchorBottomRight() ax, ay = W - 1920 * s, H - 1080 * s end
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

        --- Text with a dark halo, its baseline at y; align "left", "right" or "center".
        local function text(str, x, y, size, c, alpha, align)
            local font = Cache.Font(FONT, math.max(FONT_MIN, math.floor(size * s + 0.5)))
            local w, h = 700 * s, size * 1.6 * s
            local left = X(x)
            if align == "right" then left = left - w elseif align == "center" then left = left - w / 2 end
            Gui:beginStackContainer()
            Gui:setClipping(false)
            Gui:setFixedSize(w, h)
            Gui:setPosition(left, Y(y) - size * 1.15 * s)
            local halo = style.halo -- fainter text, fainter halo
            Gui:textOutlined(str, font, color(c, alpha), NO_OUTLINE, Color(halo.r, halo.g, halo.b, halo.a * alpha))
            Gui:setAlignment(
                align == "right" and AlignHorizontal.Right or align == "center" and AlignHorizontal.Center or AlignHorizontal.Left,
                AlignVertical.Center)
            Gui:endContainer()
        end

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

        --- A contact: its shape, filled by how much is known, coloured by its
        --- standing once that's known (T3).
        local function glyph(o, x, y, size, tier)
            local kind, broken = glyphKind(o)
            local c = (tier >= 3 and o.standing == "hostile") and style.hostile or style.pale
            local fill = tier >= 3 and (broken and 0.3 or 0.55) or 0
            local dash = broken and { 2.5, 2 } or nil
            if kind == "asteroid" then
                round("circle", c, tier >= 3 and 0.9 or 0.6, x, y, size * 0.45)
            elseif kind == "planet" then
                if tier >= 3 then round("circle", c, 0.5, x, y, size * 0.5) end
                round("ring", c, 0.95, x, y, size * 1.1)
                round("ring", c, 0.95, x, y, size * 0.5)
            elseif kind == "class1" then
                if fill > 0 then round("circle", c, fill, x, y, size * 0.9) end
                if dash then arc(c, 0.95, 1.4, x, y, size * 0.9, 0, 360, "line", dash) else round("ring", c, 0.95, x, y, size * 0.9) end
            else
                shape(GLYPH_SHAPES[kind], x, y, size, c, 0.95, 1.4, fill, "line", dash)
            end
            if tier >= 4 then round("circle", style.core, 1, x, y, size * 0.3, true) end
        end

        --- A tier's own look, for the range rings and the echo meter's notches.
        local function tierIcon(tier, x, y)
            if tier == 1 then blob(style.cyan, 1, x, y, 9) return end
            shape({ { 0, -1 }, { 0.92, 0.7 }, { -0.92, 0.7 } }, x, y, 5, style.pale, 0.95, 1.3, tier >= 3 and 0.8 or 0)
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
        -- The radar
        anchorBottomCentre()
        radarCurve(style.hud, 0.9, 1.6, function() return 1 end, SEGMENTS.rim, "neon")

        if target then
            local g = target.ranges
            for tier, d in ipairs({ g.detect, g.classify, g.identify, g.scan4 }) do
                if d > RADAR_MIN and d <= RADAR_MAX then
                    local r = radarRadius(d)
                    radarCurve(tier == 4 and style.pale or style.cyan, 0.32, 1, function() return r end, SEGMENTS.tier)
                    local lx, ly = polar(r, 180)
                    tierIcon(tier, lx, ly + 1)
                end
            end
        end

        do -- the cross through the disc
            local x0, y0 = onRadar(-1, 0)
            local x1, y1 = onRadar(1, 0)
            line(style.hud, 0.18, 1, x0, y0, x1, y1)
            x0, y0 = onRadar(0, -1)
            x1, y1 = onRadar(0, 1)
            line(style.hud, 0.18, 1, x0, y0, x1, y1)
        end

        do -- the sweep
            local cx, cy = onRadar(0, 0)
            local sweep = (clock * SWEEP_SPEED) % (2 * math.pi)
            for j = 0, SWEEP_STEPS - 1 do
                local a1 = sweep - j * SWEEP_STEP
                local a0 = a1 - SWEEP_STEP
                local x1, y1 = onRadar(math.sin(a1), math.cos(a1))
                local x0, y0 = onRadar(math.sin(a0), math.cos(a0))
                triangle(style.cyan, 0.07 * (1 - j / SWEEP_STEPS), cx, cy, x1, y1, x0, y0)
            end
        end

        do -- the ship, nose up
            local nx, ny = onRadar(0, 0.09)
            local lx, ly = onRadar(-0.06, -0.06)
            local rx, ry = onRadar(0.06, -0.06)
            triangle(style.pale, 0.95, nx, ny, rx, ry, lx, ly)
        end

        do -- how far you're heard: your power draw, and further while a scan or a pulse shouts
            local phase = -clock * 12
            local r = radarRadius(picture.heardWithin)
            radarCurve(style.pale, 0.45, 1.2, function() return r end, SEGMENTS.rim, "line", nil, nil, { 2, 5, phase })
            local shout = picture.scanning and 1 or math.max(0, math.min(1, PULSE_SHOUT_TIME - picture.pulseAge))
            if shout > 0 then
                local rs = radarRadius(picture.pulseHeardWithin)
                radarCurve(style.amber, 0.75 * shout, 1.2, function() return rs end, SEGMENTS.rim, "line", nil, nil, { 6, 4, phase })
            end
        end

        if picture.heat > 0.005 then -- your heat, round your ship
            local col = picture.bloom and style.amber or style.pale
            local glow = picture.bloom and (0.7 + 0.3 * math.sin(clock * 6)) or 0.8
            radarCurve(col, 0.18, 2.4, function() return HEAT_RING end, SEGMENTS.heat)
            radarCurve(col, glow, 2, function() return HEAT_RING end, SEGMENTS.heat, "neon", -math.pi,
                -math.pi + picture.heat * 2 * math.pi)
        end

        if picture.pulseAge < PULSE_RING_TIME then
            local r = math.min(1, picture.pulseAge)
            radarCurve(style.amber, 0.9 * (1 - picture.pulseAge / PULSE_RING_TIME), 1.4, function() return r end,
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
            local function radiusAt(a)
                local deg, m = a / DEG, 0
                for _, b in ipairs(bumps) do
                    local x = wrapDegrees(deg - b[1]) / width
                    m = math.max(m, b[2] * math.exp(-x * x))
                end
                return SIGNAL_RING + SIGNAL_BULGE * m
            end
            radarCurve(style.cyan, 0.18, 1, function() return SIGNAL_RING end, SEGMENTS.tier)
            radarCurve(style.cyan, 0.7, 1.2, radiusAt, SEGMENTS.signal, "neon")
        end

        -- Contacts, who heard your pulse, and the target's corners and beam
        for _, c in ipairs(picture.objects) do
            local r = c.reading
            local x, y = polar(radarRadius(c.distance), c.bearing)
            if r.tier == 1 then
                -- heard: full; only a pulse echo: fading out over its last seconds
                local fade = r.passive.snr >= 1 and 1 or clamp(c.echo / ECHO_FADE_TIME, 0.25, 1)
                blob(style.cyan, fade, x + 2.5 * math.sin(clock * 1.7 + c.index * 2.1),
                    y + 1.5 * math.cos(clock * 1.3 + c.index), r.count > 1 and MERGED_BLOB_RADIUS or BLOB_RADIUS)
            elseif r.tier >= 2 then
                glyph(c.object, x, y, GLYPH_SIZE, r.tier)
            end
            if c.heardPulse then
                icon("ping", x, y - 15, 9, style.amber, 0.95 * math.min(1, PULSE_SHOUT_TIME - picture.pulseAge))
            end
            if c.object == targetObject then
                local half, len = 11, 4
                for _, sgn in ipairs({ { -1, -1 }, { 1, -1 }, { 1, 1 }, { -1, 1 } }) do
                    local sx, sy = sgn[1], sgn[2]
                    polyline(style.cyan, r.tier >= 1 and 0.95 or 0.35, 1.6, {
                        { x + sx * half, y + sy * (half - len) },
                        { x + sx * half, y + sy * half },
                        { x + sx * (half - len), y + sy * half },
                    })
                end
                if picture.scanning and r.tier >= 1 then
                    local cx, cy = onRadar(0, 0)
                    line(style.amber, 0.8, 1.4, cx, cy, x, y, "line", 0, { 3, 5, -clock * 30 })
                end
            end
        end

        ------------------------------------------------------------------
        -- The target panel
        anchorBottomRight()
        do
            local x, y, R = PANEL.x, PANEL.y, PANEL.r
            local contact = target and target.contact
            local tier = contact and contact.reading.tier or 0
            local o = contact and contact.object
            local read = target and math.max(target.read, tier >= 3 and 3 or 0) or 0
            round("circle", style.hud, 0.06, x, y, R)

            if tier < 1 then
                -- no contact (or no target): static, and a dashed ring that
                -- flashes while a scan has nothing to aim at
                local scanningNothing = picture.scanning and target ~= nil
                arc(scanningNothing and style.amber or style.hud,
                    scanningNothing and (0.6 + 0.3 * math.sin(clock * 8)) or 0.45, 1.2, x, y, R, 0, 360, "line", { 4, 6 })
                staticDots(x, y, R, 40, 0.55)
            else
                -- the ring: once its shields are known, it glows as strongly as they are
                local shield = read >= 3 and o.shield or nil
                if shield then
                    arc(style.cyan, 0.18 + 0.82 * shield ^ 1.1, 1.2 + 0.8 * shield, x, y, R, 0, 360, "neon")
                    Gui:setStrokeLookParam("glow", 0.25 + 0.6 * shield)
                    Gui:setStrokeLookParam("glowWidth", (4 + 14 * shield) * s)
                else
                    arc(style.hud, 0.85, 1.4, x, y, R, 0, 360, "neon")
                end

                -- what it is, as far as it's known
                if tier == 1 then
                    staticDots(x, y, R, 24, 0.35)
                    blob(style.cyan, 1, x + 3 * math.sin(clock * 1.7), y + 2 * math.cos(clock * 1.3),
                        contact.reading.count > 1 and 46 or 36)
                elseif read < 3 then
                    glyph(o, x, y, 34, 2)
                else
                    -- the hologram's stand-in: a wireframe of what it is, brighter at T4
                    local kind = glyphKind(o)
                    local sil = SILHOUETTES[o.kind] or SILHOUETTES[kind] or SILHOUETTES.class2
                    local col = (o.standing == "hostile") and style.hostile or style.cyan
                    local size, bright = R * 0.62, read >= 4 and 1 or 0.85
                    if sil.round then
                        round("circle", col, 0.16 * bright, x, y, sil.round * size)
                        round("ring", col, bright, x, y, sil.round * size)
                    end
                    if sil.outline then shape(sil.outline, x, y, size, col, bright, 1.6, 0.16 * bright, "neon") end
                    for _, l in ipairs(sil.lines or {}) do
                        line(col, 0.6 * bright, 1.1, x + l[1] * size, y + l[2] * size, x + l[3] * size, y + l[4] * size)
                    end
                    if sil.rays then
                        for d = 0, 315, 45 do
                            local a = d * DEG
                            line(col, 0.6 * bright, 1.1, x + math.cos(a) * 0.65 * size, y + math.sin(a) * 0.65 * size,
                                x + math.cos(a) * 0.95 * size, y + math.sin(a) * 0.95 * size)
                        end
                    end
                    if read >= 4 then round("circle", style.core, 0.9, x, y, 5, true) end

                    -- its power draw: five bars on a log scale (1 MW to 10 GW), or a struck-out bolt with none
                    local bx, by = x - 22, y + R - 26
                    if (o.mw or 0) > 0 then
                        icon("bolt", bx - 12, by - 4, 6, style.pale, 0.9)
                        local lit = clamp(math.floor(math.log10(o.mw) * 1.25 + 0.5), 1, 5)
                        for i = 0, 4 do
                            rect(i < lit and style.cyan or style.hud, i < lit and 0.9 or 0.2, bx + i * 9, by - 3 - i * 2, 6, 6 + i * 2)
                        end
                    elseif o.kind == "ship" or o.kind == "wreck" then
                        icon("bolt", bx - 12, by - 4, 6, style.pale, 0.55)
                        line(style.pale, 0.7, 1.4, bx - 19, by + 3, bx - 5, by - 11)
                    end
                    local act = o.kind == "ship" and ({ thrust = "flame", firing = "aim", cloak = "eyeOff" })[o.activity]
                    if act then icon(act, x + R * 0.62, y - R * 0.62, 8, style.amber, 0.95) end
                end

                -- the name (once it's known) and the distance, where the HUD vision's panel has its title
                if read >= 3 and o.name then text(o.name, x, y - R - 40, 13, style.core, 0.95, "center") end
                text(SensorText.distance(contact.distance), x, y - R - 22, 12, style.pale, 0.8, "center")

                -- the scan: progress round the rim, the echo in a meter under it (a notch for T3 and one for T4)
                if target.reading then
                    arc(style.amber, 0.95, 2.2, x, y, R + 7, 0, target.progress * 360, "neon")
                end
                local MR, m = R + ECHO_METER.gap, ECHO_METER
                local function at(snr) return m.from + m.sweep - m.sweep * clamp(math.log10(math.max(1, snr)) / 6, 0, 1) end
                arc(style.amber, 0.14, 4, x, y, MR, m.from, m.sweep)
                local fillFrom = at(target.echo.snr)
                arc(style.amber, target.echo.level > 0 and 0.9 or 0.55, 4, x, y, MR, fillFrom, m.from + m.sweep - fillFrom)
                for _, notch in ipairs({ { SensorModel.constants.scanT3Snr, 3 }, { SensorModel.constants.scanT4Snr, 4 } }) do
                    local a = at(notch[1]) * DEG
                    local sa, ca = math.sin(a), -math.cos(a)
                    line(style.pale, 0.8, 1.4, x + sa * (MR - 6), y + ca * (MR - 6), x + sa * (MR + 6), y + ca * (MR + 6))
                    tierIcon(notch[2], x + sa * (MR + 15), y + ca * (MR + 15))
                end
                -- too weak to read what's left: get closer
                if picture.scanning and not target.reading and target.read < 4 then
                    icon("chevrons", x, y + MR + 24, 9, style.amber, 0.5 + 0.45 * math.sin(clock * 6))
                end
                -- the target hears the scan
                if picture.scanning and (o.kind == "ship" or o.kind == "station") then
                    for k = 0, 1 do
                        local ph = (clock * 0.8 + k * 0.5) % 1
                        arc(style.amber, 0.5 * (1 - ph), 1.2, x, y, R + 22 + ph * 26, -54, 108)
                    end
                end

                -- what a full scan read, left of the panel: an icon each, and only the words that matter
                if target.read >= 4 then
                    for i, contentLine in ipairs(o.contents or {}) do
                        local kind = contentIcon(contentLine)
                        local words = kind and contentLine:gsub("^[^:]*:%s*", "") or contentLine
                        local ly, lx = y - 18 + (i - 1) * 24, x - R - 22
                        if kind then icon(kind, lx, ly - 4, 7, style.pale, 0.95) end
                        text(words, lx - 14, ly, 13, style.pale, 0.95, "right")
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
