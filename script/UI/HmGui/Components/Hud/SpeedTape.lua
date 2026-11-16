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
-- beside their parts. The tag's colour (and glow) follows the speed state:
-- stopped, normal flight (deep blue at low speed to light blue at max),
-- boosting, travel mode; it blends between states.
--
--   local tape = UIComponent.SpeedTape { visible = function() return shown end }
--   -- every frame, before rendering:
--   tape:update({ speed = s, maxSpeed = m, boostSpeed = 3 * m, travelSpeed = m * cap, travel = false }, dt)
--
-- Sizes are pixels at 1080p and scale with the window height, so the arc keeps
-- its radius relative to the screen on any aspect ratio. `centre` and `scale`
-- override where the reticle is and that scale (the test app shows several
-- tapes at once).

local UniverseScaleConfig = require("Config.Gen.UniverseScaleConfig")

---@class SpeedTapeInput
---@field speed number current speed, game units per second
---@field maxSpeed number top speed in normal flight
---@field boostSpeed number top speed while boosting
---@field travelSpeed number top speed in travel mode in the current zone
---@field travel boolean the travel drive is active (or slowing down from it)

---@class UIComponentSpeedTape: UIComponent
---@field state UIComponentState
---@field update fun(self: UIComponentSpeedTape, input: SpeedTapeInput, dt: number|nil) takes this frame's values; dt nil = no blending
---@field render fun(self: UIComponentSpeedTape) renders the tape

---@class UIComponentSpeedTapeConstructor
---@field visible boolean|nil
---@field centre (fun(): number, number)|nil the reticle's screen position; default the window's centre
---@field scale number|(fun(): number)|nil sizes relative to 1080p; default window height / 1080

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
local TAG_HEIGHT = 34
local TAG_POINT = 17         -- how far the point sticks out on the left
local TAG_TIP_GAP = 3        -- between the line and the point's tip
local TAG_PADDING = 12       -- each side of the widest speed text
local TAG_WIDEST_TEXT = "000.0 km/s"
local TAG_OUTLINE = 3        -- the glowing outline under the fill (which covers its inner half)
local TAG_CORE = 2           -- the outline's core line, on top of the fill
local TAG_FONT, TAG_FONT_SIZE = "UbuntuBold", 22
local LABEL_FONT, LABEL_FONT_SIZE, LABEL_FONT_MIN = "UbuntuBold", 14, 11
local LABEL_GAP = 16         -- between a label's right edge and the line
local LABEL_WIDTH = 220      -- a label's box; the text sits at its right edge
local LABEL_HEIGHT = 24

-- The scale's colours ("HUD blue").
local NORMAL_COLOR = Color(26 / 255, 153 / 255, 1.0, 1.0)
local BOOST_COLOR = Color(51 / 255, 230 / 255, 1.0, 1.0)
local TRAVEL_COLOR = Color(190 / 255, 225 / 255, 1.0, 1.0)
local CAP_COLOR = Color(1.0, 1.0, 1.0, 1.0)
local TRAVEL_CORE_BRIGHTNESS = 1.0 -- the travel part's core in its true colour (the other cores: the look's 1.6)
local BOOST_LABEL_ALPHA, TRAVEL_LABEL_ALPHA = 0.8, 0.55 -- small and dim

-- The tag per speed state: outline colour, its glow and its brightness (the
-- look's `glow` and `boost`).
local STOPPED = { 0.69, 0.71, 0.75 } -- dim grey-white
local SLOW = { 0.12, 0.41, 0.84 }    -- deep blue at low speed ...
local AT_MAX = { 0.43, 0.80, 1.00 }  -- ... to light blue at max speed
local BOOST = { 0.16, 0.94, 1.00 }   -- full cyan
local TRAVEL = { 0.96, 0.98, 1.00 }  -- bright white
local GLOW = { stopped = 0.25, normal = 0.5, boost = 0.7, travel = 0.8 }
local BRIGHTNESS = { stopped = 0.9, normal = 1.6, boost = 1.7, travel = 1.3 }
local TAG_FILL = Color(0.0, 0.0, 0.0, 0.9)
local TEXT_TINT = 0.35           -- the text: the state colour mixed this much towards white, readable in every state
local COLOR_BLEND_TIME = 0.1     -- seconds; time constant of the colour blend (~0.3 s to settle)
local TRAVEL_TOP_BLEND_TIME = 0.5 -- seconds; the travel part rescales smoothly when the zone changes

-- Looks (`UICore.Look`), widths in pixels at 1080p. The scale is neon
-- (`speedTape`: core and glow only add light), which washes out over bright
-- nebulae and stars; its core line is drawn again on top in `speedTapeCore`,
-- which covers what's behind (`shade = 1`), so the scale stays recognizable
-- there without a dark halo. The tag's outline is neon too, but alpha blended:
-- its five strokes overlap at the corners, and added light would make bright
-- dots there. Its glow and brightness are set per widget from the speed state.
local LOOKS = {
    speedTape = { core = 0.5, coreWidth = 1, glow = 0.4, glowWidth = 10 / 3, boost = 2, shade = 0 },
    speedTapeCore = { core = 1, coreWidth = 0.6, glow = 0, boost = 1.6, shade = 1 },
    speedTag = { core = 0.9, coreWidth = 0.8, glow = 0.5, glowWidth = 6, boost = 1.6, shade = 1 },
}
local SCALED_LOOK_PARAMS = { coreWidth = true, glowWidth = true, haloWidth = true }

local definedLookScale -- the scale the looks were last defined at

--- (Re)defines the looks for this scale; only when it changes.
local function defineLooks(scale)
    if definedLookScale == scale then return end
    definedLookScale = scale
    for name, values in pairs(LOOKS) do
        local scaled = {}
        for param, value in pairs(values) do
            scaled[param] = SCALED_LOOK_PARAMS[param] and value * scale or value
        end
        UICore.Look.define(name, scaled)
    end
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
---@return number[] colour, string state
local function stateOf(input)
    if input.travel then return TRAVEL, "travel" end
    if input.speed > input.maxSpeed * 1.02 then return BOOST, "boost" end
    if input.speed / UniverseScaleConfig.globalScale < 1 then return STOPPED, "stopped" end -- below 1 m/s
    return mix(SLOW, AT_MAX, math.min(1, input.speed / input.maxSpeed)), "normal"
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

    ---@type SpeedTapeInput|nil
    local input
    local color, glow, brightness, logTravelTop -- blended

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

        local targetColor, state = stateOf(input)
        local blend = (dt and color) and (1 - math.exp(-dt / COLOR_BLEND_TIME)) or 1
        local function towards(current, target) return (current or target) + (target - (current or target)) * blend end
        color = mix(color or targetColor, targetColor, blend)
        glow = towards(glow, GLOW[state])
        brightness = towards(brightness, BRIGHTNESS[state])

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
        defineLooks(scale)
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

        --- An arc on the tape's circle, from `from` to `to` (pixels along).
        local function arc(color, width, from, to, look)
            local start = angleAt(from)
            Gui:arc(color, width * scale, start, angleAt(to) - start)
            Gui:setFixedSize(2 * r, 2 * r)
            Gui:setPosition(cx - r, cy - r)
            Gui:setShapeFade(270, fadeVisible, fadeGone)
            Gui:setStrokeLook(look)
        end

        local maxAt = LENGTH_NORMAL
        local boostAt = maxAt + LENGTH_BOOST
        local topAt = boostAt + LENGTH_TRAVEL

        -- The pieces: the line (solid, dashed, dotted), then the ticks at
        -- quarters of max speed and the caps.
        local pieces = {
            { color = NORMAL_COLOR, width = LINE_WIDTH, from = 0, to = maxAt },
            { color = BOOST_COLOR, width = LINE_WIDTH, from = maxAt, to = boostAt, dash = { DASH[1], DASH[2] } },
            { color = TRAVEL_COLOR, width = DOT_WIDTH, from = boostAt, to = topAt, dash = { 0, DOT_GAP },
                roundCaps = true },
        }
        local function tick(at, size, color)
            pieces[#pieces + 1] = { color = color, width = size.length,
                from = at - 0.5 * size.width, to = at + 0.5 * size.width, corner = 0.5 * size.width }
        end
        -- The travel-mode part is pale; its brightened core would be white and
        -- vanish over white, so its core keeps its own colour.
        local function travelPart(piece) return piece.color == TRAVEL_COLOR end
        for i = 1, TICKS_PER_MAX - 1 do
            tick(maxAt * i / TICKS_PER_MAX, MINOR_TICK, NORMAL_COLOR)
        end
        tick(0, CAP_TICK, CAP_COLOR)
        tick(maxAt, CAP_TICK, CAP_COLOR)
        tick(boostAt, CAP_TICK, CAP_COLOR)
        tick(topAt, CAP_TICK, TRAVEL_COLOR)

        local function draw(piece, look)
            arc(piece.color, piece.width, piece.from, piece.to, look)
            if piece.dash then
                Gui:setStrokeDash(piece.dash[1] * scale, piece.dash[2] * scale, 0)
            end
            if piece.roundCaps then
                Gui:setStrokeRoundCaps(true)
            end
            if piece.corner then
                Gui:setStrokeCornerRadius(piece.corner * scale)
            end
            if look == 'speedTapeCore' and travelPart(piece) then
                Gui:setStrokeLookParam('boost', TRAVEL_CORE_BRIGHTNESS)
            end
        end
        -- All the neon first, then every core line on top of it (see LOOKS).
        for _, piece in ipairs(pieces) do
            draw(piece, 'speedTape')
        end
        for _, piece in ipairs(pieces) do
            draw(piece, 'speedTapeCore')
        end

        -- Names beside their parts, scrolling and fading with them.
        local labelFont = Cache.Font(LABEL_FONT, math.max(LABEL_FONT_MIN, math.floor(LABEL_FONT_SIZE * scale + 0.5)))
        local function label(text, at, color, alpha)
            local fade = fadeAt(at - here, FADE_VISIBLE, FADE_GONE)
            if fade <= 0 then return end
            local a = math.rad(angleAt(at))
            local x = cx + r * math.sin(a) - LABEL_GAP * scale
            local y = cy - r * math.cos(a)
            Gui:beginStackContainer()
            Gui:setFixedSize(LABEL_WIDTH * scale, LABEL_HEIGHT * scale)
            Gui:setPosition(x - LABEL_WIDTH * scale, y - 0.5 * LABEL_HEIGHT * scale)
            Gui:text(text, labelFont, Color(color.r, color.g, color.b, alpha * fade))
            Gui:setAlignment(AlignHorizontal.Right, AlignVertical.Center)
            Gui:endContainer()
        end
        label("BOOST", 0.5 * (maxAt + boostAt), BOOST_COLOR, BOOST_LABEL_ALPHA)
        label("TRAVEL MODE", 0.5 * (boostAt + topAt), TRAVEL_COLOR, TRAVEL_LABEL_ALPHA)

        -- The tag `< 167 m/s |`: pointed on the left with its tip just right
        -- of the line, straight on the right; dark fill, neon outline.
        local tagFont = Cache.Font(TAG_FONT, math.max(LABEL_FONT_MIN, math.floor(TAG_FONT_SIZE * scale + 0.5)))
        local h = TAG_HEIGHT * scale
        local point = TAG_POINT * scale
        local body = tagFont:getSize2(TAG_WIDEST_TEXT).x + 2 * TAG_PADDING * scale
        local left = cx - r + TAG_TIP_GAP * scale
        local top = cy - 0.5 * h
        local width = point + body
        local tagColor = Color(color[1], color[2], color[3], 1.0)

        -- Outline: five straight strokes with round caps, so the corners join.
        -- Its glow is drawn under the fill, so it shows outside only and the
        -- text stays on a dark ground in every state; the crisp core line is
        -- drawn again on top of the fill.
        local p = point / width
        local corners = { { 0, 0.5 }, { p, 0 }, { 1, 0 }, { 1, 1 }, { p, 1 }, { 0, 0.5 } }
        local function outline(thickness, outlineGlow)
            for i = 1, #corners - 1 do
                local a, b = corners[i], corners[i + 1]
                Gui:strokeBetween(tagColor, a[1], a[2], b[1], b[2], 0, thickness * scale)
                Gui:setFixedSize(width, h)
                Gui:setPosition(left, top)
                Gui:setStrokeLook('speedTag')
                Gui:setStrokeLookParam('glow', outlineGlow)
                Gui:setStrokeLookParam('boost', brightness)
                Gui:setStrokeRoundCaps(true)
            end
        end
        outline(TAG_OUTLINE, glow)

        -- Fill: the point (reaching 1 px under the body, so no seam shows)
        -- and the body.
        Gui:triangle(TAG_FILL, 0, 0.5, 1, 0, 1, 1, false)
        Gui:setFixedSize(point + scale, h)
        Gui:setPosition(left, top)
        Gui:rect()
        Gui:setBackgroundColor(TAG_FILL)
        Gui:setFixedSize(body, h)
        Gui:setPosition(left + point, top)
        outline(TAG_CORE, 0)

        -- The speed, centred in the body.
        local tint = mix(color, { 1, 1, 1 }, TEXT_TINT)
        Gui:beginStackContainer()
        Gui:setFixedSize(body, h)
        Gui:setPosition(left + point, top)
        Gui:text(UniverseScaleConfig:formatSpeed(input.speed), tagFont, Color(tint[1], tint[2], tint[3], 1.0))
        Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)
        Gui:endContainer()

        Gui:endContainer()
        Gui:setPosition(0, 0)
        Gui:setPercentSize(100, 100)
    end

    return tape
end

UIComponent.SpeedTape = setmetatable({}, { __call = new })

return UIComponent.SpeedTape
