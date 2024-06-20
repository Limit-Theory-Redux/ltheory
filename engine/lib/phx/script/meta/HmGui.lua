---@meta

---@class HmGui
HmGui = {}

-- Begin GUI declaration. Region is limited by [0, 0] - [sx, sy] rectangle.
---@param sx number
---@param sy number
function HmGui:beginGui(sx, sy) end

-- Finish GUI declaration, calculate hierarchy widgets sizes and layout.
---@param input Input
function HmGui:endGui(input) end

-- Pass information about widgets to the renderer and draw them.
function HmGui:draw() end

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

-- Update current element minimum size.
---@param offset Vec2f
function HmGui:updateElementOffset(offset) end

---@param image Tex2D
function HmGui:image(image) end

function HmGui:rect() end

---@param text string
---@param font Font
---@param color Color
function HmGui:text(text, font, color) end

-- Add multiline styled text element.
---@param textData TextData
---@param editable boolean
function HmGui:textView(textData, editable) end

-- Makes current widget `focusable` and returns true if mouse is over it.
-- Returns true if mouse is over the widget (was calculated in the previous frame).
---@param ty FocusType
---@return boolean
function HmGui:isMouseOver(ty) end

-- Sets current widget in `focus`.
-- To be used in combination with some input check, i.e. mouse left click.
function HmGui:setFocus() end

-- Returns true if current widget is in focus.
---@return boolean
function HmGui:hasFocus() end

-- Returns true if there is an editable text view in focus.
---@return boolean
function HmGui:hasActiveInput() end

---@param width number
function HmGui:setMinWidth(width) end

---@param height number
function HmGui:setMinHeight(height) end

---@param width number
---@param height number
function HmGui:setMinSize(width, height) end

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

---@param color Color
function HmGui:setBackgroundColor(color) end

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

