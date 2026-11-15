-- Shape components: one widget per engine shape (`Gui:annulus`, `Gui:circle`,
-- `Gui:point`, `Gui:ring`, `Gui:wedge`, `Gui:triangle`, `Gui:line`) and the
-- bendable strokes (`Gui:arc`, `Gui:strokeBetween`, `Gui:strokeBent`).
--
-- The round shapes are centred in their widget and as wide as its smaller
-- side, so `size` is the shape's size. Triangles, lines and the strokes
-- between or from a point place their points at fractions of the widget
-- (0, 0 = top left, 1, 1 = bottom right), like CSS `clip-path: polygon()`.
-- Glow reaches past the widget's edge (like a CSS box-shadow); a clipping
-- parent container cuts it off at its own edge, so give the parent room or
-- `Gui:setClipping(false)`.
--
-- Shapes of one size stacked in one container share a centre: concentric
-- rings, or the pieces of an arc. Give them all the same `fade` and they fade
-- out together towards the ends.
--
-- Strokes are one line that bends from straight to a full circle, described
-- three ways: `Arc` (centre fixed: its middle line on the widget's inscribed
-- circle), `StrokeBetween` (both ends fixed, bent by `bend` degrees) and
-- `StrokeBent` (start, direction and length fixed; `bend` curls it up, 360
-- closes it). Annulus, ring, wedge and line are drawn as strokes too, so all
-- of these take a `look` (see `UICore.Look`), `lookParams` (single look values
-- for this widget), `dash = { dash, gap, phase }`, `roundCaps`, `cornerRadius`
-- (rounds flat ends) and `strokeFade = { centre, visible, gone }` (in pixels
-- along the stroke).
--
--   UIComponent.Annulus { size = 260, thickness = 10, color = Color(0, 1, 0, 1) }
--   UIComponent.Wedge { size = 816, thickness = 2, start = 270, sweep = 30, color = c, glow = true,
--       fade = { centre = 270, visible = 15, gone = 32 } }
--   UIComponent.Triangle { size = { 12, 30 }, points = { 1, 0, 0, 0.5, 1, 1 }, color = Color(0, 0, 0, 0.9) }
--   UIComponent.Arc { size = 320, thickness = 2, start = 285, sweep = 15, color = c, look = 'hudNeon',
--       dash = { 7, 4 }, fade = { centre = 270, visible = 15, gone = 40 } }
--   UIComponent.StrokeBent { size = 100, at = { 0, 0.5 }, direction = 90, length = 150, bend = 180 }
--
-- Every argument may also be a function returning the value (read every frame).

---@class UIComponentShape: UIComponent
---@field render fun(self: UIComponentShape) renders the shape

---@class UIComponentShapeFade
---@field centre number degrees, 0 = up, clockwise
---@field visible number fully drawn within this many degrees of `centre`
---@field gone number not drawn beyond this many degrees of `centre`

---@class UIComponentShapeConstructor
---@field visible boolean|nil
---@field size number|{ [1]: number, [2]: number } widget size in pixels: a square, or { width, height }
---@field color Color|nil default white
---@field glow boolean|nil draw additive (brightens only; neon) instead of alpha blended (can darken). Default false
---@field align { [1]: AlignHorizontal, [2]: AlignVertical }|nil
---@field margin { [1]: number, [2]: number }|nil

---@class UIComponentRoundShapeConstructor: UIComponentShapeConstructor
---@field fade UIComponentShapeFade|nil fade out by direction, measured from the shape's centre

---@class UIComponentBandConstructor: UIComponentRoundShapeConstructor
---@field thickness number band width in pixels, from the widget's edge inwards (nothing drawn at 0)

---@class UIComponentWedgeConstructor: UIComponentBandConstructor
---@field start number degrees, 0 = up, clockwise (like CSS conic-gradient)
---@field sweep number degrees from `start` (negative = counter-clockwise; nothing drawn at 0)

---@class UIComponentTriangleConstructor: UIComponentShapeConstructor
---@field points number[] { x1, y1, x2, y2, x3, y3 }, fractions of the widget

---@class UIComponentLineConstructor: UIComponentShapeConstructor
---@field from { [1]: number, [2]: number } fractions of the widget
---@field to { [1]: number, [2]: number } fractions of the widget

---@class UIComponentStrokeFade
---@field centre number pixels along the stroke from its start
---@field visible number fully drawn within this many pixels of `centre`
---@field gone number not drawn beyond this many pixels of `centre`

-- Also on Annulus, Ring, Wedge and Line, which are drawn as strokes.
---@class UIComponentStrokeOptions
---@field look string|nil a named look (`UICore.Look.define`); default: the plain `stroke` look
---@field lookParams table<string, number>|nil single look values for this widget, on top of its look
---@field dash { [1]: number, [2]: number, [3]: number|nil }|nil { dash, gap, phase } in pixels along the stroke
---@field roundCaps boolean|nil round ends (and dashes) instead of flat ones
---@field cornerRadius number|nil rounds the corners of flat ends (and dashes), in pixels; wedges have 2
---@field strokeFade UIComponentStrokeFade|nil fade along the length (replaces `fade`)

---@class UIComponentArcConstructor: UIComponentRoundShapeConstructor, UIComponentStrokeOptions
---@field thickness number stroke width in pixels, around its middle line on the widget's inscribed circle
---@field start number degrees, 0 = up, clockwise
---@field sweep number degrees from `start` (negative = counter-clockwise; 360 = full circle)

---@class UIComponentStrokeBetweenConstructor: UIComponentShapeConstructor, UIComponentStrokeOptions
---@field from { [1]: number, [2]: number } fractions of the widget
---@field to { [1]: number, [2]: number } fractions of the widget
---@field bend number|nil degrees it turns on the way, positive = clockwise, below 360. Default 0 (straight)
---@field thickness number stroke width in pixels

---@class UIComponentStrokeBentConstructor: UIComponentShapeConstructor, UIComponentStrokeOptions
---@field at { [1]: number, [2]: number } where it starts, fractions of the widget
---@field direction number degrees it leaves in, 0 = up, clockwise
---@field length number pixels
---@field bend number|nil degrees it turns in all, positive = clockwise; 360 closes it. Default 0 (straight)
---@field thickness number stroke width in pixels

local function newShape(args, draw)
    if not args then
        return
    end

    local shape = {}
    shape.state = UICore.ComponentState {
        visible = args.visible,
        size = args.size or 20,
        color = args.color or Color(1, 1, 1, 1),
        glow = args.glow or false,
        align = args.align or { AlignHorizontal.Default, AlignVertical.Default },
        margin = args.margin,
        fade = args.fade,
        thickness = args.thickness or 1,
        start = args.start or 0,
        sweep = args.sweep or 360,
        points = args.points or { 0.5, 0, 1, 1, 0, 1 },
        from = args.from or { 0, 0.5 },
        to = args.to or { 1, 0.5 },
        at = args.at or { 0, 0.5 },
        direction = args.direction or 90,
        length = args.length or 100,
        bend = args.bend or 0,
        look = args.look,
        lookParams = args.lookParams,
        dash = args.dash,
        roundCaps = args.roundCaps,
        cornerRadius = args.cornerRadius,
        strokeFade = args.strokeFade,
    }

    shape.render = function(self)
        if not self.state.visible() then
            return
        end

        draw(self.state, self.state.color(), self.state.glow() == true)

        local size = self.state.size()
        if type(size) == "table" then
            Gui:setFixedSize(size[1], size[2])
        else
            Gui:setFixedSize(size, size)
        end

        local align = self.state.align()
        Gui:setAlignment(align[1], align[2])

        if self.state.margin then
            local margin = self.state.margin()
            Gui:setMargin(margin[1], margin[2])
        end

        local fade = self.state.fade and self.state.fade()
        if fade then
            Gui:setShapeFade(fade.centre, fade.visible, fade.gone)
        end

        -- Stroke options (strokes, annulus, ring, wedge and line).
        local look = self.state.look and self.state.look()
        if look then
            Gui:setStrokeLook(look)
        end
        local lookParams = self.state.lookParams and self.state.lookParams()
        if lookParams then
            for param, value in pairs(lookParams) do
                Gui:setStrokeLookParam(param, value)
            end
        end
        local dash = self.state.dash and self.state.dash()
        if dash then
            Gui:setStrokeDash(dash[1], dash[2], dash[3] or 0)
        end
        local roundCaps = self.state.roundCaps and self.state.roundCaps()
        if roundCaps ~= nil then
            Gui:setStrokeRoundCaps(roundCaps)
        end
        local cornerRadius = self.state.cornerRadius and self.state.cornerRadius()
        if cornerRadius then
            Gui:setStrokeCornerRadius(cornerRadius)
        end
        local strokeFade = self.state.strokeFade and self.state.strokeFade()
        if strokeFade then
            Gui:setStrokeFade(strokeFade.centre, strokeFade.visible, strokeFade.gone)
        end
    end

    return shape
end

local function component(draw)
    return setmetatable({}, {
        __call = function(_, args)
            return newShape(args, draw)
        end
    })
end

---@type fun(args: UIComponentBandConstructor): UIComponentShape|nil
UIComponent.Annulus = component(function(state, color, glow)
    Gui:annulus(color, state.thickness(), glow)
end)

---@type fun(args: UIComponentRoundShapeConstructor): UIComponentShape|nil
UIComponent.Circle = component(function(_, color, glow)
    Gui:circle(color, glow)
end)

---@type fun(args: UIComponentRoundShapeConstructor): UIComponentShape|nil
UIComponent.Point = component(function(_, color, glow)
    Gui:point(color, glow)
end)

---@type fun(args: UIComponentRoundShapeConstructor): UIComponentShape|nil
UIComponent.Ring = component(function(_, color, glow)
    Gui:ring(color, glow)
end)

---@type fun(args: UIComponentWedgeConstructor): UIComponentShape|nil
UIComponent.Wedge = component(function(state, color, glow)
    Gui:wedge(color, state.thickness(), state.start(), state.sweep(), glow)
end)

---@type fun(args: UIComponentTriangleConstructor): UIComponentShape|nil
UIComponent.Triangle = component(function(state, color, glow)
    local p = state.points()
    Gui:triangle(color, p[1], p[2], p[3], p[4], p[5], p[6], glow)
end)

---@type fun(args: UIComponentLineConstructor): UIComponentShape|nil
UIComponent.Line = component(function(state, color, glow)
    local from, to = state.from(), state.to()
    Gui:line(color, from[1], from[2], to[1], to[2], glow)
end)

---@type fun(args: UIComponentArcConstructor): UIComponentShape|nil
UIComponent.Arc = component(function(state, color)
    Gui:arc(color, state.thickness(), state.start(), state.sweep())
end)

---@type fun(args: UIComponentStrokeBetweenConstructor): UIComponentShape|nil
UIComponent.StrokeBetween = component(function(state, color)
    local from, to = state.from(), state.to()
    Gui:strokeBetween(color, from[1], from[2], to[1], to[2], state.bend(), state.thickness())
end)

---@type fun(args: UIComponentStrokeBentConstructor): UIComponentShape|nil
UIComponent.StrokeBent = component(function(state, color)
    local at = state.at()
    Gui:strokeBent(color, at[1], at[2], state.direction(), state.length(), state.bend(), state.thickness())
end)

return {
    Annulus = UIComponent.Annulus,
    Arc = UIComponent.Arc,
    Circle = UIComponent.Circle,
    Line = UIComponent.Line,
    Point = UIComponent.Point,
    Ring = UIComponent.Ring,
    StrokeBent = UIComponent.StrokeBent,
    StrokeBetween = UIComponent.StrokeBetween,
    Triangle = UIComponent.Triangle,
    Wedge = UIComponent.Wedge,
}
