-- Sensor HUD: what a ship's sensor shows, in two panels over the game.
--
-- Passive, at the bottom: a radar disc tilted away from the viewer with the
-- ship in the middle, nose up, and distance on a log scale (50 m at the
-- centre, 100 km at the rim). Contacts sit at their bearing and distance: a
-- soft blob for T1 (unresolved), a class icon for T2 (outline) and T3
-- (filled), amber at T4. Rings show where the target would turn T1 to T4 in
-- open space; a ring round the disc bulges where signals come from, before
-- they are contacts. Beside it, the contact list; to its left, the ship as
-- others hear it (power draw, heat, the last pulse).
--
-- Active, on the right: the scan window. The target's tier and echo, whether
-- a scan can read it from here, the scan's progress, and what scans have read
-- (T3 identity, T4 contents).
--
--   local hud = UIComponent.SensorHud { visible = function() return shown end, scanHint = "HOLD X TO SCAN" }
--   -- every frame, before rendering:
--   hud:update(SensorPicture.build(sensor, forward, right, "FRIGATE"), dt)
--
-- Places and sizes are pixels on a 1920 x 1080 screen and scale with the
-- window height. The passive panel keeps its place relative to the bottom
-- centre of the screen, the scan window relative to the bottom right corner.

local SensorModel = require("Modules.Sensors.SensorModel")
local SensorText = require("Modules.Sensors.SensorText")

---@class UIComponentSensorHud: UIComponent
---@field state UIComponentState
---@field update fun(self: UIComponentSensorHud, picture: SensorPictureData|nil, dt: number|nil) takes this frame's picture; nil hides the panels
---@field render fun(self: UIComponentSensorHud) renders both panels

---@class UIComponentSensorHudConstructor
---@field visible boolean|(fun(): boolean)|nil
---@field scale number|(fun(): number)|nil sizes relative to 1080p; default window height / 1080
---@field scanHint string|nil what the scan window says to start a scan

local DEG = math.pi / 180

-- Geometry, in pixels on a 1920 x 1080 screen.
local RADAR = { x = 880, y = 935, r = 118, tilt = 40 * DEG, depth = 7 } -- depth: perspective, in radii
local LIST = { x = 1050, y = 846, row = 21, max = 7 }
local YOU = { x = 560, y = 858, row = 21 }
local SCAN = { x = 1440, y = 540, w = 452, h = 510, pad = 18, row = 21 }
local RADAR_MIN, RADAR_MAX = 50, 1e5 -- metres at the radar's centre and at its rim
local SIGNAL_RING = 1.1              -- the signal ring's radius, in radar radii
local SIGNAL_BULGE = 0.3             -- how far a strong signal bulges it out
local SWEEP_SPEED = 1.4              -- radians per second
local SWEEP_STEPS, SWEEP_STEP = 14, 0.06
local PULSE_RING_TIME = 1.2          -- seconds the pulse ring takes to reach the rim and fade
local PULSE_TEXT_TIME = 4            -- seconds "pulse heard" stays up
local ECHO_FADE_TIME = 5             -- a blob seen by its pulse echo alone fades over its last seconds
-- Strokes per closed curve on the radar; each is bent to follow the curve
local SEGMENTS = { rim = 24, tier = 20, signal = 60 }
local GLYPH_SIZE, LIST_GLYPH_SIZE = 5.5, 4.5
local BLOB_RADIUS, MERGED_BLOB_RADIUS = 13, 17
local FONT, FONT_MIN = "UbuntuBold", 9

local function rgb(r, g, b) return { r / 255, g / 255, b / 255 } end

--- Colours ("HUD blue" and friends) and looks; public as `UIComponent.SensorHud.style`.
local STYLE = {
    hud = rgb(26, 153, 255),
    cyan = rgb(51, 230, 255),
    pale = rgb(190, 225, 255),
    core = rgb(225, 248, 255),
    amber = rgb(255, 170, 51),
    halo = Color(0, 0.02, 0.04, 0.6),
    -- neon: the rim, the signal and pulse rings and the scan window's frame,
    -- with a soft glow; line: thin rings and glyph outlines. Both are alpha
    -- blended: a curve is many strokes whose round ends overlap, and added
    -- light would make bright beads there.
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

--- Glyph outlines per kind of contact, in glyph radii (y down): class icons
--- (I circle, II triangle, III triangle down, IV hexagon), then planets,
--- stars and asteroids. nil = a circle.
local function glyphShape(o)
    local class = SensorModel.shipClass(o)
    if class == 2 then return { { 0, -1 }, { 0.95, 0.75 }, { -0.95, 0.75 } } end
    if class == 3 then return { { 0, 1 }, { 0.95, -0.75 }, { -0.95, -0.75 } } end
    if class == 4 then
        local hexagon = {}
        for i = 0, 5 do
            local a = (i * 60 + 30) * DEG
            hexagon[#hexagon + 1] = { math.cos(a), math.sin(a) }
        end
        return hexagon
    end
    if o.kind == "star" then
        return { { 0, -1.3 }, { 0.3, -0.3 }, { 1.3, 0 }, { 0.3, 0.3 }, { 0, 1.3 }, { -0.3, 0.3 }, { -1.3, 0 }, { -0.3, -0.3 } }
    end
    if o.kind == "asteroid" then return { { 0, -1 }, { 1, 0 }, { 0, 1 }, { -1, 0 } } end
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
    local scanHint = args.scanHint or "HOLD SCAN TO READ IT"

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

        -- 1080p places to window pixels: the passive panel from the bottom
        -- centre, the scan window from the bottom right.
        local ax, ay -- this anchor's offsets
        local function anchorBottomCentre() ax, ay = W / 2 - 960 * s, H - 1080 * s end
        local function anchorBottomRight() ax, ay = W - 1920 * s, H - 1080 * s end
        local function X(x) return ax + x * s end
        local function Y(y) return ay + y * s end

        -- Shapes, at 1080p places.
        --- A stroke from one place to another, bent `bend` degrees on the
        --- way (positive = clockwise). Its box holds the stroke with room to
        --- spare: the bulge of the bend (the arc's sagitta) and its width.
        local function line(c, alpha, width, x0, y0, x1, y1, look, bend)
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
            Gui:setStrokeRoundCaps(true)
        end

        local function polyline(c, alpha, width, points, look)
            for i = 1, #points - 1 do
                local a, b = points[i], points[i + 1]
                line(c, alpha, width, a[1], a[2], b[1], b[2], look)
            end
        end

        --- A closed curve on the radar's plane: radiusAt(angle) in radar
        --- radii, angle 0 ahead and growing clockwise. Each stroke is bent by
        --- the turn of the curve's direction between its ends, so a few dozen
        --- strokes follow the curve smoothly.
        local function radarCurve(c, alpha, width, radiusAt, segments, look)
            local function at(a)
                local r = radiusAt(a)
                return onRadar(r * math.sin(a), r * math.cos(a))
            end
            local step = 2 * math.pi / segments
            local function heading(a) -- screen direction of the curve at a, degrees (y down: clockwise grows)
                local x0, y0 = at(a - 0.05 * step)
                local x1, y1 = at(a + 0.05 * step)
                return math.atan2(y1 - y0, x1 - x0) / DEG
            end
            local x0, y0 = at(0)
            local h0 = heading(0)
            for i = 1, segments do
                local a = i * step
                local x1, y1 = at(a)
                local h1 = heading(a)
                line(c, alpha, width, x0, y0, x1, y1, look, wrapDegrees(h1 - h0))
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

        --- A round shape (`Gui:circle`, `ring`, `point`) of radius r around x, y.
        local function round(shape, c, alpha, x, y, r, glow)
            Gui[shape](Gui, color(c, alpha), glow == true)
            Gui:setFixedSize(2 * r * s, 2 * r * s)
            Gui:setPosition(X(x) - r * s, Y(y) - r * s)
        end

        local function arc(c, alpha, width, x, y, r, start, sweep)
            Gui:arc(color(c, alpha), width * s, start, sweep)
            Gui:setFixedSize(2 * r * s, 2 * r * s)
            Gui:setPosition(X(x) - r * s, Y(y) - r * s)
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

        local function glyph(o, x, y, size, c, filled)
            local shape = glyphShape(o)
            if not shape then
                local r = size * (o.kind == "planet" and 1.1 or 0.9)
                if filled then round("circle", c, 0.55, x, y, r) end
                round("ring", c, 0.95, x, y, r)
                return
            end
            local points = {}
            for _, p in ipairs(shape) do points[#points + 1] = { x + p[1] * size, y + p[2] * size } end
            if filled then
                -- Convex outlines fan out from their first corner; the star
                -- (concave) from its centre
                if o.kind == "star" then
                    for i = 1, #points do
                        local a, b = points[i], points[i % #points + 1]
                        triangle(c, 0.55, x, y, a[1], a[2], b[1], b[2])
                    end
                else
                    for i = 2, #points - 1 do
                        local a, b = points[i], points[i + 1]
                        triangle(c, 0.55, points[1][1], points[1][2], a[1], a[2], b[1], b[2])
                    end
                end
            end
            points[#points + 1] = points[1]
            polyline(c, 0.95, 1.4, points)
        end

        local function tierColor(tier) return tier >= 4 and style.amber or style.pale end

        -- A full-window layer; everything in it is placed in window pixels.
        Gui:beginStackContainer()
        Gui:setClipping(false)

        ------------------------------------------------------------------
        -- Passive: the radar
        anchorBottomCentre()
        local target = picture.target
        local targetObject = target and target.contact.object

        radarCurve(style.hud, 0.9, 1.6, function() return 1 end, SEGMENTS.rim, "neon")

        if target then
            local g = target.ranges
            local rings = {
                { g.scan4, "T4", style.amber }, { g.identify, "T3", style.cyan },
                { g.classify, "T2", style.cyan }, { g.detect, "T1", style.cyan },
            }
            for _, ring in ipairs(rings) do
                local d = ring[1]
                if d > RADAR_MIN and d <= RADAR_MAX then
                    local r = radarRadius(d)
                    radarCurve(ring[3], 0.32, 1, function() return r end, SEGMENTS.tier)
                    local lx, ly = polar(r, 180)
                    text(ring[2], lx, ly + 13, 11, ring[3], 0.8, "center")
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

        -- Contacts, and the target's corners and scan progress
        for _, c in ipairs(picture.objects) do
            local r = c.reading
            local x, y = polar(radarRadius(c.distance), c.bearing)
            if r.tier == 1 then
                -- heard: full; only a pulse echo: fading out over its last seconds
                local fade = r.passive.snr >= 1 and 1 or clamp(c.echo / ECHO_FADE_TIME, 0.25, 1)
                local bx = x + 2.5 * math.sin(clock * 1.7 + c.index * 2.1)
                local by = y + 1.5 * math.cos(clock * 1.3 + c.index)
                round("point", style.cyan, 0.6 * fade, bx, by, r.count > 1 and MERGED_BLOB_RADIUS or BLOB_RADIUS, true)
            elseif r.tier >= 2 then
                glyph(c.object, x, y, GLYPH_SIZE, tierColor(r.tier), r.tier >= 3)
            end
            if c.object == targetObject then
                local half, len = 11, 4
                local alpha = r.tier >= 1 and 0.95 or 0.35
                for _, sgn in ipairs({ { -1, -1 }, { 1, -1 }, { 1, 1 }, { -1, 1 } }) do
                    local sx, sy = sgn[1], sgn[2]
                    polyline(style.amber, alpha, 1.6, {
                        { x + sx * half, y + sy * (half - len) },
                        { x + sx * half, y + sy * half },
                        { x + sx * (half - len), y + sy * half },
                    })
                end
                if picture.scanning and target.progress > 0 then
                    arc(style.amber, 0.95, 2, x, y, 15, 0, target.progress * 360)
                end
            end
        end

        ------------------------------------------------------------------
        -- Passive: the contact list
        text(string.format("PASSIVE · 360° · ALWAYS ON%s", picture.dials.heatBand and " · HEAT BAND" or ""),
            LIST.x, LIST.y, 13, style.hud, 0.9)
        if #picture.contacts == 0 then
            text("NOTHING HEARD", LIST.x, LIST.y + LIST.row, 13, style.pale, 0.6)
        end
        for i = 1, math.min(LIST.max, #picture.contacts) do
            local c = picture.contacts[i]
            local r, o = c.reading, c.object
            local y = LIST.y + LIST.row * i
            local col = tierColor(r.tier)
            if r.tier == 1 then
                round("point", style.cyan, 0.6, LIST.x + 5, y - 5, 8, true)
            else
                glyph(o, LIST.x + 5, y - 5, LIST_GLYPH_SIZE, col, r.tier >= 3)
            end
            text("T" .. r.tier, LIST.x + 18, y, 13, style.hud, 0.95)
            text(SensorText.contact(o, r), LIST.x + 42, y, 13, col, o == targetObject and 1 or 0.85)
            if o == targetObject then rect(style.amber, 0.9, LIST.x - 9, y - 11, 3, 13) end
        end
        if #picture.contacts > LIST.max then
            text(string.format("+%d MORE", #picture.contacts - LIST.max), LIST.x + 42,
                LIST.y + LIST.row * (LIST.max + 1), 13, style.hud, 0.6)
        end

        ------------------------------------------------------------------
        -- Passive: you, as others hear you
        do
            local x, y = YOU.x, YOU.y
            text(string.format("YOU · %s · %s", string.upper(picture.title), SensorText.power(picture.dials.ownMw)),
                x, y, 13, style.hud, 0.9)
            text(string.format("HEARD WITHIN %s (EM)", string.upper(SensorText.distance(picture.heardWithin))),
                x, y + YOU.row, 13, style.pale, 0.85)
            y = y + 2 * YOU.row
            if picture.heat > 0.005 then
                local col = picture.bloom and style.amber or style.pale
                text("HEAT", x, y, 13, col, 0.9)
                rect(col, 0.2, x + 46, y - 10, 120, 8)
                rect(col, 0.85, x + 46, y - 10, 120 * picture.heat, 8)
                if picture.bloom then text("THERMAL BLOOM", x, y + YOU.row, 13, style.amber, 0.95) end
                y = y + (picture.bloom and 2 or 1) * YOU.row
            end
            if picture.pulseAge < PULSE_TEXT_TIME then
                local alpha = 0.95 * math.min(1, PULSE_TEXT_TIME - picture.pulseAge)
                text("PULSE HEARD WITHIN " .. string.upper(SensorText.distance(picture.pulseHeardWithin)),
                    x, y, 13, style.amber, alpha)
                text(string.format("YOUR POSITION HEARD BY %d", picture.pulseHeardBy), x, y + YOU.row, 13,
                    style.amber, alpha)
            end
        end

        ------------------------------------------------------------------
        -- Active: the scan window
        anchorBottomRight()
        do
            local x0, y0, x1, y1 = SCAN.x, SCAN.y, SCAN.x + SCAN.w, SCAN.y + SCAN.h
            rect(style.amber, 0.035, x0, y0, SCAN.w, SCAN.h)
            polyline(style.amber, 0.5, 1.1, { { x0, y0 }, { x1, y0 }, { x1, y1 }, { x0, y1 }, { x0, y0 } }, "neon")

            local x, right = x0 + SCAN.pad, x1 - SCAN.pad
            local y = y0 + 30
            local contact = target and target.contact
            local tier = contact and contact.reading.tier or 0
            text("ACTIVE SCAN", x, y, 14, style.amber, 0.95)
            local stateText = picture.scanning and (tier >= 1 and "SCANNING" or "NO CONTACT") or "STANDING BY"
            text(stateText, right, y, 14, picture.scanning and style.amber or style.pale, 0.85, "right")
            y = y + 30
            local function row(str, c, alpha)
                text(str, x, y, 13, c, alpha)
                y = y + SCAN.row
            end

            if not target then
                row("NO TARGET", style.pale, 0.6)
            elseif tier < 1 then
                local o = contact.object
                row(string.format("%s (FROM THE MAP) · %s", string.upper(o.label),
                    string.upper(SensorText.distance(contact.distance))), style.pale, 0.6)
                row("NO CONTACT: NOTHING TO AIM AT", style.amber, 0.9)
                row("PASSIVE: " .. string.upper(contact.reading.why), style.pale, 0.6)
                y = y + 8
                row("TRY A PULSE, OR GET CLOSER", style.pale, 0.6)
            else
                local o, echo, ranges = contact.object, target.echo, target.ranges
                row(string.format("T%d · %s", tier, SensorText.contact(o, contact.reading)), style.pale, 0.95)
                row("ECHO SNR " .. SensorText.number(echo.snr), style.amber, 0.9)
                if echo.level < 3 then
                    row("TOO WEAK TO READ · T3 WITHIN " .. string.upper(SensorText.distance(ranges.scan3)), style.pale, 0.7)
                elseif echo.level < 4 then
                    row("READS T3 · T4 WITHIN " .. string.upper(SensorText.distance(ranges.scan4)), style.pale, 0.7)
                else
                    row("READS T4: EVERYTHING", style.pale, 0.7)
                end
                y = y + 4
                rect(style.amber, 0.18, x, y - 10, SCAN.w - 2 * SCAN.pad, 8)
                if target.reading then
                    rect(style.amber, 0.9, x, y - 10, (SCAN.w - 2 * SCAN.pad) * target.progress, 8)
                end
                y = y + 18
                if not picture.scanning then
                    row(scanHint, style.pale, 0.7)
                elseif target.reading then
                    row(string.format("READING T%d · %d%%", echo.level, math.floor(target.progress * 100)), style.amber, 0.95)
                elseif echo.level == 0 then
                    row("ECHO TOO WEAK: GET CLOSER", style.amber, 0.9)
                else
                    row("NOTHING MORE TO READ FROM HERE", style.pale, 0.7)
                end
                y = y + 10
                rect(style.amber, 0.35, x, y - 14, SCAN.w - 2 * SCAN.pad, 1)
                y = y + 8

                text("T3 · IDENTITY", x, y, 12, style.hud, 0.9)
                y = y + SCAN.row
                if target.read >= 3 then
                    row(SensorText.name(o), style.pale, 0.95)
                    local extra = {}
                    if o.details then extra[#extra + 1] = o.details end
                    if (o.mw or 0) > 0 then extra[#extra + 1] = "DRAWING " .. SensorText.power(o.mw) end
                    if o.kind == "ship" and o.activity and o.activity ~= "idle" then
                        extra[#extra + 1] = string.upper(SensorModel.activityNames[o.activity])
                    end
                    if #extra > 0 then row(table.concat(extra, " · "), style.pale, 0.8) end
                else
                    row("NOT READ YET", style.pale, 0.55)
                end
                y = y + 6
                text("T4 · CONTENTS", x, y, 12, style.amber, 0.9)
                y = y + SCAN.row
                if target.read >= 4 then
                    for _, contentLine in ipairs(o.contents or {}) do row(contentLine, style.amber, 0.95) end
                else
                    row("NOT READ YET", style.pale, 0.55)
                end

                if picture.scanning then
                    local hears = o.kind == "ship" or o.kind == "station"
                    text(hears and "THE TARGET HEARS YOUR SCAN" or "SCANNING HEATS YOU UP", x, y1 - SCAN.pad, 13,
                        style.amber, 0.9)
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
