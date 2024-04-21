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

function HmGui:beginHorizontalContainer() end

function HmGui:beginVerticalContainer() end

function HmGui:beginStackContainer() end

function HmGui:endContainer() end

---Start scroll area.
---
---Internally scroll area represented by 2 nested stack containers for a area itself
---and 2 other containers for scroll bars. So it is possible to set layout parameters
---for both external and internal containers. For the former parameters should be
---specified after `Gui:end_scroll_area()` function call and for the latter after
---`Gui:beginScrollArea()`.
---
---Parameters:
---**dir** - define directions in which scrolling is enabled: All, Horizontal, Vertical.
---
---Example:
---```lua
---Gui:setPropertyBool(GuiProperties.ScrollAreaHScrollShow, false)
---Gui:beginScrollArea(ScrollDirection.All)
---
---Gui:beginVerticalContainer()
---Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Top)
---Gui:setChildrenAlignment(AlignHorizontal.Stretch, AlignVertical.Top)
---
---Gui:button("Button1")
---Gui:button("Button2")
---
---Gui:endContainer()
---Gui:endScrollArea(InputInstance)
---Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)
---Gui:setFixedSize(500, 500)
---```
---@param dir ScrollDirection
function HmGui:beginScrollArea(dir) end

---End of the scroll area.
---
---See [`HmGui::begin_scroll_area`] for example.
---@param input Input
function HmGui:endScrollArea(input) end

---Begins window element.
---@param title string
---@param input Input
function HmGui:beginWindow(title, input) end

---Ends window element.
function HmGui:endWindow() end

---Invisible element that stretches in all directions.
---Use for pushing neighbor elements to the sides. See [`Self::checkbox`] for example.
function HmGui:spacer() end

---@param label string
---@return boolean
function HmGui:button(label) end

---@param label string
---@param value boolean
---@return boolean
function HmGui:checkbox(label, value) end

---@param lower number
---@param upper number
---@param value number
---@return number
function HmGui:slider(lower, upper, value) end

---@param height number
---@param color Color
function HmGui:horizontalDivider(height, color) end

---@param width number
---@param color Color
function HmGui:verticalDivider(width, color) end

---@param image Tex2D
function HmGui:image(image) end

---@param color Color
function HmGui:rect(color) end

---@param text string
function HmGui:text(text) end

---@param text string
---@param color Color
function HmGui:textColored(text, color) end

---@param font Font
---@param text string
---@param color Color
function HmGui:textEx(font, text, color) end

---Makes current widget `focusable` and returns true if mouse is over it.
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

---@param color Color
function HmGui:setBorderColor(color) end

---@param color Color
function HmGui:setBorderColorV4(color) end

---@param width number
---@param color Color
function HmGui:setBorder(width, color) end

---@param width number
---@param color Color
function HmGui:setBorderV4(width, color) end

---@param color Color
function HmGui:setBgColor(color) end

---@param color Color
function HmGui:setBgColorV4(color) end

---@param h AlignHorizontal
---@param v AlignVertical
function HmGui:setAlignment(h, v) end

---@param align AlignHorizontal
function HmGui:setHorizontalAlignment(align) end

---@param align AlignVertical
function HmGui:setVerticalAlignment(align) end

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

---Get style id by its name.
---@param name string
---@return integer
function HmGui:getStyleId(name) end

---Set a style for the following element.
---@param id integer
function HmGui:setStyle(id) end

---Remove element style.
function HmGui:clearStyle() end

---Get property type by its id.
---@param id integer
---@return HmGuiPropertyType
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
---@param value boolean
---@param mapId string
---@return integer
function HmGui:registerPropertyBool(name, value, mapId) end

---@param name string
---@param value integer
---@param mapId string
---@return integer
function HmGui:registerPropertyI8(name, value, mapId) end

---@param name string
---@param value integer
---@param mapId string
---@return integer
function HmGui:registerPropertyU8(name, value, mapId) end

---@param name string
---@param value integer
---@param mapId string
---@return integer
function HmGui:registerPropertyI16(name, value, mapId) end

---@param name string
---@param value integer
---@param mapId string
---@return integer
function HmGui:registerPropertyU16(name, value, mapId) end

---@param name string
---@param value integer
---@param mapId string
---@return integer
function HmGui:registerPropertyI32(name, value, mapId) end

---@param name string
---@param value integer
---@param mapId string
---@return integer
function HmGui:registerPropertyU32(name, value, mapId) end

---@param name string
---@param value integer
---@param mapId string
---@return integer
function HmGui:registerPropertyI64(name, value, mapId) end

---@param name string
---@param value integer
---@param mapId string
---@return integer
function HmGui:registerPropertyU64(name, value, mapId) end

---@param name string
---@param value number
---@param mapId string
---@return integer
function HmGui:registerPropertyF32(name, value, mapId) end

---@param name string
---@param value number
---@param mapId string
---@return integer
function HmGui:registerPropertyF64(name, value, mapId) end

---@param name string
---@param value Vec2
---@param mapId string
---@return integer
function HmGui:registerPropertyVec2(name, value, mapId) end

---@param name string
---@param value Vec3
---@param mapId string
---@return integer
function HmGui:registerPropertyVec3(name, value, mapId) end

---@param name string
---@param value Vec4
---@param mapId string
---@return integer
function HmGui:registerPropertyVec4(name, value, mapId) end

---@param name string
---@param value IVec2
---@param mapId string
---@return integer
function HmGui:registerPropertyIVec2(name, value, mapId) end

---@param name string
---@param value IVec3
---@param mapId string
---@return integer
function HmGui:registerPropertyIVec3(name, value, mapId) end

---@param name string
---@param value IVec4
---@param mapId string
---@return integer
function HmGui:registerPropertyIVec4(name, value, mapId) end

---@param name string
---@param value UVec2
---@param mapId string
---@return integer
function HmGui:registerPropertyUVec2(name, value, mapId) end

---@param name string
---@param value UVec3
---@param mapId string
---@return integer
function HmGui:registerPropertyUVec3(name, value, mapId) end

---@param name string
---@param value UVec4
---@param mapId string
---@return integer
function HmGui:registerPropertyUVec4(name, value, mapId) end

---@param name string
---@param value DVec2
---@param mapId string
---@return integer
function HmGui:registerPropertyDVec2(name, value, mapId) end

---@param name string
---@param value DVec3
---@param mapId string
---@return integer
function HmGui:registerPropertyDVec3(name, value, mapId) end

---@param name string
---@param value DVec4
---@param mapId string
---@return integer
function HmGui:registerPropertyDVec4(name, value, mapId) end

---@param name string
---@param value Color
---@param mapId string
---@return integer
function HmGui:registerPropertyColor(name, value, mapId) end

---@param name string
---@param value Box3
---@param mapId string
---@return integer
function HmGui:registerPropertyBox3(name, value, mapId) end

---@param name string
---@param value string
---@param mapId string
---@return integer
function HmGui:registerPropertyString(name, value, mapId) end

---@param name string
---@param value Font
---@param mapId string
---@return integer
function HmGui:registerPropertyFont(name, value, mapId) end

---@param propertyId integer
---@param value boolean
function HmGui:setPropertyBool(propertyId, value) end

---@param propertyId integer
---@param value integer
function HmGui:setPropertyI8(propertyId, value) end

---@param propertyId integer
---@param value integer
function HmGui:setPropertyU8(propertyId, value) end

---@param propertyId integer
---@param value integer
function HmGui:setPropertyI16(propertyId, value) end

---@param propertyId integer
---@param value integer
function HmGui:setPropertyU16(propertyId, value) end

---@param propertyId integer
---@param value integer
function HmGui:setPropertyI32(propertyId, value) end

---@param propertyId integer
---@param value integer
function HmGui:setPropertyU32(propertyId, value) end

---@param propertyId integer
---@param value integer
function HmGui:setPropertyI64(propertyId, value) end

---@param propertyId integer
---@param value integer
function HmGui:setPropertyU64(propertyId, value) end

---@param propertyId integer
---@param value number
function HmGui:setPropertyF32(propertyId, value) end

---@param propertyId integer
---@param value number
function HmGui:setPropertyF64(propertyId, value) end

---@param propertyId integer
---@param value Vec2
function HmGui:setPropertyVec2(propertyId, value) end

---@param propertyId integer
---@param value Vec3
function HmGui:setPropertyVec3(propertyId, value) end

---@param propertyId integer
---@param value Vec4
function HmGui:setPropertyVec4(propertyId, value) end

---@param propertyId integer
---@param value IVec2
function HmGui:setPropertyIVec2(propertyId, value) end

---@param propertyId integer
---@param value IVec3
function HmGui:setPropertyIVec3(propertyId, value) end

---@param propertyId integer
---@param value IVec4
function HmGui:setPropertyIVec4(propertyId, value) end

---@param propertyId integer
---@param value UVec2
function HmGui:setPropertyUVec2(propertyId, value) end

---@param propertyId integer
---@param value UVec3
function HmGui:setPropertyUVec3(propertyId, value) end

---@param propertyId integer
---@param value UVec4
function HmGui:setPropertyUVec4(propertyId, value) end

---@param propertyId integer
---@param value DVec2
function HmGui:setPropertyDVec2(propertyId, value) end

---@param propertyId integer
---@param value DVec3
function HmGui:setPropertyDVec3(propertyId, value) end

---@param propertyId integer
---@param value DVec4
function HmGui:setPropertyDVec4(propertyId, value) end

---@param propertyId integer
---@param value Color
function HmGui:setPropertyColor(propertyId, value) end

---@param propertyId integer
---@param value Box3
function HmGui:setPropertyBox3(propertyId, value) end

---@param propertyId integer
---@param value string
function HmGui:setPropertyString(propertyId, value) end

---@param propertyId integer
---@param value Font
function HmGui:setPropertyFont(propertyId, value) end

---@param propertyId integer
---@return boolean
function HmGui:getPropertyBool(propertyId) end

---@param propertyId integer
---@return integer
function HmGui:getPropertyI8(propertyId) end

---@param propertyId integer
---@return integer
function HmGui:getPropertyU8(propertyId) end

---@param propertyId integer
---@return integer
function HmGui:getPropertyI16(propertyId) end

---@param propertyId integer
---@return integer
function HmGui:getPropertyU16(propertyId) end

---@param propertyId integer
---@return integer
function HmGui:getPropertyI32(propertyId) end

---@param propertyId integer
---@return integer
function HmGui:getPropertyU32(propertyId) end

---@param propertyId integer
---@return integer
function HmGui:getPropertyI64(propertyId) end

---@param propertyId integer
---@return integer
function HmGui:getPropertyU64(propertyId) end

---@param propertyId integer
---@return number
function HmGui:getPropertyF32(propertyId) end

---@param propertyId integer
---@return number
function HmGui:getPropertyF64(propertyId) end

---@param propertyId integer
---@return Vec2
function HmGui:getPropertyVec2(propertyId) end

---@param propertyId integer
---@return Vec3
function HmGui:getPropertyVec3(propertyId) end

---@param propertyId integer
---@return Vec4
function HmGui:getPropertyVec4(propertyId) end

---@param propertyId integer
---@return IVec2
function HmGui:getPropertyIVec2(propertyId) end

---@param propertyId integer
---@return IVec3
function HmGui:getPropertyIVec3(propertyId) end

---@param propertyId integer
---@return IVec4
function HmGui:getPropertyIVec4(propertyId) end

---@param propertyId integer
---@return UVec2
function HmGui:getPropertyUVec2(propertyId) end

---@param propertyId integer
---@return UVec3
function HmGui:getPropertyUVec3(propertyId) end

---@param propertyId integer
---@return UVec4
function HmGui:getPropertyUVec4(propertyId) end

---@param propertyId integer
---@return DVec2
function HmGui:getPropertyDVec2(propertyId) end

---@param propertyId integer
---@return DVec3
function HmGui:getPropertyDVec3(propertyId) end

---@param propertyId integer
---@return DVec4
function HmGui:getPropertyDVec4(propertyId) end

---@param propertyId integer
---@return Color
function HmGui:getPropertyColor(propertyId) end

---@param propertyId integer
---@return Box3
function HmGui:getPropertyBox3(propertyId) end

---@param propertyId integer
---@return string
function HmGui:getPropertyString(propertyId) end

---@param propertyId integer
---@return Font
function HmGui:getPropertyFont(propertyId) end

---Prints widgets hierarchy to the console. For testing.
function HmGui:dumpWidgets() end

