---@meta

HmGui = HmGui

---Begin GUI declaration. Region is limited by [0, 0] - [sx, sy] rectangle.
---@param sx number
---@param sy number
---@param input Input
function HmGui:beginGui(self, sx, sy, input) end

---Finish GUI declaration, calculate hierarchy widgets sizes and layout.
---@param input Input
function HmGui:endGui(self, input) end

---Pass information about widgets to the renderer and draw them.
function HmGui:draw(self) end

function HmGui:beginHorizontalContainer(self) end

function HmGui:beginVerticalContainer(self) end

function HmGui:beginStackContainer(self) end

function HmGui:endContainer(self) end

---@param _max_size number
function HmGui:beginScroll(self, _max_size) end

---@param input Input
function HmGui:endScroll(self, input) end

---Begins window element.
---@param _title string
---@param input Input
function HmGui:beginWindow(self, _title, input) end

---Ends window element.
function HmGui:endWindow(self) end

---Invisible element that stretches in all directions.
---Use for pushing neighbor elements to the sides. See [`Self::checkbox`] for example.
function HmGui:spacer(self) end

---@param label string
---@return boolean
function HmGui:button(self, label) end

---@param label string
---@param value boolean
---@return boolean
function HmGui:checkbox(self, label, value) end

---@param _lower number
---@param _upper number
---@param _value number
---@return number
function HmGui:slider(self, _lower, _upper, _value) end

---@param height number
---@param color Color
function HmGui:horizontalDivider(self, height, color) end

---@param width number
---@param color Color
function HmGui:verticalDivider(self, width, color) end

---@param image Tex2D
function HmGui:image(self, image) end

---@param color Color
function HmGui:rect(self, color) end

---@param text string
function HmGui:text(self, text) end

---@param text string
---@param color Color
function HmGui:textColored(self, text, color) end

---@param font Font
---@param text string
---@param color Color
function HmGui:textEx(self, font, text, color) end

---@param width number
function HmGui:setMinWidth(self, width) end

---@param height number
function HmGui:setMinHeight(self, height) end

---@param width number
---@param height number
function HmGui:setMinSize(self, width, height) end

---@param width number
function HmGui:setFixedWidth(self, width) end

---@param height number
function HmGui:setFixedHeight(self, height) end

---@param width number
---@param height number
function HmGui:setFixedSize(self, width, height) end

---@param width number
function HmGui:setPercentWidth(self, width) end

---@param height number
function HmGui:setPercentHeight(self, height) end

---@param width number
---@param height number
function HmGui:setPercentSize(self, width, height) end

---@param px number
---@param py number
function HmGui:setMargin(self, px, py) end

---@param left number
---@param top number
---@param right number
---@param bottom number
function HmGui:setMarginEx(self, left, top, right, bottom) end

---@param margin number
function HmGui:setMarginLeft(self, margin) end

---@param margin number
function HmGui:setMarginTop(self, margin) end

---@param margin number
function HmGui:setMarginRight(self, margin) end

---@param margin number
function HmGui:setMarginBottom(self, margin) end

---@param width number
function HmGui:setBorderWidth(self, width) end

---@param color Color
function HmGui:setBorderColor(self, color) end

---@param color Color
function HmGui:setBorderColorV4(self, color) end

---@param width number
---@param color Color
function HmGui:setBorder(self, width, color) end

---@param width number
---@param color Color
function HmGui:setBorderV4(self, width, color) end

---@param color Color
function HmGui:setBgColor(self, color) end

---@param color Color
function HmGui:setBgColorV4(self, color) end

---@param h AlignHorizontal
---@param v AlignVertical
function HmGui:setAlignment(self, h, v) end

---@param align AlignHorizontal
function HmGui:setHorizontalAlignment(self, align) end

---@param align AlignVertical
function HmGui:setVerticalAlignment(self, align) end

---@param px number
---@param py number
function HmGui:setPadding(self, px, py) end

---@param left number
---@param top number
---@param right number
---@param bottom number
function HmGui:setPaddingEx(self, left, top, right, bottom) end

---@param padding number
function HmGui:setPaddingLeft(self, padding) end

---@param padding number
function HmGui:setPaddingTop(self, padding) end

---@param padding number
function HmGui:setPaddingRight(self, padding) end

---@param padding number
function HmGui:setPaddingBottom(self, padding) end

---@param spacing number
function HmGui:setSpacing(self, spacing) end

---Makes current container `focusable` and returns if it's currently in focus.
---@param ty FocusType
---@return boolean
function HmGui:isMouseOver(self, ty) end

---@param h AlignHorizontal
---@param v AlignVertical
function HmGui:setChildrenAlignment(self, h, v) end

---@param align AlignHorizontal
function HmGui:setChildrenHorizontalAlignment(self, align) end

---@param align AlignVertical
function HmGui:setChildrenVerticalAlignment(self, align) end

---Set a theme by merging it into the default properties.
---@param name string
function HmGui:setTheme(self, name) end

---Restore default properties.
function HmGui:clearTheme(self) end

---Get style id by its name.
---@param name string
---@return integer
function HmGui:getStyleId(self, name) end

---Set a style for the following element.
---@param id integer
function HmGui:setStyle(self, id) end

---Remove element style.
function HmGui:clearStyle(self) end

---Get property type by its id.
---@param id integer
---@return HmGuiPropertyType
function HmGui:getPropertyType(self, id) end

---Write property value into the mapped properties in the active element style.
---@param property_id integer
function HmGui:mapProperty(self, property_id) end

---Remove property by id from the active element style.
---@param property_id integer
function HmGui:removeProperty(self, property_id) end

---@param name string
---@param value boolean
---@param map_id string
---@return integer
function HmGui:registerPropertyBool(self, name, value, map_id) end

---@param name string
---@param value integer
---@param map_id string
---@return integer
function HmGui:registerPropertyI8(self, name, value, map_id) end

---@param name string
---@param value integer
---@param map_id string
---@return integer
function HmGui:registerPropertyU8(self, name, value, map_id) end

---@param name string
---@param value integer
---@param map_id string
---@return integer
function HmGui:registerPropertyI16(self, name, value, map_id) end

---@param name string
---@param value integer
---@param map_id string
---@return integer
function HmGui:registerPropertyU16(self, name, value, map_id) end

---@param name string
---@param value integer
---@param map_id string
---@return integer
function HmGui:registerPropertyI32(self, name, value, map_id) end

---@param name string
---@param value integer
---@param map_id string
---@return integer
function HmGui:registerPropertyU32(self, name, value, map_id) end

---@param name string
---@param value integer
---@param map_id string
---@return integer
function HmGui:registerPropertyI64(self, name, value, map_id) end

---@param name string
---@param value integer
---@param map_id string
---@return integer
function HmGui:registerPropertyU64(self, name, value, map_id) end

---@param name string
---@param value number
---@param map_id string
---@return integer
function HmGui:registerPropertyF32(self, name, value, map_id) end

---@param name string
---@param value number
---@param map_id string
---@return integer
function HmGui:registerPropertyF64(self, name, value, map_id) end

---@param name string
---@param value Vec2
---@param map_id string
---@return integer
function HmGui:registerPropertyVec2(self, name, value, map_id) end

---@param name string
---@param value Vec3
---@param map_id string
---@return integer
function HmGui:registerPropertyVec3(self, name, value, map_id) end

---@param name string
---@param value Vec4
---@param map_id string
---@return integer
function HmGui:registerPropertyVec4(self, name, value, map_id) end

---@param name string
---@param value IVec2
---@param map_id string
---@return integer
function HmGui:registerPropertyIVec2(self, name, value, map_id) end

---@param name string
---@param value IVec3
---@param map_id string
---@return integer
function HmGui:registerPropertyIVec3(self, name, value, map_id) end

---@param name string
---@param value IVec4
---@param map_id string
---@return integer
function HmGui:registerPropertyIVec4(self, name, value, map_id) end

---@param name string
---@param value UVec2
---@param map_id string
---@return integer
function HmGui:registerPropertyUVec2(self, name, value, map_id) end

---@param name string
---@param value UVec3
---@param map_id string
---@return integer
function HmGui:registerPropertyUVec3(self, name, value, map_id) end

---@param name string
---@param value UVec4
---@param map_id string
---@return integer
function HmGui:registerPropertyUVec4(self, name, value, map_id) end

---@param name string
---@param value DVec2
---@param map_id string
---@return integer
function HmGui:registerPropertyDVec2(self, name, value, map_id) end

---@param name string
---@param value DVec3
---@param map_id string
---@return integer
function HmGui:registerPropertyDVec3(self, name, value, map_id) end

---@param name string
---@param value DVec4
---@param map_id string
---@return integer
function HmGui:registerPropertyDVec4(self, name, value, map_id) end

---@param name string
---@param value Color
---@param map_id string
---@return integer
function HmGui:registerPropertyColor(self, name, value, map_id) end

---@param name string
---@param value Box3
---@param map_id string
---@return integer
function HmGui:registerPropertyBox3(self, name, value, map_id) end

---@param name string
---@param value string
---@param map_id string
---@return integer
function HmGui:registerPropertyString(self, name, value, map_id) end

---@param name string
---@param value Font
---@param map_id string
---@return integer
function HmGui:registerPropertyFont(self, name, value, map_id) end

---@param property_id integer
---@param value boolean
function HmGui:setPropertyBool(self, property_id, value) end

---@param property_id integer
---@param value integer
function HmGui:setPropertyI8(self, property_id, value) end

---@param property_id integer
---@param value integer
function HmGui:setPropertyU8(self, property_id, value) end

---@param property_id integer
---@param value integer
function HmGui:setPropertyI16(self, property_id, value) end

---@param property_id integer
---@param value integer
function HmGui:setPropertyU16(self, property_id, value) end

---@param property_id integer
---@param value integer
function HmGui:setPropertyI32(self, property_id, value) end

---@param property_id integer
---@param value integer
function HmGui:setPropertyU32(self, property_id, value) end

---@param property_id integer
---@param value integer
function HmGui:setPropertyI64(self, property_id, value) end

---@param property_id integer
---@param value integer
function HmGui:setPropertyU64(self, property_id, value) end

---@param property_id integer
---@param value number
function HmGui:setPropertyF32(self, property_id, value) end

---@param property_id integer
---@param value number
function HmGui:setPropertyF64(self, property_id, value) end

---@param property_id integer
---@param value Vec2
function HmGui:setPropertyVec2(self, property_id, value) end

---@param property_id integer
---@param value Vec3
function HmGui:setPropertyVec3(self, property_id, value) end

---@param property_id integer
---@param value Vec4
function HmGui:setPropertyVec4(self, property_id, value) end

---@param property_id integer
---@param value IVec2
function HmGui:setPropertyIVec2(self, property_id, value) end

---@param property_id integer
---@param value IVec3
function HmGui:setPropertyIVec3(self, property_id, value) end

---@param property_id integer
---@param value IVec4
function HmGui:setPropertyIVec4(self, property_id, value) end

---@param property_id integer
---@param value UVec2
function HmGui:setPropertyUVec2(self, property_id, value) end

---@param property_id integer
---@param value UVec3
function HmGui:setPropertyUVec3(self, property_id, value) end

---@param property_id integer
---@param value UVec4
function HmGui:setPropertyUVec4(self, property_id, value) end

---@param property_id integer
---@param value DVec2
function HmGui:setPropertyDVec2(self, property_id, value) end

---@param property_id integer
---@param value DVec3
function HmGui:setPropertyDVec3(self, property_id, value) end

---@param property_id integer
---@param value DVec4
function HmGui:setPropertyDVec4(self, property_id, value) end

---@param property_id integer
---@param value Color
function HmGui:setPropertyColor(self, property_id, value) end

---@param property_id integer
---@param value Box3
function HmGui:setPropertyBox3(self, property_id, value) end

---@param property_id integer
---@param value string
function HmGui:setPropertyString(self, property_id, value) end

---@param property_id integer
---@param value Font
function HmGui:setPropertyFont(self, property_id, value) end

---@param property_id integer
---@return boolean
function HmGui:getPropertyBool(self, property_id) end

---@param property_id integer
---@return integer
function HmGui:getPropertyI8(self, property_id) end

---@param property_id integer
---@return integer
function HmGui:getPropertyU8(self, property_id) end

---@param property_id integer
---@return integer
function HmGui:getPropertyI16(self, property_id) end

---@param property_id integer
---@return integer
function HmGui:getPropertyU16(self, property_id) end

---@param property_id integer
---@return integer
function HmGui:getPropertyI32(self, property_id) end

---@param property_id integer
---@return integer
function HmGui:getPropertyU32(self, property_id) end

---@param property_id integer
---@return integer
function HmGui:getPropertyI64(self, property_id) end

---@param property_id integer
---@return integer
function HmGui:getPropertyU64(self, property_id) end

---@param property_id integer
---@return number
function HmGui:getPropertyF32(self, property_id) end

---@param property_id integer
---@return number
function HmGui:getPropertyF64(self, property_id) end

---@param property_id integer
---@return Vec2
function HmGui:getPropertyVec2(self, property_id) end

---@param property_id integer
---@return Vec3
function HmGui:getPropertyVec3(self, property_id) end

---@param property_id integer
---@return Vec4
function HmGui:getPropertyVec4(self, property_id) end

---@param property_id integer
---@return IVec2
function HmGui:getPropertyIVec2(self, property_id) end

---@param property_id integer
---@return IVec3
function HmGui:getPropertyIVec3(self, property_id) end

---@param property_id integer
---@return IVec4
function HmGui:getPropertyIVec4(self, property_id) end

---@param property_id integer
---@return UVec2
function HmGui:getPropertyUVec2(self, property_id) end

---@param property_id integer
---@return UVec3
function HmGui:getPropertyUVec3(self, property_id) end

---@param property_id integer
---@return UVec4
function HmGui:getPropertyUVec4(self, property_id) end

---@param property_id integer
---@return DVec2
function HmGui:getPropertyDVec2(self, property_id) end

---@param property_id integer
---@return DVec3
function HmGui:getPropertyDVec3(self, property_id) end

---@param property_id integer
---@return DVec4
function HmGui:getPropertyDVec4(self, property_id) end

---@param property_id integer
---@return Color
function HmGui:getPropertyColor(self, property_id) end

---@param property_id integer
---@return Box3
function HmGui:getPropertyBox3(self, property_id) end

---@param property_id integer
---@return string
function HmGui:getPropertyString(self, property_id) end

---@param property_id integer
---@return Font
function HmGui:getPropertyFont(self, property_id) end

---Prints widgets hierarchy to the console. For testing.
function HmGui:dumpWidgets(self) end

