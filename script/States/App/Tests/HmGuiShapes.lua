--- HmGui shape widgets: every kind alpha blended and glowing, the angle
--- convention, and the HUD cases they are for (integrity rings, a curved
--- speed tape that fades towards its ends, with a dark halo on a bright
--- background and a pointed speed tag). All HmGui, no DrawEx.
--- Run: `./bin/ltr.exe HmGuiShapes` (capture with LTHEORY_CAPTURE).
local Test = require('States.Application')

function Test:getTitle() return 'HmGui Shapes' end

local white = Color(1.0, 1.0, 1.0, 1.0)
local dim = Color(0.6, 0.65, 0.7, 1.0)
local blue = Color(0.10, 0.60, 1.00, 1.0)
local cyan = Color(0.20, 0.90, 1.00, 1.0)
local pale = Color(0.75, 0.88, 1.00, 1.0)
local backdrop = Color(0.02, 0.025, 0.04, 1.0)

local center = { AlignHorizontal.Center, AlignVertical.Center }

-- Speed tape sketch: every piece is a widget of the same size, centred in one
-- stack container, so all pieces share the reticle as their centre. They all
-- get the same fade window around the tag (left, 270 degrees), so the tape
-- fades out towards both ends as one.
local TAPE = 320 -- widget size: the line's outer radius is 160
local FADE = { centre = 270, visible = 15, gone = 40 }

local function tapePieces()
    local function wedge(args)
        args.fade = FADE
        args.align = center
        return UIComponent.Wedge(args)
    end

    local pieces = {
        -- Dark halo under the tape, alpha blended.
        wedge { size = TAPE + 10, thickness = 14, start = 220, sweep = 100, color = Color(0, 0, 0, 0.7) },
        -- Solid line, 0 to max.
        wedge { size = TAPE, thickness = 2, start = 225, sweep = 55, color = blue, glow = true },
    }
    -- Dashes, max to boost.
    for a = 280, 297, 4 do
        pieces[#pieces + 1] = wedge { size = TAPE, thickness = 2, start = a + 0.5, sweep = 2.5, color = cyan, glow = true }
    end
    -- Dots, boost to travel mode.
    for a = 301, 315, 2 do
        pieces[#pieces + 1] = wedge { size = TAPE, thickness = 2, start = a, sweep = 0.6, color = pale, glow = true }
    end
    -- Ticks and caps: a small sweep, deeper than the line, sticking out on both sides.
    for _, a in ipairs({ 225, 239, 253, 267, 280, 300, 315 }) do
        local cap = a == 225 or a == 280 or a == 300 or a == 315
        pieces[#pieces + 1] = wedge { size = TAPE + (cap and 16 or 8), thickness = cap and 10 or 6,
            start = a - 0.3, sweep = 0.6, color = cap and white or blue, glow = true }
    end
    return pieces
end

function Test:onInit()
    self.font = Cache.Font('Unageo-Medium', 14)
    self.titleFont = Cache.Font('Exo2Bold', 22)
    self.tape = tapePieces()

    self.integrity = {
        UIComponent.Annulus { size = 200, thickness = 10, color = Color(0.2, 0.7, 1.0, 1.0), align = center },
        UIComponent.Annulus { size = 170, thickness = 10, color = Color(1.0, 0.8, 0.2, 1.0), align = center },
        UIComponent.Annulus { size = 140, thickness = 7, color = Color(0.2, 1.0, 0.3, 1.0), align = center },
    }
    -- A changing value read every frame: the hull band pulses (by frame, so
    -- captures don't depend on timing).
    self.frame = 0
    self.integrity[3].state.thickness = function() return 2 + 8 * (0.5 + 0.5 * math.sin(self.frame / 30)) end

    self.angles = {
        UIComponent.Ring { size = 150, color = dim, align = center },
        UIComponent.Wedge { size = 150, thickness = 16, start = -10, sweep = 20, color = Color(1, 0.2, 0.2, 1), align = center },
        UIComponent.Wedge { size = 150, thickness = 16, start = 80, sweep = 20, color = Color(0.2, 1, 0.2, 1), align = center },
        UIComponent.Wedge { size = 150, thickness = 16, start = 170, sweep = 20, color = Color(0.3, 0.5, 1, 1), align = center },
        UIComponent.Wedge { size = 150, thickness = 16, start = 260, sweep = 20, color = Color(1, 1, 0.2, 1), align = center },
        -- From 270 a quarter turn on, the way the speed tape grows.
        UIComponent.Wedge { size = 110, thickness = 4, start = 270, sweep = 90, color = cyan, glow = true, align = center },
    }

    -- The speed tag's point: a dark triangle with a glowing outline (two lines).
    local tip = { 1, 0, 0, 0.5, 1, 1 }
    self.tagPoint = {
        UIComponent.Triangle { size = { 14, 28 }, points = tip, color = Color(0, 0, 0, 0.9) },
        UIComponent.Line { size = { 14, 28 }, from = { 1, 0 }, to = { 0, 0.5 }, color = cyan, glow = true },
        UIComponent.Line { size = { 14, 28 }, from = { 0, 0.5 }, to = { 1, 1 }, color = cyan, glow = true },
    }
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

--- A stack container (all children share one centre) of `components`; the
--- caller ends it.
function Test:stack(components, w, h)
    Gui:beginStackContainer()
    Gui:setClipping(false)
    Gui:setFixedSize(w, h)
    for _, component in ipairs(components) do
        component:render()
    end
end

function Test:buildGui()
    self.frame = self.frame + 1
    Gui:beginGui(self.resX, self.resY)

    Gui:beginVerticalContainer()
    Gui:setClipping(false)
    Gui:setPadding(30, 20)
    Gui:setSpacing(18)

    Gui:text('HmGui shapes', self.titleFont, white)

    -- Every kind, alpha blended (left) and glowing (right).
    Gui:beginHorizontalContainer()
    Gui:setClipping(false)
    Gui:setSpacing(22)
    Gui:setChildrenVerticalAlignment(AlignVertical.Bottom)
    local kinds = {
        { 'annulus', 60, function(c, glow) Gui:annulus(c, 10, glow) end },
        { 'circle', 60, function(c, glow) Gui:circle(c, glow) end },
        -- A star-like sprite: the core is about 1/100 of the widget, the
        -- glow fills it (the Legacy HUD draws points 128 to 320 px wide).
        { 'point', 96, function(c, glow) Gui:point(c, glow) end },
        { 'ring', 60, function(c, glow) Gui:ring(c, glow) end },
        { 'wedge', 60, function(c, glow) Gui:wedge(c, 12, 300, 120, glow) end },
        { 'triangle', 60, function(c, glow) Gui:triangle(c, 0.5, 0.1, 0.95, 0.9, 0.05, 0.9, glow) end },
        { 'line', 60, function(c, glow) Gui:line(c, 0.1, 0.9, 0.9, 0.1, glow) end },
    }
    for _, kind in ipairs(kinds) do
        self:cell(kind[1] .. ': alpha, glow', function()
            Gui:beginHorizontalContainer()
            Gui:setClipping(false)
            Gui:setSpacing(kind[2] > 60 and 0 or 12)
            Gui:setHorizontalAlignment(AlignHorizontal.Center)
            for _, glow in ipairs({ false, true }) do
                kind[3](blue, glow)
                Gui:setFixedSize(kind[2], kind[2])
            end
            Gui:endContainer()
        end)
    end
    Gui:endContainer()

    Gui:beginHorizontalContainer()
    Gui:setClipping(false)
    Gui:setSpacing(40)

    self:cell('angles: 0 up, 90 right, clockwise', function()
        self:stack(self.angles, 200, 200)
        Gui:endContainer()
    end)

    self:cell('integrity rings, text on top', function()
        self:stack(self.integrity, 200, 200)
        Gui:text('73%', self.titleFont, white)
        Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)
        Gui:endContainer()
    end)

    self:cell('speed tape: fade window, dark halo on bright, tag', function()
        Gui:beginStackContainer()
        Gui:setClipping(false)
        Gui:setFixedSize(TAPE + 16, TAPE + 16)
        -- Bright backdrop behind the left half of the arc.
        Gui:rect()
        Gui:setFixedSize(70, 260)
        Gui:setBackgroundColor(Color(0.85, 0.9, 1.0, 1.0))
        Gui:setOpacity(1.0)
        Gui:setAlignment(AlignHorizontal.Left, AlignVertical.Center)
        for _, piece in ipairs(self.tape) do
            piece:render()
        end
        -- The tag, its point on the tape at 270 degrees.
        Gui:beginHorizontalContainer()
        Gui:setClipping(false)
        Gui:setAlignment(AlignHorizontal.Left, AlignVertical.Center)
        Gui:setMargin(8, 0) -- symmetric: one-sided margins are off on main (#385)
        self:stack(self.tagPoint, 14, 28)
        Gui:endContainer()
        Gui:beginStackContainer()
        Gui:setFixedSize(76, 28)
        Gui:setBackgroundColor(Color(0, 0, 0, 1))
        Gui:setOpacity(0.9)
        Gui:setBorderWidth(1)
        Gui:setBorderColor(cyan)
        Gui:text('167 m/s', self.font, white)
        Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)
        Gui:endContainer()
        Gui:endContainer()
        Gui:endContainer()
    end)

    Gui:beginVerticalContainer()
    Gui:setClipping(false)
    Gui:setSpacing(18)

    self:cell('widget box vs shape', function()
        Gui:beginVerticalContainer()
        Gui:setClipping(false)
        Gui:setSpacing(30)
        Gui:ring(cyan, true)
        Gui:setFixedSize(180, 90)
        Gui:setBorderWidth(1)
        Gui:setBorderColor(dim)
        Gui:annulus(blue, 1000, false) -- thicker than the radius: a disc
        Gui:setFixedSize(180, 60)
        Gui:setBorderWidth(1)
        Gui:setBorderColor(dim)
        Gui:endContainer()
    end)

    self:cell('clipping off / on (default)', function()
        Gui:beginHorizontalContainer()
        Gui:setClipping(false)
        Gui:setSpacing(30)
        for _, clip in ipairs({ false, true }) do
            Gui:beginStackContainer()
            Gui:setClipping(clip) -- a clipping container cuts the glow at its edge
            Gui:setFixedSize(70, 70)
            Gui:setBorderWidth(1)
            Gui:setBorderColor(dim)
            Gui:ring(cyan, true)
            Gui:setFixedSize(60, 60)
            Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)
            Gui:endContainer()
        end
        Gui:endContainer()
    end)

    Gui:endContainer()

    Gui:endContainer()
    Gui:endContainer()

    Gui:endGui()
end

-- Built and drawn from the render event (`Application` never calls `onUpdate`).
function Test:onRender()
    self:buildGui()
    self:immediateUI(function()
        -- An opaque backdrop, like the game's scene behind the HUD (glow is
        -- additive, so on the empty backbuffer it would only add alpha).
        Imm.Rect(0, 0, self.resX, self.resY, backdrop)
        Gui:draw()
    end)
end

return Test
