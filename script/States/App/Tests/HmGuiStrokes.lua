--- HmGui bendable strokes: one line that bends from straight to a full
--- circle, described three ways, with dashes, caps, a fade along its length
--- and its look (core, glow, shade, dark halo) set from Lua. Ends with the
--- curved speed tape of `HmGuiShapes` (25 wedges) next to the same tape built
--- from 5 strokes.
--- Run: `./bin/ltr.exe HmGuiStrokes` (capture with LTHEORY_CAPTURE).
local Test = require('States.Application')

function Test:getTitle() return 'HmGui Strokes' end

local white = Color(1.0, 1.0, 1.0, 1.0)
local dim = Color(0.6, 0.65, 0.7, 1.0)
local red = Color(1.0, 0.25, 0.2, 1.0)
local green = Color(0.2, 1.0, 0.35, 1.0)
local blue = Color(0.10, 0.60, 1.00, 1.0)
local cyan = Color(0.20, 0.90, 1.00, 1.0)
local pale = Color(0.75, 0.88, 1.00, 1.0)
local bright = Color(0.85, 0.9, 1.0, 1.0)
local backdrop = Color(0.02, 0.025, 0.04, 1.0)

local center = { AlignHorizontal.Center, AlignVertical.Center }

-- The speed tape of `HmGuiShapes`: every piece shares the reticle as its
-- centre and fades out around the tag on the left (270 degrees).
local TAPE = 320
local FADE = { centre = 270, visible = 15, gone = 40 }

--- The tape as `HmGuiShapes` builds it, from 25 wedges.
local function wedgeTape()
    local function wedge(args)
        args.fade = FADE
        args.align = center
        return UIComponent.Wedge(args)
    end

    local pieces = {
        wedge { size = TAPE + 10, thickness = 14, start = 220, sweep = 100, color = Color(0, 0, 0, 0.7) },
        wedge { size = TAPE, thickness = 2, start = 225, sweep = 55, color = blue, glow = true },
    }
    for a = 280, 297, 4 do
        pieces[#pieces + 1] = wedge { size = TAPE, thickness = 2, start = a + 0.5, sweep = 2.5, color = cyan, glow = true }
    end
    for a = 301, 315, 2 do
        pieces[#pieces + 1] = wedge { size = TAPE, thickness = 2, start = a, sweep = 0.6, color = pale, glow = true }
    end
    for _, a in ipairs({ 225, 239, 253, 267, 280, 300, 315 }) do
        local cap = a == 225 or a == 280 or a == 300 or a == 315
        pieces[#pieces + 1] = wedge { size = TAPE + (cap and 16 or 8), thickness = cap and 10 or 6,
            start = a - 0.3, sweep = 0.6, color = cap and white or blue, glow = true }
    end
    return pieces
end

--- The same tape from 5 strokes. Their middle lines all lie on the widget's
--- inscribed circle (radius 160); the dark halo is part of the look, and
--- ticks are a dash pattern on a thick arc.
local function strokeTape()
    local radius = 0.5 * TAPE
    local px = function(degrees) return radius * math.rad(degrees) end

    local function arc(args)
        args.size = TAPE
        args.fade = FADE
        args.align = center
        args.look = args.look or 'tapeHalo'
        return UIComponent.Arc(args)
    end

    --- Ticks `width` px wide across the tape every `every` degrees from
    --- `from` to `to`: the arc reaches a degree past both ends so the first
    --- and last ticks aren't cut, and the phase centres a tick on `from`.
    local function ticks(from, to, every, width, args)
        args.start = from - 1
        args.sweep = to - from + 2
        args.dash = { width, px(every) - width, 0.5 * width - px(1) }
        args.cornerRadius = 2
        return arc(args)
    end

    return {
        -- 0 to max: solid. The line pieces carry the look's dark halo, so
        -- the tape stays readable on a bright background.
        arc { thickness = 2, start = 225, sweep = 60, color = blue },
        -- Max to boost: dashed.
        arc { thickness = 2, start = 285, sweep = 15, color = cyan, dash = { 7, 4, 0 } },
        -- Boost to travel mode: dotted (dashes of 0 with round caps).
        arc { thickness = 2.5, start = 300, sweep = 15, color = pale, dash = { 0, 5, 0 }, roundCaps = true },
        -- Ticks every 15 degrees on the solid part, white caps on the
        -- boundaries (inside the line's halo, so without one of their own).
        ticks(225, 285, 15, 1.6, { thickness = 6, color = blue, look = 'tape' }),
        ticks(285, 315, 15, 1.6, { thickness = 10, color = white, look = 'tape' }),
    }
end

function Test:onInit()
    self.font = Cache.Font('Unageo-Medium', 14)
    self.titleFont = Cache.Font('Exo2Bold', 22)
    self.frame = 0

    UICore.Look.define('neon', { core = 0.9, coreWidth = 0.8, glow = 0.5, glowWidth = 6, boost = 1.6, shade = 0 })
    UICore.Look.define('neonHalo', {
        core = 0.9, coreWidth = 0.8, glow = 0.5, glowWidth = 6, boost = 1.6, shade = 0,
        halo = 0.75, haloWidth = 14,
    })
    -- The tape: the look of the wedges it replaces (`wedgeGlow`), and the
    -- same with a dark halo for the line pieces.
    UICore.Look.define('tape', { core = 0.5, coreWidth = 1, glow = 0.4, glowWidth = 10 / 3, boost = 2, shade = 0 })
    UICore.Look.define('tapeHalo', {
        core = 0.5, coreWidth = 1, glow = 0.4, glowWidth = 10 / 3, boost = 2, shade = 0,
        halo = 0.6, haloWidth = 22,
    })

    self.wedgeTape = wedgeTape()
    self.strokeTape = strokeTape()
end

function Test:onInput() end

function Test:label(text)
    Gui:text(text, self.font, dim)
    Gui:setHorizontalAlignment(AlignHorizontal.Center)
end

--- A titled column: `body` adds the content.
function Test:cell(title, body)
    Gui:beginVerticalContainer()
    Gui:setClipping(false)
    Gui:setSpacing(6)
    body()
    self:label(title)
    Gui:endContainer()
end

function Test:stack(w, h)
    Gui:beginStackContainer()
    Gui:setClipping(false)
    Gui:setFixedSize(w, h)
end

--- One stroke, 130 px long, bent from 0 to a full circle.
function Test:bends()
    local length = 130
    Gui:beginHorizontalContainer()
    Gui:setClipping(false)
    Gui:setSpacing(26)
    Gui:setChildrenVerticalAlignment(AlignVertical.Top)
    local animated = 180 - 180 * math.cos(self.frame / 40)
    local bends = { 0, 60, 120, 180, 240, 300, 360, animated }
    for i, bend in ipairs(bends) do
        local title = i == #bends and 'animated' or string.format('%d', bend)
        self:cell(title, function()
            self:stack(length + 4, 120)
            -- Starts at the left, heads right, curls clockwise (down).
            Gui:strokeBent(cyan, 0.02, 0.12, 90, length, bend, 3)
            Gui:setStrokeLook('neon')
            Gui:setFixedSize(length + 4, 120)
            Gui:endContainer()
        end)
    end
    Gui:endContainer()
end

--- The same quarter circle described three ways: between two points (red,
--- thick), as a wire of that length (green), around the centre (white, thin).
function Test:descriptions()
    self:cell('between (red), bent (green), arc (white)', function()
        self:stack(200, 200)
        Gui:ring(Color(0.3, 0.35, 0.4, 1), false)
        Gui:setFixedSize(200, 200)
        -- From the left point up to the top point.
        Gui:strokeBetween(red, 0, 0.5, 0.5, 0, 90, 10)
        Gui:setFixedSize(200, 200)
        Gui:strokeBent(green, 0, 0.5, 0, 100 * math.pi / 2, 90, 5)
        Gui:setFixedSize(200, 200)
        Gui:arc(white, 1.5, 270, 90)
        Gui:setFixedSize(200, 200)
        -- Three quarters the other way round, from the top point.
        Gui:strokeBetween(red, 0.5, 0, 1, 0.5, -270, 10)
        Gui:setFixedSize(200, 200)
        Gui:strokeBent(green, 0.5, 0, 270, 100 * math.pi * 1.5, -270, 5)
        Gui:setFixedSize(200, 200)
        Gui:arc(white, 1.5, 0, -270)
        Gui:setFixedSize(200, 200)
        Gui:endContainer()
    end)
end

--- Straight strokes with caps and dash patterns.
function Test:dashes()
    self:cell('flat, round, dashes, pills, dots, rounded ticks, running', function()
        Gui:beginVerticalContainer()
        Gui:setClipping(false)
        Gui:setSpacing(12)
        local rows = {
            function() Gui:strokeBetween(blue, 0.05, 0.5, 0.95, 0.5, 0, 6) end,
            function()
                Gui:strokeBetween(blue, 0.05, 0.5, 0.95, 0.5, 0, 6)
                Gui:setStrokeRoundCaps(true)
            end,
            function()
                Gui:strokeBetween(cyan, 0.05, 0.5, 0.95, 0.5, 0, 4)
                Gui:setStrokeDash(12, 6, 0)
            end,
            function()
                Gui:strokeBetween(cyan, 0.05, 0.5, 0.95, 0.5, 0, 6)
                Gui:setStrokeDash(6, 12, 0)
                Gui:setStrokeRoundCaps(true)
            end,
            function()
                Gui:strokeBetween(pale, 0.05, 0.5, 0.95, 0.5, 0, 4)
                Gui:setStrokeDash(0, 9, 0)
                Gui:setStrokeRoundCaps(true)
            end,
            function()
                Gui:strokeBetween(white, 0.05, 0.5, 0.95, 0.5, 0, 12)
                Gui:setStrokeDash(3, 17, 1.5)
                Gui:setStrokeCornerRadius(1.5)
            end,
            function()
                Gui:strokeBetween(cyan, 0.05, 0.5, 0.95, 0.5, 0, 3)
                Gui:setStrokeDash(10, 8, -self.frame)
                Gui:setStrokeLook('neon')
            end,
        }
        for _, row in ipairs(rows) do
            row()
            Gui:setFixedSize(220, 14)
        end
        Gui:endContainer()
    end)
end

--- Looks over a dark and a bright background: plain alpha, neon (only adds
--- light, so it vanishes on bright), neon with a dark halo (readable on
--- both), and neon whose glow one widget overrides (pulsing).
function Test:looks()
    self:cell('plain, neon, neon + dark halo, glow override', function()
        self:stack(260, 150)
        Gui:rect()
        Gui:setFixedSize(130, 150)
        Gui:setBackgroundColor(bright)
        Gui:setOpacity(1.0)
        Gui:setAlignment(AlignHorizontal.Right, AlignVertical.Center)

        Gui:beginVerticalContainer()
        Gui:setClipping(false)
        Gui:setSpacing(22)
        Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)
        local rows = {
            function() Gui:strokeBetween(blue, 0.04, 0.5, 0.96, 0.5, 8, 3) end,
            function()
                Gui:strokeBetween(blue, 0.04, 0.5, 0.96, 0.5, 8, 3)
                Gui:setStrokeLook('neon')
            end,
            function()
                Gui:strokeBetween(blue, 0.04, 0.5, 0.96, 0.5, 8, 3)
                Gui:setStrokeLook('neonHalo')
            end,
            function()
                Gui:strokeBetween(blue, 0.04, 0.5, 0.96, 0.5, 8, 3)
                Gui:setStrokeLook('neonHalo')
                Gui:setStrokeLookParam('glow', 0.5 + 0.5 * math.sin(self.frame / 15))
            end,
        }
        for _, row in ipairs(rows) do
            row()
            Gui:setFixedSize(240, 12)
        end
        Gui:endContainer()
        Gui:endContainer()
    end)
end

--- Fades along the length: a straight stroke faded towards both ends, and a
--- full circle faded around the left by an angle window (`setShapeFade`).
function Test:fades()
    self:cell('fade along a line, window on a circle', function()
        Gui:beginVerticalContainer()
        Gui:setClipping(false)
        Gui:setSpacing(16)
        Gui:strokeBetween(cyan, 0, 0.5, 1, 0.5, 0, 3)
        Gui:setStrokeLook('neon')
        Gui:setStrokeFade(110, 40, 105)
        Gui:setFixedSize(220, 10)
        Gui:setHorizontalAlignment(AlignHorizontal.Center)

        Gui:arc(cyan, 3, 0, 360)
        Gui:setStrokeLook('neon')
        Gui:setShapeFade(270, 30, 90)
        Gui:setFixedSize(110, 110)
        Gui:setHorizontalAlignment(AlignHorizontal.Center)
        Gui:endContainer()
    end)
end

--- One tape: a bright backdrop behind its left half, its pieces, the tag.
function Test:tape(pieces)
    self:stack(TAPE + 16, TAPE + 16)
    Gui:rect()
    Gui:setFixedSize(70, 260)
    Gui:setBackgroundColor(bright)
    Gui:setOpacity(1.0)
    Gui:setAlignment(AlignHorizontal.Left, AlignVertical.Center)
    for _, piece in ipairs(pieces) do
        piece:render()
    end
    Gui:beginStackContainer()
    Gui:setFixedSize(76, 28)
    Gui:setAlignment(AlignHorizontal.Left, AlignVertical.Center)
    Gui:setMargin(22, 0)
    Gui:setBackgroundColor(Color(0, 0, 0, 1))
    Gui:setOpacity(0.9)
    Gui:setBorderWidth(1)
    Gui:setBorderColor(cyan)
    Gui:text('167 m/s', self.font, white)
    Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)
    Gui:endContainer()
    Gui:endContainer()
end

function Test:buildGui()
    self.frame = self.frame + 1
    Gui:beginGui(self.resX, self.resY)

    Gui:beginVerticalContainer()
    Gui:setClipping(false)
    Gui:setPadding(30, 20)
    Gui:setSpacing(18)

    Gui:text('HmGui strokes', self.titleFont, white)

    self:cell('one stroke, 130 px long, bent 0 to 360 degrees', function() self:bends() end)

    Gui:beginHorizontalContainer()
    Gui:setClipping(false)
    Gui:setSpacing(40)
    Gui:setChildrenVerticalAlignment(AlignVertical.Top)
    self:descriptions()
    self:dashes()
    self:looks()
    self:fades()
    Gui:endContainer()

    Gui:beginHorizontalContainer()
    Gui:setClipping(false)
    Gui:setSpacing(60)
    self:cell('speed tape: 25 wedges (HmGuiShapes)', function() self:tape(self.wedgeTape) end)
    self:cell('the same tape: 5 strokes', function() self:tape(self.strokeTape) end)
    Gui:endContainer()

    Gui:endContainer()

    Gui:endGui()
end

-- Built and drawn from the render event (`Application` never calls `onUpdate`).
function Test:onRender()
    self:buildGui()
    self:immediateUI(function()
        -- An opaque backdrop, like the game's scene behind the HUD.
        Imm.Rect(0, 0, self.resX, self.resY, backdrop)
        Gui:draw()
    end)
end

return Test
