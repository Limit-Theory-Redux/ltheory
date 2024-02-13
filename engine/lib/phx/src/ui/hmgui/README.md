# HmGui

HmGui (Hybrid Mode GUI) is a Graphical User Interface (GUI) framework that combines the best parts of the Immediate Mode and Retained Mode GUI models while minimizing their downsides.

The basic element of HmGui is the **widget**. There are three fundamental types of widgets: Text, Rect (rectangle), and Container. All more complex HmGui elements, such as Label, Textbox, Button, Checkbox, and Menu, are composed of combinations of these three basic widget types.

A widget represents an object covering a rectangular area to which can be assigned several different types of layout (positioning), styling (appearance), and window scrolling properties.

Images will be supported as a special property of the widget.

These features of HmGui are sufficient to accurately build complex GUI structures.

## Layout System

The HmGui layout system is based on the following property assignments:

1. Layout model of the container: Stack, Horizontal, Vertical
2. Alignment of child elements.
   - horizontal: Center, Left (default), Right, Stretch
   - vertical: Center, Top (default), Right, Stretch
3. Size specification: Fixed, Percent
4. Decorations: padding, spacing, margin and border

All of these properties can affect how elements are placed on the screen relative to each other and to the parent.

### Layout Model

A Container is a widget that may or may not have child elements attached to it. If it does, those child elements will be drawn such that they are positioned inside their parent Container according to the layout model assigned to that Container.

HmGui supports three layout models for arranging child elements in a Container:

- **Stack** (elements are drawn on top of each other and may overlap)
- **Horizontal** (elements are drawn horizontally from left to right without overlapping)
- **Vertical** (elements are drawn vertically from top to bottom without overlapping)

In addition to a parent Container's layout model, the positioning of elements in a Container is also affected by several properties of child elements:

- **alignment** - telling an element to attach one or more of its four sides (Left, Right, Top and Bottom) to its parent's side, to be centered in its parent, or to stretch as far as possible (inside its parent) either vertically or horizontally
- **fixed/percent size** - telling an element to either:
  - span a fixed size in pixels, or
  - expand, if possible, to a percentage of its parent's size
- **padding** - transparent space in pixels between the Container's enclosing Rect and its children
- **spacing** - transparent space in pixels between each child in either a Horizontal or Vertical layout
- **border** - styled space in pixels outside a Container rect
- **margin** - transparent space in pixels outside a Container and its border if one is specified

All elements may have alignment and fixed/percent size property values assigned to them.

Containers may also have decoration property values assigned to them, which may affect how child elements are laid out inside them.

Example:
```lua
Gui:beginHorizontalContainer()
Gui:setFixedSize(100, 100)

Gui:rect()
Gui:setFixedHeight(10)
Gui:setFixedWidth(20)

Gui:rect()
Gui:setFixedHeight(10)
Gui:setFixedWidth(20)

Gui:rect()
Gui:setFixedHeight(10)
Gui:setFixedWidth(20)

Gui:endContainer()
```
All three Rect elements will be drawn in a horizontal row, in order from left to right next to each other, with the entire group of three Rects centered in their parent Container. Each element will retain its fixed height and width.

### Alignment

Alignment is a property that can be used to position an element in relation to any of the four sides of its parent Container, or to the element beside the selected child element in the designated direction.

There are two dimensions along which elements can be aligned: Horizontal (width) and Vertical (height). In both alignment dimensions, one of six parameters can be specified: Center, Left (default), Right, Top (default), Bottom, and Stretch.

The HmGui programmer can set alignment for both dimensions independently.

Stretching an element in the horizontal or vertical dimension by alignment always has priority over fixed/percent size in that dimension. If both alignment and a fixed or percent size are specified for an element, the alignment effect will be applied.

Example:
```lua
Gui:beginStackContainer()
Gui:setFixedSize(100, 100)

Gui:rect()
Gui:setFixedWidth(20)
Gui:setVerticalAlignment(AlignVertical.Stretch)

Gui:rect()
Gui:setFixedHeight(20)
Gui:setHorizontalAlignment(AlignHorizontal.Stretch)

Gui:rect()
Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Stretch)

Gui:endContainer()
```
Because this Container has been told to use the Stack layout model (through the command `Gui:beginStackContainer()`, these three Rects will overlap. There will be one tall, skinny Rect on the left; one short, wide Rect at the top; and one Rect that fills the Container.

Additionally, the commands `setChildrenHorizontalAlignment()` and `setChildrenVerticalAlignment()` may be applied to Containers. These will impose one alignment effect on all of that Container's child elements. For example, to align all children to the left inside a container using the Vertical layout model, users can do the following:

```lua
Gui:beginVerticalContainer()
Gui:setFixedSize(100, 100)
Gui:setChildrenHorizontalAlignment(AlignHorizontal.Left)

Gui:rect()
Gui:setFixedWidth(20)

Gui:rect()
Gui:setFixedWidth(40)

Gui:rect()
Gui:setFixedWidth(30)

Gui:endContainer()
```

All of the three child Rect elements in this example will be drawn stacked on top of each other, in order from top to bottom, with each element's left side touching the left side of their parent container, and the group of three elements will be centered vertically by default (since no explicit Vertical alignment was specified).

Note: `setChildrenHorizontalAlignment()` will accept only the parameters AlignHorizontal.Center, AlignHorizontal.Left, AlignHorizontal.Right, and AlignHorizontal.Stretch; and `setChildrenVerticalAlignment()` will accept only the parameters AlignVertical.Center, AlignVertical.Top, AlignVertical.Bottom, and AlignVertical.Stretch.

#### Advanced Alignment

`Gui:setChildren*Alignment()` methods can be used for simple alignment of the Container's children. This alignment will be applied only for the children without explicit child alignment specification and explicit fixed/percent size (children alignment has lower priority than child fixed/percent size in contrast to child alignment that has higher priority).

Example:
```lua
Gui:beginHorizontalContainer()
Gui:setFixedSize(100, 100)
Gui:setChildrenHorizontalAlignment(AlignHorizontal.Stretch)

Gui:rect()

Gui:rect()
Gui:setFixedWidth(20)

Gui:rect()

Gui:endContainer()
```
In this example middle Rect will keep width of 20 pixels while first and last Rects will be stretched proportionally in the horizontal dimension to fill remaining parent's width.

### Fixed/Percent Size

Fixed size specifies a length in pixels.

It is possible for a child element to be defined to have a greater fixed size (height and/or width) than its parent Container. The part of the element outside its parent Container's extents will be clipped unless the Container has the scrollable property in that dimension set to true.

Percent size specifies length in terms of a percentage of the parent container's size.

In the case of a Container with a Horizontal or Vertical layout model specified, if a percent size is applied in the specified dimension, then that percentage will be calculated against the total parent size minus the size of all fixed sized elements and minus any padding in that dimension.

Example:
```lua
Gui:beginHorizontalContainer()
Gui:setFixedWidth(100)

Gui:rect()
Gui:setFixedWidth(20)

Gui:rect()
Gui:setPercentWidth(75)

Gui:rect()
Gui:setPercentWidth(25)

Gui:endContainer()
```
In this case the width of the second rect will be 60 and third one - 20 (instead of 75 and 25).

Any element size defined as a percent size may be reduced, possibly to 0, to prevent the total size of all elements in that dimension from exceeding their parent Container's size.

### Decorations

HmGui elements can be optionally assigned an outer margin (free space outside itself) and a border. These decorations are drawn outside of the element's enclosing rectangle. If the element is a Container, when fixed or percent sizes are specified for child elements, they are applied only to the current interior size of their parent Container.

Other decorations that Containers can have are inner padding (free space) inside its enclosing rectangle, and (for Containers with Horizontal or Vertical layout) spacing in between each child element in the dimension specified by that layout model.

Example:
```lua
Gui:beginHorizontalContainer()
Gui:setPadding(10)
Gui:setSpacing(5)

Gui:rect()
Gui:setFixedWidth(20)
Gui:setMargin(5, 5)

Gui:rect()
Gui:setPercentWidth(100)
Gui:setBorderWidth(2)

Gui:rect()
Gui:setFixedWidth(25)

Gui:endContainer()
```

## Style system

In addition to properties affecting layout, some elements may be assigned additional properties affecting their appearance. These are referred to as styling properties.

The styling properties to be implemented initially are:

- border-color
- background-color
- background-image

Color is specified as an ordered set of four floating-point values, with each value constrained to the range [0.0 : 1.0]. The four values of a Color are: Red, Green, Blue, Alpha (opacity).

Image is specified as an image format data blob whose contents have been read from a named file organized in one of the following graphics formats: JPG/JPEG, GIF, PNG.

---

### Property

Property can have one of the following types:
```rust
bool,
i8, u8, i16, u16, i32, u32, i64, u64, f32, f64,
Vec2, Vec3, Vec4, IVec2, IVec3, IVec4, UVec2, UVec3, UVec4, DVec2, DVec3, DVec4,
Box3, String, Font
```

Property name should have following format: `(<element_name>.)+<property-name>`. Example:
```
text.color
button.text-font
menu.ok-button.text-color
```

Properties can be defined in the Yaml configuration file. Example:
```yaml
prop.bool: true
prop.i8: -10
prop.u8: 63
prop.i16: -400
prop.u16: 1000
prop.i32: -100000
prop.u32: 630000
prop.i64: -10
prop.u64: 63
prop.f32: -10.69
prop.f64: 63.132
prop.vec2: [-10.2, 4.729]
prop.vec3: [-10.2, 4.729, 0.0]
prop.vec4: [-10.2, 4.729, 740, 44.6]
prop.ivec2: [-10, 4]
prop.ivec3: [-10, 4, 0]
prop.ivec4: [-10, 4, 740, 44]
prop.uvec2: [10, 4]
prop.uvec3: [10, 4, 0]
prop.uvec4: [10, 4, 740, 44]
prop.dvec2: [-10.2, 4.729]
prop.dvec3: [-10.2, 4.729, 0.0]
prop.dvec4: [-10.2, 4.729, 740, 44.6]
prop.box3: [[10.2, 4.729, 1.0], [740, 44.6, -1.0]]
prop.string: "Test"
prop.font: ["NovaMono", 14]
```

### Property mapping

This feature is used for automatically transferring a property value from one element to another. Let's take a `button` element as an example. The `button` element internally contains a `text` element that expects `text.font` and `text.color` properties. To preserve the benefits of Immediate Mode UI definition, properties cannot be set directly on an element. Instead, we define ("register") special variables that can be accessed from LuaJIT, and map these variables to the various properties of elements.

In our `button` example, we first register `button.text-font` and `button.text-color`, then we map these to the corresponding `text` properties of `button`. Imagine we have defined a `button` element in Lua:
```lua
-- registration in GuiEnums.lua
Enums.Gui.ButtonTextColorId = Gui:registerPropertyVec4("button.text-color", Vec4(1, 1, 1, 1), "text.color")

-- button element
function button(name)
Gui:mapProperty(Enums.Gui.ButtonTextColorId) -- copy this property value into the "text.color"
...
Gui:text(name)
...
end

-- somewhere later
Gui:clearStyle()
Gui:setPropertyColor(Enums.Gui.ButtonTextColorId, Color(0, 1, 0, 1))
Gui:button("My button")
```

### Property methods

HmGui provides several methods allowing Lua scripters to manage element properties:

- `Gui:getPropertyType(id)`: returns property type
- `Gui:mapProperty(id)`: copies property value to its mapped properties for the current following element. Should be used inside element function definition.
- `Gui:removeProperty(id)`: remove property from the current element style
- `Gui:registerProperty*(name, value, map_id)`: register a new property with optional id of the mapped property
- `Gui:setProperty*(id, value)`: set property value
- `Gui:getProperty*(id)`: get property value

`*` in `register/set/getProperty*()` methods is a property type substitution, i.e.: `setPropertyU32`, `getPropertyFont`, etc.

### Styling groups

There are 3 groups of the properties used for styling:

1. **Default properties**. These are hardcoded in the engine in the [core_properties.rs](core_properties.rs) file. These default property values are used if a value for a required property was not found in the other two groups. An autogenerated `GuiProperties` enum contains the IDs of all core properties.

2. **Themes**. A "theme" is a collection (possibly a large collection!) of element properties that replace the default property values. Specific named themes are defined in individual Yaml files in the `\theme` resource folder. The name of the file will be used as the theme name. All themes are automatically loaded by the engine during startup.

The following methods can be used in Lua scripts to manage themes:
   - `Gui:setTheme(name)`: to set theme
   - `Gui:clearTheme()`: remove theme and fall back to the default properties
Setting a theme is a heavy operation and should not be used too often, i.e. every frame.
Any property value applied by setting a theme that contains that property will be used only if it is not overridden by a per-element styling on that property (see group 3 below).

3. **Per-element styles**. This type of styling sets a property on an individual element. There are 2 ways to do it: either explictly in Lua code, or in a global style configuration file.

  - Example of direct styling in code:
```lua
Gui:clearStyle()
Gui:setPropertyColor(GuiProperties.ButtonTextColorId, Color(1, 0, 0, 1))
Gui:button("MyButton")
```
  - Example of styling through the global style configuration file `styles.yaml`:
```lua
Gui:setStyle(Enums.Gui.Styles.MyButtonStyleId)
Gui:button("MyButton")
```
During startup the engine loads all element styles from the `styles.yaml` configuration file in the resource folder. [Example](../../../test_data/styles.yaml).
IDs of element styles should be initialized in the [`GuiEnums.lua`](/script/Enums/GuiEnums.lua) file.

Element style scripting methods:
- `Gui:getStyleId(name)`: get style id by its name
- `Gui:setStyle(id)`: set style for the following element
- `Gui:clearStyle()`: clear element style so either theme or default properties will be used

### Custom properties

Scripters can define custom properties in the [`GuiEnums.lua`](/script/Enums/GuiEnums.lua) file. Example:
```lua
Enums.Gui.MItemTextColorId = Gui:registerPropertyVec4("menuitem.text-color", Vec4(1, 0, 1, 1), "text.color")
Enums.Gui.MenuItemTextColorId = Gui:registerPropertyVec4("menu.item-text-color", Vec4(1, 1, 0, 1), "menuitem.text-color")
Enums.Gui.MenuBorderWidthId = Gui:registerPropertyVec4("menu.border-width", 2, nil)
```

Then this property can be used in the custom element definition:
```lua
function menu(variants)
local borderWidth = Gui:getPropertyF32(Enums.Gui.MenuBorderWidthId)

Gui:mapProperty(Enums.Gui.MenuItemTextColorId) -- copy this property value into the "menuitem.text-color"
...
menu_item(variants[1])
...
end

function menu_item(name)
Gui:mapProperty(Enums.Gui.MItemTextColorId) -- copy this property value into the "text.color"
...
Gui:text(name)
...
end

```
