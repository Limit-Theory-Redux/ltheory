---@meta

---@class HmGui
HmGui = {}

---Begin GUI declaration. Region is limited by [0, 0] - [sx, sy] rectangle.
---@param sx number
---@param sy number
---@param input Input
function HmGui:beginGui(sx, sy, input) end

---Finish GUI declaration, calculate hierarchy widgets sizes and layout.
---@param input Input
function HmGui:endGui(input) end

---Pass information about widgets to the renderer and draw them.
function HmGui:draw() end

---Start a new container with a specified layout.
---@param layout GuiLayoutType
function HmGui:beginContainer(layout) end

---Starts stack container.
---Equivalent to: Gui:beginContainer(GuiLayoutType.Stack)
function HmGui:beginStackContainer() end

---Starts horizontal container.
---Equivalent to: Gui:beginContainer(GuiLayoutType.Horizontal)
function HmGui:beginHorizontalContainer() end

---Starts vertical container.
---Equivalent to: Gui:beginContainer(GuiLayoutType.Vertical)
function HmGui:beginVerticalContainer() end

---Closes container started with one of `Gui:beginContainer()` calls.
function HmGui:endContainer() end

---Update current container offset.
---Return offset value.
---@param offset Vec2f
---@return Vec2f
function HmGui:updateContainerOffset(offset) end

---Return current container element size calculated in previous frame.
---@return Vec2f
function HmGui:containerSize() end

---Return current container element size calculated in previous frame.
---@return Vec2f
function HmGui:containerMinSize() end

---Update current element minimum size.
---@param offset Vec2f
function HmGui:updateElementOffset(offset) end

---@param image Tex2D
function HmGui:image(image) end

function HmGui:rect() end

---@param text string
---@param font Font
---@param color Color
function HmGui:text(text, font, color) end

---Makes current widget `focusable` and returns true if mouse is over it.
---Returns true if mouse is over the widget (was calculated in the previous frame).
---@param ty FocusType
---@return boolean
function HmGui:isMouseOver(ty) end

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

---Set a theme by merging it into the default properties.
---@param name string
function HmGui:setTheme(name) end

---Restore default properties.
function HmGui:clearTheme() end

---Create a new empty style.
---Returns style id or None/nil if style with the same name already exists.
---
---Example:
---```lua
---local styleId = Gui:newStyle("MyStyle")
---Gui:setStyleProperty(GuiProperties.BackgroundColor, Color(1, 0, 0, 1))
---Gui:setStyleProperty(GuiProperties.Opacity, 0.5)
---
----- Later in the code
---
---Gui:setStyle(styleId)
---Gui:beginStackContainer()
---
---Gui:endContainer()
---```
---@param name string
---@return integer
function HmGui:newStyle(name) end

---Sets style property value.
---See example in `Gui:newStyle()` method description.
---@param styleId integer
---@param propId integer
---@param value GuiPropertyValue
function HmGui:setStylePropertyValue(styleId, propId, value) end

---Get style id by its name.
---@param name string
---@return integer
function HmGui:getStyleId(name) end

---Set a style for the following element by its id.
---Completely replaces current style with a new one.
---@param id integer
function HmGui:setStyle(id) end

---Set a style for the following element by its name.
---Completely replaces current style with a new one.
---NOTE: this method is slower than 'id' version.
---@param name string
function HmGui:setStyleByName(name) end

---Remove element style.
function HmGui:clearStyle() end

---Get property type by its id.
---@param id integer
---@return GuiPropertyType
function HmGui:getPropertyType(id) end

---Write property value into the mapped properties in the active element style.
---@param propertyId integer
function HmGui:mapProperty(propertyId) end

---Write all properties values of the group into their mapped properties in the active element style.
---Example: `gui.map_property_group("button")`
---It will map all properties with prefix "button.".
---@param group string
function HmGui:mapPropertyGroup(group) end

---Remove property by id from the active element style.
---@param propertyId integer
function HmGui:removeProperty(propertyId) end

---@param name string
---@param value GuiPropertyValue
---@param mapId string
---@return integer
function HmGui:registerProperty(name, value, mapId) end

---@param id integer
---@param value GuiPropertyValue
function HmGui:setPropertyValue(id, value) end

---@param id integer
---@return GuiPropertyValue
function HmGui:getPropertyValue(id) end

---Get number of registered properties.
---@return integer
function HmGui:getPropertiesCount() end

---Prints widgets hierarchy to the console. For testing.
function HmGui:dumpWidgets() end

