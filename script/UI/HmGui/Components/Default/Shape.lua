-- Shape components: one widget per engine shape (`Gui:annulus`, `Gui:circle`,
-- `Gui:point`, `Gui:ring`, `Gui:wedge`, `Gui:triangle`, `Gui:line`).
--
-- The round shapes are centred in their widget and as wide as its smaller
-- side, so `size` is the shape's size. Triangles and lines place their points
-- at fractions of the widget (0, 0 = top left, 1, 1 = bottom right), like CSS
-- `clip-path: polygon()`. Glow reaches past the widget's edge (like a CSS
-- box-shadow); a clipping parent container cuts it off at its own edge, so
-- give the parent room or `Gui:setClipping(false)`.
--
-- Shapes of one size stacked in one container share a centre: concentric
-- rings, or the pieces of an arc as separate wedges. Give them all the same
-- `fade` and they fade out together towards the ends.
--
--   UIComponent.Annulus { size = 260, thickness = 10, color = Color(0, 1, 0, 1) }
--   UIComponent.Wedge { size = 816, thickness = 2, start = 270, sweep = 30, color = c, glow = true,
--       fade = { centre = 270, visible = 15, gone = 32 } }
--   UIComponent.Triangle { size = { 12, 30 }, points = { 1, 0, 0, 0.5, 1, 1 }, color = Color(0, 0, 0, 0.9) }
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

return {
    Annulus = UIComponent.Annulus,
    Circle = UIComponent.Circle,
    Line = UIComponent.Line,
    Point = UIComponent.Point,
    Ring = UIComponent.Ring,
    Triangle = UIComponent.Triangle,
    Wedge = UIComponent.Wedge,
}
