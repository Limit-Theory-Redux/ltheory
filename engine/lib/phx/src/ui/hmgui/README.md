# HmGui

HmGui (hybrid mode GUI) is a GUI framework that combines best parts of the immediate and retain mode GUIs while avoiding their downsides.

Core part of the GUI is a widget. It represents rectangular area with layout and styling parameters. There are 4 types of widgets: container, image, text and rectangle. These types have their own parameters for more detailed layouting and styling.

Important part of the layout system is docking and fixed/percent size.

Container has child widgets that can be arranged in 3 different ways: stacked - widgets go on top of each other, horizontal - lined horizontally, and vertical - lined vertically.

Using all these features user can build reach GUI systems.

## Layout system

Layout system is based on the layout type (Stack, Horizontal, Vertical) of the container, docking (Left, Top, Right, Bottom) of the widget and container's children, size specification (Fixed, Percent) and decorations (margin, border, padding and spacing). All these parameters define how elements are placed on the screen.

### Layout type

Container can be one of three types: Stack, Horizontal or Vertical. It also can specify children [docking](#docking), internal padding and spacing between children.

There are 2 rules of how children docking is applied to the individual child:
1. For both dimensions of stack layout, vertical dimension of horizontal layout and horizontal dimension of vertical one, we have following behavior: children docking is applied to the widget only if in that dimension child doesn't have any docking. E.g. if in vertical container child widget doesn't have neither left nor right docking then container's children docking is applied.
2. For horizontal and vertical containers in their main dimensions we have different situation: container's children docking has priority over individual widgets own docking and even fixed/percent sizes (?).

Example:
```lua
Gui:beginHorizontalContainer()
Gui:setFixedSize(100, 100)
Gui:setChildrenDocking(Docking.StretchHorizontal)

Gui:rect()
Gui:setFixedHeight(20)

Gui:rect()
Gui:setFixedHeight(20)

Gui:rect()
Gui:setFixedHeight(20)

Gui:endContainer()
```
All 3 widgets will be organized in a row, evenly stretched in the horizontal dimension and centered vertically.

### Docking

There are 4 docking flags that can be combined: Left, Right, Top and Bottom.
Left docking can be combined with right for horizontal stretching, and top can be combined with bottom for the vertical one.

Stretching for the individual widget in the dimension (horizontal or vertical) has priority over fixed/percent size in that dimension. So if user specifies both then stretching will be used.
Behavior of stretching of the container's children docking is described [above](#layout-type).

Example:
```lua
Gui:beginStackContainer()
Gui:setFixedSize(100, 100)

Gui:rect()
Gui:setFixedWidth(20)
Gui:setDocking(Docking.StretchVertical)

Gui:rect()
Gui:setFixedHeight(20)
Gui:setDocking(Docking.StretchHorizontal)

Gui:rect()
Gui:setDocking(Docking.StretchAll)

Gui:endContainer()
```

### Fixed/Percent size

Fixed size specifies length in pixels (f32). It is possible situation when chid has bigger size than its parent.

Percent size specifies length in percent of the parent size. In case of horizontal and vertical containers if percent size applied to the main dimension then percentage will be calculated from the parent size minus size of all fixed sized elements (should it be like this?).

To make percent size work properly parent container should have either stretching docking or fixed/percent size in that dimension.

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
In this case width of the second rect will be 60 and third one - 20 (instead of 75 and 25).

If percentage sum became more than 100% then children will be bigger than parent.

### Decorations

Each widget can have outer margin (free space around itself) and border. These elements lay outside of the widgets rectangle, so when user specify fixed or percentage size, it's a size of the widget rectangle without margin and border.

Container can have inner padding (free space) between its widget rectangle and children.
For the horizontal and vertical containers user also can specify spacing between its children.

Example:
```lua
Gui:beginHorizontalContainer()
Gui:setPadding(10)
Gui:setSpacing(5)

Gui:rect()
Gui:setFixedWidth(20)

Gui:rect()
Gui:setPercentWidth(75)

Gui:rect()
Gui:setPercentWidth(25)

Gui:endContainer()
```
