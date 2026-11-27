-- Speed tape: a speed scale curved around the reticle, on its left, with the
-- current-speed tag fixed at the reticle's height. The scale scrolls along the
-- arc behind the tag and fades out towards both ends, so only the part around
-- the current speed shows.
--
-- The scale, from the bottom:
--   0 to max speed: solid, linear, ticks at quarters of max
--   max to boost:   dashed, compressed
--   boost to the travel-mode top speed of the current zone: dotted, log scale
-- with a cap tick at each boundary and the names "BOOST" and "TRAVEL MODE"
-- beside their parts. The tag is the speed just outside the arc, ended with
-- a pointer glyph towards it (`167 m/s >`), all one text. Its colour follows
-- the speed state: stopped, normal flight (deep blue at low speed to light
-- blue at max), boosting, travel mode; it blends between states.
--
--   local tape = UIComponent.SpeedTape { visible = function() return shown end }
--   -- every frame, before rendering:
--   tape:update({ speed = s, maxSpeed = m, boostSpeed = 3 * m, travelSpeed = m * cap, travel = false }, dt)
--
-- Sizes are pixels at 1080p and scale with the window height, so the arc keeps
-- its radius relative to the screen on any aspect ratio. `centre` and `scale`
-- override where the reticle is and that scale (the test app shows several
-- tapes at once).
--
-- Colours and looks are in `UIComponent.SpeedTape.style` (below); change a
-- value there and every tape follows from the next frame. One tape can have
-- its own values on top: `style` in the constructor, or `tape.style` later,
-- with only the values that differ:
--
--   UIComponent.SpeedTape.style.normal = Color(1.0, 0.6, 0.1, 1.0)
--   tape.style = { looks = { tape = { glow = 0 } }, tag = { travel = Color(1, 1, 0, 1) } }

local UniverseScaleConfig = require("Config.Gen.UniverseScaleConfig")

---@class SpeedTapeInput
---@field speed number current speed, game units per second
---@field maxSpeed number top speed in normal flight
---@field boostSpeed number top speed while boosting
---@field travelSpeed number top speed in travel mode in the current zone
---@field travel boolean the travel drive is active (or slowing down from it)

---@class UIComponentSpeedTape: UIComponent
---@field state UIComponentState
---@field style table|nil this tape's own colours and looks, on top of `UIComponent.SpeedTape.style`
---@field update fun(self: UIComponentSpeedTape, input: SpeedTapeInput, dt: number|nil) takes this frame's values; dt nil = no blending
---@field render fun(self: UIComponentSpeedTape) renders the tape

---@class UIComponentSpeedTapeConstructor
---@field visible boolean|nil
---@field centre (fun(): number, number)|nil the reticle's screen position; default the window's centre
---@field scale number|(fun(): number)|nil sizes relative to 1080p; default window height / 1080
---@field style table|nil this tape's own colours and looks: only the values that differ from `UIComponent.SpeedTape.style`

-- Geometry, in pixels at 1080p.
local RADIUS = 408           -- the arc around the reticle (0.38 x the screen height)
local LENGTH_NORMAL = 291    -- 0 to max speed, along the arc
local LENGTH_BOOST = 79      -- max to boost
local LENGTH_TRAVEL = 68     -- boost to the travel-mode top speed
local FADE_VISIBLE = 110     -- fully drawn within this distance of the tag, along the arc
local FADE_GONE = 230        -- gone beyond this distance
local LINE_WIDTH = 2
local DOT_WIDTH = 2.5
local DASH = { 7, 4 }        -- boost part: dash, gap
local DOT_GAP = 5            -- travel part: dots this far apart
local MINOR_TICK = { length = 8, width = 1.6 }
local CAP_TICK = { length = 18, width = 2 }
local TICKS_PER_MAX = 4      -- minor ticks at quarters of max speed
-- Ticks give way to the tag: within `hidden` of it, along the arc, only
-- their inner half shows, from `shown` on all of them.
local TICK_CLEAR = { hidden = 8, shown = 16 }
local TAG_TIP_GAP = 4        -- between the line and the tag's right edge, outside the arc
local TAG_POINTER = ">"      -- the tag's last glyph, pointing at the line
local TAG_WIDEST_TEXT = "0000.0 Mm/s " .. TAG_POINTER -- the tag's box; the text sits at its right edge
local TAG_HEIGHT = 24
local TAG_FONT, TAG_FONT_SIZE = "Ubuntu", 14
local LABEL_FONT, LABEL_FONT_SIZE, LABEL_FONT_MIN = "UbuntuBold", 14, 11
local LABEL_GAP = 16         -- between a label's right edge and the line
local LABEL_WIDTH = 220      -- a label's box; the text sits at its right edge
local LABEL_HEIGHT = 24
-- The labels share the arc's outside with the speed: within `hidden` of the
-- tag, along the arc, they are gone, from `shown` on fully drawn.
local LABEL_CLEAR = { hidden = 14, shown = 36 }

local COLOR_BLEND_TIME = 0.1     -- seconds; time constant of the tag's colour blend (~0.3 s to settle)
local TRAVEL_TOP_BLEND_TIME = 0.5 -- seconds; the travel part rescales smoothly when the zone changes

--- The tape's colours and looks; public as `UIComponent.SpeedTape.style`.
---
--- The looks (see `UICore.Look`; widths in pixels at 1080p): the scale is neon
--- (`tape`: core and glow only add light), which washes out over bright
--- nebulae and stars; its core line is drawn again on top in `tapeCore`,
--- which covers what's behind (`shade = 1`), so the scale stays recognizable
--- there without a dark halo.
local STYLE = {
    -- The scale ("HUD blue").
    normal = Color(26 / 255, 153 / 255, 1.0, 1.0),  -- 0 to max speed, and the ticks at its quarters
    boost = Color(51 / 255, 230 / 255, 1.0, 1.0),   -- max to boost (dashed), and "BOOST"
    travel = Color(190 / 255, 225 / 255, 1.0, 1.0), -- boost to the travel-mode top (dotted), its cap tick, "TRAVEL MODE"
    cap = Color(1.0, 1.0, 1.0, 1.0),                -- the cap ticks at 0, max and boost
    -- The core line of the travel part and its cap tick, instead of
    -- `looks.tapeCore.boost`: 1 keeps their pale colour, which brightened
    -- would be white and vanish over white.
    travelCoreBrightness = 1.0,
    boostLabelAlpha = 0.8,  -- the labels are small and dim
    travelLabelAlpha = 0.55,

    -- The tag's colour per speed state (stopped, normal flight, boosting,
    -- travel mode).
    tag = {
        stopped = Color(0.69, 0.71, 0.75, 1.0), -- dim grey-white
        slow = Color(0.12, 0.41, 0.84, 1.0),    -- normal flight: deep blue at low speed ...
        atMax = Color(0.43, 0.80, 1.00, 1.0),   -- ... to light blue at max speed
        boost = Color(0.16, 0.94, 1.00, 1.0),   -- full cyan
        travel = Color(0.96, 0.98, 1.00, 1.0),  -- bright white
        textTint = 0.55, -- the state colour mixed this much towards white, readable in every state
        -- How strongly the text glows in the state colour: none when
        -- stopped, growing with speed to `atMax` at max speed, then
        -- boosting and travel mode. The glow lies under the text's dark
        -- halo, so the text stays readable.
        glow = { atMax = 0.35, boost = 0.6, travel = 0.8 },
    },

    looks = {
        tape = { core = 0.5, coreWidth = 1, glow = 0.4, glowWidth = 10 / 3, boost = 2, shade = 0 },
        tapeCore = { core = 1, coreWidth = 0.6, glow = 0, boost = 1.6, shade = 1 },
    },
}
local SCALED_LOOK_PARAMS = { coreWidth = true, glowWidth = true, haloWidth = true }

--- `base` with `own`'s values on top, section by section (a Color is a value,
--- a plain table a section).
local function merged(base, own)
    if type(own) ~= "table" then return base end
    local result = {}
    for key, value in pairs(base) do result[key] = value end
    for key, value in pairs(own) do
        if type(value) == "table" and type(base[key]) == "table" then
            result[key] = merged(base[key], value)
        else
            result[key] = value
        end
    end
    return result
end

local function sameValues(a, b)
    if not a then return false end
    for key, value in pairs(a) do
        if b[key] ~= value then return false end
    end
    for key in pairs(b) do
        if a[key] == nil then return false end
    end
    return true
end

local tapeCount = 0 -- each tape defines its looks under its own names

local function rgb(c)
    return { c.r, c.g, c.b }
end

local function mix(a, b, t)
    return { a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t, a[3] + (b[3] - a[3]) * t }
end

--- Like the stroke shader's fade: 1 within `visible`, 0 beyond `gone`.
local function fadeAt(distance, visible, gone)
    local t = math.max(0, math.min(1, (math.abs(distance) - visible) / (gone - visible)))
    return 1 - t * t * (3 - 2 * t)
end

---@param input SpeedTapeInput
---@param tag table the style's `tag` section
---@return number[] colour, number glow
local function stateOf(input, tag)
    if input.travel then return rgb(tag.travel), tag.glow.travel end
    if input.speed > input.maxSpeed * 1.02 then return rgb(tag.boost), tag.glow.boost end
    if input.speed / UniverseScaleConfig.globalScale < 1 then return rgb(tag.stopped), 0 end -- below 1 m/s
    local t = math.min(1, input.speed / input.maxSpeed)
    return mix(rgb(tag.slow), rgb(tag.atMax), t), tag.glow.atMax * t
end

---@param args UIComponentSpeedTapeConstructor
---@return UIComponentSpeedTape|nil
local function new(_, args)
    if not args then
        return
    end

    local tape = {}
    tape.state = UICore.ComponentState {
        visible = args.visible,
        scale = args.scale or function() return Window:height() / 1080 end,
    }
    local centreOf = args.centre or function() return 0.5 * Window:width(), 0.5 * Window:height() end
    tape.style = args.style

    tapeCount = tapeCount + 1
    local lookPrefix = string.format("speedTape%d.", tapeCount)
    local lookNames, definedLooks = {}, {} -- per look: its name, the values it was last defined with

    --- The style now: the shared one with this tape's own values on top. A
    --- replaced shared style only needs the values that differ, too.
    local function currentStyle()
        local shared = UIComponent.SpeedTape.style
        if shared ~= STYLE then
            shared = merged(STYLE, shared)
        end
        return merged(shared, tape.style)
    end

    --- (Re)defines this tape's looks from the style at this scale, each only
    --- when its values changed (a style change, or a new window size).
    local function defineLooks(looks, scale)
        for key, values in pairs(looks) do
            local scaled = {}
            for param, value in pairs(values) do
                scaled[param] = SCALED_LOOK_PARAMS[param] and value * scale or value
            end
            if not sameValues(definedLooks[key], scaled) then
                lookNames[key] = lookPrefix .. key
                definedLooks[key] = scaled
                UICore.Look.define(lookNames[key], scaled)
            end
        end
    end

    ---@type SpeedTapeInput|nil
    local input
    local color, glow, logTravelTop -- blended

    ---@param newInput SpeedTapeInput
    ---@param dt number|nil
    function tape:update(newInput, dt)
        if not (newInput.maxSpeed and newInput.maxSpeed > 0) then
            input = nil
            return
        end
        -- Keep the parts in order and at least a little long.
        local maxSpeed = newInput.maxSpeed
        local boostSpeed = math.max(newInput.boostSpeed or 0, maxSpeed * 1.01)
        local travelSpeed = math.max(newInput.travelSpeed or 0, boostSpeed * 1.01)
        input = {
            speed = math.max(0, newInput.speed or 0),
            maxSpeed = maxSpeed,
            boostSpeed = boostSpeed,
            travelSpeed = travelSpeed,
            travel = newInput.travel == true,
        }

        local targetColor, targetGlow = stateOf(input, currentStyle().tag)
        local blend = (dt and color) and (1 - math.exp(-dt / COLOR_BLEND_TIME)) or 1
        color = mix(color or targetColor, targetColor, blend)
        glow = (glow or targetGlow) + (targetGlow - (glow or targetGlow)) * blend

        local blendTop = (dt and logTravelTop) and (1 - math.exp(-dt / TRAVEL_TOP_BLEND_TIME)) or 1
        local targetTop = math.log(travelSpeed)
        logTravelTop = (logTravelTop or targetTop) + (targetTop - (logTravelTop or targetTop)) * blendTop
    end

    --- Where speed `v` sits on the scale, in pixels along the arc from 0.
    local function along(v)
        if v <= input.maxSpeed then
            return LENGTH_NORMAL * v / input.maxSpeed
        end
        if v <= input.boostSpeed then
            return LENGTH_NORMAL + LENGTH_BOOST * (v - input.maxSpeed) / (input.boostSpeed - input.maxSpeed)
        end
        local logBoost = math.log(input.boostSpeed)
        local t = (math.log(v) - logBoost) / math.max(1e-6, logTravelTop - logBoost)
        return LENGTH_NORMAL + LENGTH_BOOST + LENGTH_TRAVEL * math.min(1, t)
    end

    function tape:render()
        if not input or not self.state.visible() then
            return
        end

        local scale = self.state.scale()
        local style = currentStyle()
        defineLooks(style.looks, scale)
        local cx, cy = centreOf()
        local r = RADIUS * scale
        local here = along(input.speed)

        -- Angles: 0 = up, clockwise; the tag is at the arc's leftmost point
        -- (270) and higher speeds lie clockwise from it (upwards).
        local function angleAt(px) return 270 + math.deg((px - here) / RADIUS) end
        local function degrees(px) return math.deg(px / RADIUS) end
        local fadeVisible, fadeGone = degrees(FADE_VISIBLE), degrees(FADE_GONE)

        -- A full-window layer; everything in it is placed in window pixels.
        Gui:beginStackContainer()
        Gui:setClipping(false)

        --- An arc on the tape's circle, from `from` to `to` (pixels along);
        --- `inset` pixels inside it if given.
        local function arc(color, width, from, to, look, inset)
            local radius = r - (inset or 0) * scale
            local start = angleAt(from)
            Gui:arc(color, width * scale, start, angleAt(to) - start)
            Gui:setFixedSize(2 * radius, 2 * radius)
            Gui:setPosition(cx - radius, cy - radius)
            Gui:setShapeFade(270, fadeVisible, fadeGone)
            Gui:setStrokeLook(look)
        end

        local maxAt = LENGTH_NORMAL
        local boostAt = maxAt + LENGTH_BOOST
        local topAt = boostAt + LENGTH_TRAVEL

        -- The pieces: the line (solid, dashed, dotted), then the ticks at
        -- quarters of max speed and the caps.
        -- The travel part and its cap are `travel`: their core line keeps
        -- their own colour (`style.travelCoreBrightness`).
        local pieces = {
            { color = style.normal, width = LINE_WIDTH, from = 0, to = maxAt },
            { color = style.boost, width = LINE_WIDTH, from = maxAt, to = boostAt, dash = { DASH[1], DASH[2] } },
            { color = style.travel, width = DOT_WIDTH, from = boostAt, to = topAt, dash = { 0, DOT_GAP },
                roundCaps = true, travel = true },
        }
        --- A tick across the line. Near the tag it gives way: it fades to
        --- its inner half (the two cross-fade, so the inner half keeps its
        --- brightness), and nothing crosses the tag outside the line.
        local function tick(at, size, color, travel)
            local away = (math.abs(at - here) - TICK_CLEAR.hidden) / (TICK_CLEAR.shown - TICK_CLEAR.hidden)
            away = math.max(0, math.min(1, away))
            local function add(alpha, width, inset)
                if alpha <= 0 then return end
                pieces[#pieces + 1] = { color = Color(color.r, color.g, color.b, color.a * alpha), width = width,
                    inset = inset, from = at - 0.5 * size.width, to = at + 0.5 * size.width,
                    corner = 0.5 * size.width, travel = travel }
            end
            add(away, size.length)
            add(1 - away, 0.5 * size.length, 0.25 * size.length)
        end
        for i = 1, TICKS_PER_MAX - 1 do
            tick(maxAt * i / TICKS_PER_MAX, MINOR_TICK, style.normal)
        end
        tick(0, CAP_TICK, style.cap)
        tick(maxAt, CAP_TICK, style.cap)
        tick(boostAt, CAP_TICK, style.cap)
        tick(topAt, CAP_TICK, style.travel, true)

        local function draw(piece, look)
            arc(piece.color, piece.width, piece.from, piece.to, lookNames[look], piece.inset)
            if piece.dash then
                Gui:setStrokeDash(piece.dash[1] * scale, piece.dash[2] * scale, 0)
            end
            if piece.roundCaps then
                Gui:setStrokeRoundCaps(true)
            end
            if piece.corner then
                Gui:setStrokeCornerRadius(piece.corner * scale)
            end
            if look == "tapeCore" and piece.travel then
                Gui:setStrokeLookParam('boost', style.travelCoreBrightness)
            end
        end
        -- The texts come first and the scale over them: the texts' dark
        -- halos never cover the line or a tick passing the tag.

        --- A text right-aligned in a box of `width` x `height` whose right
        --- edge is at `right`, centred on `y`; outlined, so it reads on
        --- any background, and glowing in `glowColor` if given.
        local function textAt(text, font, fontSize, color, right, y, width, height, glowColor)
            Gui:beginStackContainer()
            Gui:setClipping(false) -- halo and glow reach past the box
            Gui:setFixedSize(width, height)
            Gui:setPosition(right - width, y - 0.5 * height)
            UIComponent.Text.addOutlined(text, font, fontSize, color, glowColor)
            Gui:setAlignment(AlignHorizontal.Right, AlignVertical.Center)
            Gui:endContainer()
        end

        -- Names beside their parts, scrolling and fading with them, and
        -- stepping aside near the tag, which is outside the arc too.
        local labelSize = math.max(LABEL_FONT_MIN, math.floor(LABEL_FONT_SIZE * scale + 0.5))
        local labelFont = Cache.Font(LABEL_FONT, labelSize)
        local function label(text, at, color, alpha)
            local distance = math.abs(at - here)
            local clear = (distance - LABEL_CLEAR.hidden) / (LABEL_CLEAR.shown - LABEL_CLEAR.hidden)
            local fade = fadeAt(distance, FADE_VISIBLE, FADE_GONE) * math.max(0, math.min(1, clear))
            if fade <= 0 then return end
            local a = math.rad(angleAt(at))
            local x = cx + r * math.sin(a) - LABEL_GAP * scale
            local y = cy - r * math.cos(a)
            textAt(text, labelFont, labelSize, Color(color.r, color.g, color.b, color.a * alpha * fade),
                x, y, LABEL_WIDTH * scale, LABEL_HEIGHT * scale)
        end
        label("BOOST", 0.5 * (maxAt + boostAt), style.boost, style.boostLabelAlpha)
        label("TRAVEL MODE", 0.5 * (boostAt + topAt), style.travel, style.travelLabelAlpha)

        -- The tag `167 m/s >`: one text just outside the line, its pointer a
        -- glyph of it, so it has the text's size, weight, halo and glow.
        local tagSize = math.max(LABEL_FONT_MIN, math.floor(TAG_FONT_SIZE * scale + 0.5))
        local tagFont = Cache.Font(TAG_FONT, tagSize)
        local tint = mix(color, { 1, 1, 1 }, style.tag.textTint)
        textAt(UniverseScaleConfig:formatSpeed(input.speed) .. " " .. TAG_POINTER, tagFont, tagSize,
            Color(tint[1], tint[2], tint[3], 1.0), cx - r - TAG_TIP_GAP * scale, cy,
            tagFont:getSize2(TAG_WIDEST_TEXT).x, TAG_HEIGHT * scale, Color(color[1], color[2], color[3], glow))

        -- All the neon first, then every core line on top of it (see STYLE).
        for _, piece in ipairs(pieces) do
            draw(piece, "tape")
        end
        for _, piece in ipairs(pieces) do
            draw(piece, "tapeCore")
        end

        Gui:endContainer()
        Gui:setPosition(0, 0)
        Gui:setPercentSize(100, 100)
    end

    return tape
end

UIComponent.SpeedTape = setmetatable({ style = STYLE }, { __call = new })

return UIComponent.SpeedTape
