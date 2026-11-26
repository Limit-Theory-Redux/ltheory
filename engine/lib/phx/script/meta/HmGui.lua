-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class HmGui
HmGui = {}

-- Begin GUI declaration. Region is limited by [0, 0] - [sx, sy] rectangle.
---@param sx number
---@param sy number
function HmGui:beginGui(sx, sy) end

-- Finish GUI declaration, calculate hierarchy widgets sizes and layout.
---@param r Renderer
---@param input Input
function HmGui:endGui(r, input) end

-- Pass information about widgets to the renderer and draw them.
---@param r Renderer
function HmGui:draw(r) end

-- Begin a whole screen new layer on top of the current one.
-- Position of the layer (top/left corner) will be [0, 0] and size will be a size of the screen set in [`HmGui::begin_gui`].
-- All new elements will be added to this new layer.
-- Each layer has its own separate layout system.
function HmGui:beginLayer() end

-- Begin a new layer on top of the current one at specified position.
-- The size of new layer will bw up to the screen borders.
-- All new elements will be added to this new layer.
-- Each layer has its own separate layout system.
---@param pos Vec2f
function HmGui:beginLayerAtPos(pos) end

-- Begin a new layer below the latest element of the current layer.
-- Position and size of the new layer will be calculated after layouting of the previous layer.
-- All new elements will be added to this new layer.
-- Each layer has its own separate layout system.
function HmGui:beginLayerBelow() end

-- Close current layer and return to the previous one.
function HmGui:endLayer() end

-- Start a new container with a specified layout.
---@param layout GuiLayoutType
function HmGui:beginContainer(layout) end

-- Starts stack container.
-- Equivalent to: Gui:beginContainer(GuiLayoutType.Stack)
function HmGui:beginStackContainer() end

-- Starts horizontal container.
-- Equivalent to: Gui:beginContainer(GuiLayoutType.Horizontal)
function HmGui:beginHorizontalContainer() end

-- Starts vertical container.
-- Equivalent to: Gui:beginContainer(GuiLayoutType.Vertical)
function HmGui:beginVerticalContainer() end

-- Closes container started with one of `Gui:beginContainer()` calls.
function HmGui:endContainer() end

-- Update current container offset.
-- Return offset value.
---@param offset Vec2f
---@return Vec2f
function HmGui:updateContainerOffset(offset) end

-- Return last element size calculated in the previous frame.
---@return Vec2f
function HmGui:elementSize() end

-- Return current container element size calculated in the previous frame.
---@return Vec2f
function HmGui:containerSize() end

-- Return current container element size calculated in the previous frame.
---@return Vec2f
function HmGui:containerMinSize() end

-- Return current container position (top-left) calculated in the previous frame.
---@return Vec2f
function HmGui:containerPos() end

-- The UI scale of the current container (`set_scale`: its own times the
-- scales of the containers around it), screen pixels per design pixel.
-- Known while the GUI is built, unlike the sizes above. Divide what HmGui
-- reports in screen pixels (`container_size`, `element_size`, the mouse)
-- by it to size a widget inside the container from it.
---@return number
function HmGui:containerScale() end

-- Update current element minimum size.
---@param offset Vec2f
function HmGui:updateElementOffset(offset) end

---@param image Tex2D
function HmGui:image(image) end

function HmGui:rect() end

-- Add a ring band (annulus), centred in the widget: its outer diameter
-- is the widget's smaller side and it is `thickness` pixels wide,
-- inwards. Nothing is drawn at thickness 0.
--
-- Shapes draw over rects and under texts. `glow` draws additive
-- (brightens only, with a soft halo) instead of alpha blended; any glow
-- reaches past the widget's edge.
---@param color Color
---@param thickness number
---@param glow boolean
function HmGui:annulus(color, thickness, glow) end

-- Add a filled circle with a soft edge, centred in the widget, as wide
-- as the widget's smaller side. `glow`: see `annulus`.
---@param color Color
---@param glow boolean
function HmGui:circle(color, glow) end

-- Add a point of light: a small bright core in the widget's centre
-- whose soft glow fills the widget. `glow`: see `annulus`.
---@param color Color
---@param glow boolean
function HmGui:point(color, glow) end

-- Add a thin circle line, centred in the widget, as wide as the
-- widget's smaller side. `glow`: see `annulus`.
---@param color Color
---@param glow boolean
function HmGui:ring(color, glow) end

-- Add a piece of a ring band (wedge), centred in the widget: its outer
-- diameter is the widget's smaller side, it is `thickness` pixels deep,
-- inwards, and it runs from `start` to `start + sweep` degrees, where 0
-- is up and angles grow clockwise (like CSS `conic-gradient`). Nothing
-- is drawn at thickness or sweep 0. `glow`: see `annulus`.
--
-- Wedges of one widget size stacked in one container share a centre,
-- so each piece of an arc can be its own widget.
---@param color Color
---@param thickness number
---@param start number
---@param sweep number
---@param glow boolean
function HmGui:wedge(color, thickness, start, sweep, glow) end

-- Add a filled triangle with soft edges. Its corners are fractions of
-- the widget: (0, 0) is the top left, (1, 1) the bottom right (like CSS
-- `clip-path: polygon()`). A point at the left middle:
-- `triangle(c, 1, 0, 0, 0.5, 1, 1, false)`. `glow`: see `annulus`.
---@param color Color
---@param x1 number
---@param y1 number
---@param x2 number
---@param y2 number
---@param x3 number
---@param y3 number
---@param glow boolean
function HmGui:triangle(color, x1, y1, x2, y2, x3, y3, glow) end

-- Add a thin line from (`x1`, `y1`) to (`x2`, `y2`), fractions of the
-- widget as in `triangle`. Across the middle: `line(c, 0, 0.5, 1, 0.5,
-- false)`. `glow`: see `annulus`.
---@param color Color
---@param x1 number
---@param y1 number
---@param x2 number
---@param y2 number
---@param glow boolean
function HmGui:line(color, x1, y1, x2, y2, glow) end

-- Fade the last shape (annulus, circle, point, ring or wedge) by
-- direction: fully drawn within `visible` degrees of `centre`, gone
-- beyond `gone` degrees, smooth in between (0 = up, clockwise, measured
-- from the shape's centre). Shapes that share a centre and a fade fade
-- as one, like the pieces of a curved tape.
---@param centre number
---@param visible number
---@param gone number
function HmGui:setShapeFade(centre, visible, gone) end

-- Add a bendable stroke from (`x1`, `y1`) to (`x2`, `y2`), fractions of
-- the widget as in `triangle`, bent `bend` degrees on the way (positive
-- curves clockwise; below a full turn, since at 360 degrees the ends
-- meet and the circle's size is unknown), `thickness` pixels wide.
-- A bend of 0 is a straight line.
--
-- Strokes draw with the plain `stroke` look (a line in its colour)
-- until `setStrokeLook` names another; `setStrokeDash`,
-- `setStrokeRoundCaps` and `setStrokeFade` change the last one. Like
-- the other shapes they draw over rects and under texts, and glow
-- reaches past the widget's edge.
---@param color Color
---@param x1 number
---@param y1 number
---@param x2 number
---@param y2 number
---@param bend number
---@param thickness number
function HmGui:strokeBetween(color, x1, y1, x2, y2, bend, thickness) end

-- Add a bendable stroke that starts at (`x`, `y`), a fraction of the
-- widget, leaves in `direction` degrees (0 = up, clockwise) and is
-- `length` pixels long, turning `bend` degrees in all (positive =
-- clockwise): bending curls it up with its length kept, and 360 closes
-- it into a circle. `thickness` and the rest: see `strokeBetween`.
---@param color Color
---@param x number
---@param y number
---@param direction number
---@param length number
---@param bend number
---@param thickness number
function HmGui:strokeBent(color, x, y, direction, length, bend, thickness) end

-- Add a stroke around the widget's centre, its middle line on the circle
-- as wide as the widget's smaller side, from `start` to `start + sweep`
-- degrees (0 = up, clockwise; negative sweeps go counter-clockwise; 360
-- is a full circle), `thickness` pixels wide. Arcs of one widget size
-- stacked in one container share their middle line whatever their
-- thickness. It is a round shape, so `setShapeFade` works on it too.
-- The rest: see `strokeBetween`.
---@param color Color
---@param thickness number
---@param start number
---@param sweep number
function HmGui:arc(color, thickness, start, sweep) end

-- Dash the last stroke (or annulus, ring, wedge, line): `dash` pixels
-- drawn, then `gap` pixels left out, the pattern shifted back by `phase`
-- pixels (like SVG's `stroke-dashoffset`; animate it to make the dashes
-- run). Lengths are along the stroke's middle; round caps add half the
-- thickness to both ends of each dash, so `dash` 0 with round caps
-- makes dots. A `gap` of 0 is solid again.
---@param dash number
---@param gap number
---@param phase number
function HmGui:setStrokeDash(dash, gap, phase) end

-- Round ends (half circles, also on every dash) on the last stroke
-- instead of flat ones. Lines have round ends to begin with.
---@param round boolean
function HmGui:setStrokeRoundCaps(round) end

-- Round the corners of the last stroke's flat ends (and of its dashes)
-- by `radius` pixels, at most half its thickness. Wedges have 2 to
-- begin with; 0 makes them sharp.
---@param radius number
function HmGui:setStrokeCornerRadius(radius) end

-- Fade the last stroke along its length: fully drawn within `visible`
-- pixels of the point `centre` pixels from its start, gone beyond
-- `gone`, smooth in between. Replaces a `setShapeFade` window.
---@param centre number
---@param visible number
---@param gone number
function HmGui:setStrokeFade(centre, visible, gone) end

-- Draw the last stroke with the look called `look` (see `defineLook`).
-- A name not defined yet gets the default values until it is.
---@param look string
function HmGui:setStrokeLook(look) end

-- Set one look value for the last stroke only, on top of its look (for
-- example a glow that pulses on one element). `param` is one of `core`,
-- `coreWidth`, `glow`, `glowWidth`, `boost`, `shade`, `halo`, `haloWidth`
-- (see `defineLook`).
---@param param string
---@param value number
function HmGui:setStrokeLookParam(param, value) end

-- (Re)define the look called `look` with the default values: set what
-- differs with `setLookParam` (`UICore.Look.define` does both). Looks
-- outlive frames, so define them once; every stroke that uses one
-- follows when it changes. The built-in looks `annulus`, `ring`,
-- `wedge`, `line` and their `*Glow` twins are what those calls draw
-- with, and redefining one restyles them all.
--
-- The values (defaults in brackets): `core` [1] and `coreWidth` [0.5]:
-- strength of the solid core and how soft its edge is, in pixels;
-- `glow` [0] and `glowWidth` [4]: strength and reach of the glow around
-- it; `boost` [1]: multiplies the colour of the light; `shade` [1]: how
-- much it covers what is behind it, 0 = only adds light (neon), 1 =
-- normal alpha; `halo` [0] and `haloWidth` [8]: a dark halo darkening
-- what is behind out to that many pixels, 1 = black.
---@param look string
function HmGui:defineLook(look) end

-- Set one value of the look called `look` (created with the default
-- values if new). The values: see `defineLook`.
---@param look string
---@param param string
---@param value number
function HmGui:setLookParam(look, param, value) end

-- Add a line of text. A text widget narrower than its text (a fixed or
-- stretched width, or a parent that is too small) shows as much of the
-- text as fits, ended with "…".
---@param text string
---@param font Font
---@param color Color
function HmGui:text(text, font, color) end

-- Add a text element like `text`, with a 1 px `outline` and a 3 px
-- `halo` around the glyphs so it stays readable on any background (see
-- `Font::draw_outlined`). An alpha of 0 leaves a part out: a transparent
-- outline gives the halo alone. Takes the same room as `text`: outline
-- and halo spill past the widget, so a clipping container cuts them off
-- at its edge.
---@param text string
---@param font Font
---@param color Color
---@param outline Color
---@param halo Color
function HmGui:textOutlined(text, font, color, outline, halo) end

-- Break the last text into lines where it is wider than its max width
-- (`set_max_width`) or fixed width, between words, instead of cutting it
-- with "…" (like CSS `white-space: normal`). A line break ("\n") in a
-- text always starts a new line, wrapped or not. A percentage or
-- stretched width doesn't wrap it: give it a max width.
---@param wrap boolean
function HmGui:setTextWrap(wrap) end

-- Where the lines of the last text sit in its widget: `Left`, `Right`, or
-- centred (the default, as a single line is centred when its widget is
-- wider than it), like CSS `text-align`.
---@param align AlignHorizontal
function HmGui:setTextAlign(align) end

-- Add multiline styled text element.
---@param textData TextData
---@param editable boolean
function HmGui:textView(textData, editable) end

-- Apply changes to the text data if any and return if text was changed.
---@param textData TextData
---@return boolean
function HmGui:getTextViewChanges(textData) end

-- Makes current widget `focusable` and returns true if mouse is over it.
-- Returns true if mouse is over the widget (was calculated in the previous frame).
---@param ty FocusType
---@return boolean
function HmGui:isMouseOver(ty) end

-- Sets or removes current widget focus.
-- To be used in combination with some input check, i.e. mouse left click inside or outside the widget.
---@param focused boolean
function HmGui:setFocus(focused) end

-- Returns true if current widget is in focus.
---@return boolean
function HmGui:hasFocus() end

-- Returns true if there is an editable text view in focus.
---@return boolean
function HmGui:hasActiveInput() end

-- The widget is at least this wide (excluding margin and border); a
-- container or text is wider when its content needs it.
---@param width number
function HmGui:setMinWidth(width) end

-- The widget is at least this high (excluding margin and border); a
-- container or text is higher when its content needs it.
---@param height number
function HmGui:setMinHeight(height) end

-- The widget is at least this big (excluding margin and border); a
-- container or text is bigger when its content needs it.
---@param width number
---@param height number
function HmGui:setMinSize(width, height) end

-- The widget is at most this wide (excluding margin and border), also when
-- stretched or given a fixed size; a text that is wider is cut and ends
-- with "…" (like CSS `max-width` with `text-overflow: ellipsis`).
---@param width number
function HmGui:setMaxWidth(width) end

-- The widget is at most this high (excluding margin and border), also when
-- stretched or given a fixed size.
---@param height number
function HmGui:setMaxHeight(height) end

-- The widget is at most this big (excluding margin and border); see
-- `set_max_width`.
---@param width number
---@param height number
function HmGui:setMaxSize(width, height) end

-- Scale the widget and everything in it, on top of the scale of the
-- containers around it (like CSS `zoom`): every size, position, margin,
-- padding, spacing, border width, font size and shape length set from Lua
-- for it and inside it is in design pixels, `scale` screen pixels each.
-- Percentages need no scaling, and what HmGui reports back
-- (`container_pos`, `container_size`, `element_size`) and the mouse stay
-- in screen pixels. Layers begun inside it (dropdown lists, tooltips)
-- take its scale, so set it right after beginning the container, before
-- its children. A UI built for 1080p, scaled by the window height / 1080,
-- looks the same at every resolution.
---@param scale number
function HmGui:setScale(scale) end

-- Place the widget at (x, y) relative to its parent container's top left
-- corner inside the border (the parent's padding doesn't apply; its scroll
-- offset does). The widget leaves its parent's layout, like CSS
-- `position: absolute`: it doesn't move its siblings or make its parent
-- bigger, and its alignment is not used. Its size is its fixed, percent or
-- min size; its margins still apply.
---@param x number
---@param y number
function HmGui:setPosition(x, y) end

---@param width number
function HmGui:setFixedWidth(width) end

---@param height number
function HmGui:setFixedHeight(height) end

---@param width number
---@param height number
function HmGui:setFixedSize(width, height) end

---@param width number
function HmGui:setPercentWidth(width) end

---@param height number
function HmGui:setPercentHeight(height) end

---@param width number
---@param height number
function HmGui:setPercentSize(width, height) end

---@param px number
---@param py number
function HmGui:setMargin(px, py) end

---@param left number
---@param top number
---@param right number
---@param bottom number
function HmGui:setMarginEx(left, top, right, bottom) end

---@param margin number
function HmGui:setMarginLeft(margin) end

---@param margin number
function HmGui:setMarginTop(margin) end

---@param margin number
function HmGui:setMarginRight(margin) end

---@param margin number
function HmGui:setMarginBottom(margin) end

---@param width number
function HmGui:setBorderWidth(width) end

---@param h AlignHorizontal
---@param v AlignVertical
function HmGui:setAlignment(h, v) end

---@param align AlignHorizontal
function HmGui:setHorizontalAlignment(align) end

---@param align AlignVertical
function HmGui:setVerticalAlignment(align) end

---@param color Color
function HmGui:setBorderColor(color) end

-- Set the last widget's background colour: a panel in that colour is
-- drawn behind its content. Fully transparent, the default, draws none.
---@param color Color
function HmGui:setBackgroundColor(color) end

-- Set how opaque the inside of the last widget's background is, 0 to 1
-- (default 1); the background's shadow stays. Without a background
-- colour there is nothing to draw either way.
---@param opacity number
function HmGui:setOpacity(opacity) end

---@param clip boolean
function HmGui:setClipping(clip) end

---@param px number
---@param py number
function HmGui:setPadding(px, py) end

---@param left number
---@param top number
---@param right number
---@param bottom number
function HmGui:setPaddingEx(left, top, right, bottom) end

---@param padding number
function HmGui:setPaddingLeft(padding) end

---@param padding number
function HmGui:setPaddingTop(padding) end

---@param padding number
function HmGui:setPaddingRight(padding) end

---@param padding number
function HmGui:setPaddingBottom(padding) end

---@param spacing number
function HmGui:setSpacing(spacing) end

---@param h AlignHorizontal
---@param v AlignVertical
function HmGui:setChildrenAlignment(h, v) end

---@param align AlignHorizontal
function HmGui:setChildrenHorizontalAlignment(align) end

---@param align AlignVertical
function HmGui:setChildrenVerticalAlignment(align) end

-- Prints widgets hierarchy to the console. For testing.
function HmGui:dumpWidgets() end
