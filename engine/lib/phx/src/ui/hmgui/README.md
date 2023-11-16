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

- **alignment** - telling an element to attach one or more of its four sides (Left, Right, Top and Bottom) to its parent's side, center element or stretch
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
All three Rect widgets will be drawn in a horizontal row, in order, beginning in the upper-left interior corner of their parent Container, and each Rect will retain its fixed height and width.

### Alignment

Alignment is a property that can be used to position an element relatively to any of the four sides of its parent Container or to the element beside the selected child element in the designated direction.

Each alignment dimension (Horizontal, Vertical) has four parameters available: Center, Left/Top (default), Right/Bottom and Stretch.

User can set alignment for both dimensions independently.

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
These three Rects will overlap. There will be one tall, skinny Rect on the left; one short, wide Rect at the top; and one Rect that fills the Container.

Additionally Container has an alignment parameter that can be applied to its children. For example, to align children in the Horizontal container to the right user can do the following:
```lua
Gui:beginHorizontalContainer()
Gui:setFixedSize(100, 100)
Gui:setChildrenHorizontalAlignment(AlignHorizontal.Stretch)

Gui:rect()
Gui:setFixedWidth(20)

Gui:rect()
Gui:setFixedHeight(20)

Gui:rect()
Gui:setFixedHeight(20)

Gui:endContainer()
```
All rectangles in this example will be moved to the left.

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

HmGui Containers can be optionally assigned an outer margin (free space outside itself) and a border. These decorations are drawn outside of the Container's enclosing rectangle so that when fixed or percent sizes are specified for child elements, they are applied only to the current interior size of their parent Container.

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

## Styling

In addition to properties affecting layout, some elements may be assigned additional properties affecting their appearance. These are referred to as styling properties.

The styling properties to be implemented initially are:

- border-color
- background-color
- background-image

Color is specified as an ordered set of four floating-point values, with each value constrained to the range [0.0 : 1.0]. The four values of a Color are: Red, Green, Blue, Alpha (opacity).

Image is specified as an image format data blob whose contents have been read from a named file organized in one of the following graphics formats: JPG/JPEG, GIF, PNG.
