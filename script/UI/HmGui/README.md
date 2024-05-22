# UI Router
The UI Router is inspired by web development (specifically Vue.js) and divides the UI (HmGui) into:
- Pages
- Views
- Components

## Pages
Pages are the highest level of the UI. They can contain multiple Views.

## Views
Views are the second level of the UI. They contain UI components & UI logic. Views are updated by their parent page and only are so if the page is active.

## Components
Components parse HmGui code into an object to make them easily reusable and have their own state. They can also have their own functions if needed. Components are updated by their parent view and only are so if the view is active.

# Guide
An example for the UIRouter is provided with `~Tests/UIRouterTest`. It´s default components can be found in `~UI/HmGui/Components/Default`. The example uses an example page `~UI/HmGui/Pages/Example` and two example views which can be found in `~UI/HmGui/Views/Example`.

## Importing the UI Router
The UI Router can be imported and used like this:

```lua
local UIRouter = require('UI.HmGui.UICore.UIRouter')
```

## Functions
#### `UIRouter:addPage(page)` - Adds a page to the router so it can be used.

#### `UIRouter:clearCurrentPage()` - Resets the currently displayed page back to nil.

#### `UIRouter:setCurrentPage(name)` - Sets the currently displayed page. Default is nil.

#### `UIRouter:getCurrentPage()` - Returns the page object of the current page.

#### `UIRouter:getCurrentPageName()` - Returns the name of the current page.

#### `UIRouter:getLastPageName()` - Returns the name of the last page.

#### `UIRouter:getPage(name)` - Gets a page by name.

#### `UIRouter:getAvailablePages()` - Gets all pages that were added to the router and returns them in table.

## Usage
After you import the UI Router you need to import all pages that you want to use. Then you can set the view you want to display directly on the page, in this case "Main". After this is sorted you can add the page to the router and tell it to render the added page.
```lua
local UIRouter = require('UI.HmGui.UICore.UIRouter')
local UIPageExample = require('script.UI.HmGui.Pages.Example')

function AppState:onInit()
    -- set initial view
    UIPageExample:setView("Main")

    -- add page
    UIRouter:addPage(UIPageExample)
    UIRouter:setCurrentPage("Example")
end
```

You also need to route the input & update loops to the UIRouter, so the router can handle updating:
```lua
function AppState:onInput(dt)
    UIRouter:input(dt)
end

function AppState:onUpdate(dt)
    Gui:beginGui(self.resX, self.resY)
    UIRouter:update(dt)
    Gui:endGui(InputInstance)
end
```

### Pages
Pages are defined via the `UICore.Page` object which requires a name, and the views you want to add to the page.

- `addViewToPage(view)` is used to add a view to a page

```lua
---@type UIPage
local Example = UICore.Page {
    name = "Example"
}

local OtherView = require("UI.HmGui.Views.Example.OtherView")
Example:addViewToPage(OtherView)
local MainView = require("UI.HmGui.Views.Example.Main")
Example:addViewToPage(MainView)

return Example
```

### Views
Views are defined via the `UICore.View` object which requires a name.

```lua
---@type UIView
local Main = UICore.View {
    name = "Main"
}
```

In this file you can now start defining UI components which will be only updated and rendered if the view is active. The previously routed update and input loop are also accessible here so you can do runtime changes in the UI.

```lua
function Main:onInput() end
function Main:onUpdate(dt) end
```

This code shows how you can create a new container and add it to the view and also how you can modify a components state. To have a dynamic state displayed in the UI you need to define a getter function as seen in `getSomeState` you can then modify the state inside the view file with the provided update loop or just use a `GameState`. This getter function needs to be set instead of the variable value into the component ` UIComponent.Text { text = getSomeState }`. The component will use that function to get the last state of that variable and update the component state accordingly. In `switchToTitleScreen` you can see how to switch to another view. Here it is called from a button where a callback function can be defined. `UIComponent.Button { title = "Switch to other view", callback = switchToTitleScreen }`. Raw HmGui code is also allowed and can be used with the `UIComponent.RawInput` component if you need some flexibility and don´t want to create a new component for it.

- `addContent(component)` is used to add a component to a view. Components will be rendered in the order they are added to the view, you might have to take that into consideration.

```lua
local someState = 0

function Main:onUpdate(dt)
    someState = someState + dt
end

local function getSomeState()
    return someState
end

local function switchToTitleScreen()
    UIRouter:getCurrentPage():setView("Other_View")
end

---@type UIComponentContainer
local testContainerInner = UIComponent.Container {
    align = { AlignHorizontal.Center, AlignVertical.Center },
    layoutType = GuiLayoutType.Vertical,
    contents = {
        UIComponent.Button { title = "Switch to other view", callback = switchToTitleScreen },
        UIComponent.RawInput { fn = function()
            Gui:beginVerticalContainer()
            Gui:setVerticalAlignment(AlignVertical.Stretch)
            Gui:checkbox("Checkbox1", false)
            Gui:checkbox("Checkbox2", true)
            Gui:checkbox("Checkbox3", false)
            Gui:endContainer()
        end },
        UIComponent.Text { text = getSomeState }
    }
}

Main:addContent(testContainerInner)
```

## Default components
The default components are `Container`, `Text`, `Spacer`, `Button`, `RawInput` and `Window`. Check their code on how to use them. They are also fully documented by the language server.

## Defining a new component
I will use the text component as an example:

### Component class
Create a new component and overwrite the meta call so the (in this case) `Text:new()` function is called instead of `Text()`. This is mainly for convenience as this allows to use `UIComponent.Text{}` instead of `UIComponent.Text:new{}`:
```lua
local Text = {}
Text.__index = Text

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}
```

Now define the language server documentation for the object and the constructor:

```lua
---@class UIComponentText: UIComponent
---@field font string
---@field size number
---@field color Color
---@field text string
---@field render fun(self: UIComponentText) renders the text

---@class UIComponentTextConstructor
---@field font string
---@field size number
---@field color Color
---@field text string
```

After this we can define the `Text:new()` function. This will take args which are the previously defined constructor arguments:

```lua
---returns a text object
---@param args UIComponentTextConstructor
---@return UIComponentText|nil
function Text:new(args)
    if not args then
        return
    end
```

We then create our new text component object and also create a new state with the use of `UICore.ComponentState`, default values can also be defined here:
```lua
    local newText = {}
    newText.state = UICore.ComponentState {
        font = args.font or "Exo2Bold",
        size = args.size or 14,
        color = args.color or Color(1, 1, 1, 1),
        text = args.text or "undefined text",
    }
```

Now we will get to the most important part of component: the render function. This render function is the function that will run the HmGui code. Do not forget to pass self, so it can access the current state of the component within that function. The previously defined state can be accessed with `self.state`. The values of the state have to be accessed via a function call e.g. `self.state.font()` this function either returns the state value or runs the state value getter function if defined before e.g. `getSomeState`. If you don´t want your component to affect other components with styling, don´t forget to call `Gui:clearStyle()` at the end of your render function.

```lua
    newText.render = function(self)
        if self.state.font() then
            Gui:setProperty(GuiProperties.TextFont, Cache.Font(self.state.font(), self.state.size()))
        end

        Gui:setProperty(GuiProperties.TextColor, self.state.color())
        Gui:text(tostring(self.state.text()))

        Gui:clearStyle() -- clear style so it doesn´t affect other components
    end
```
At the end we will have to return the newly created component object:

```lua
    return newText
end
```

We finish the file with a setmetatable() so our meta changes we did first take effect and also add our component to the global UIComponent table:

```lua
setmetatable(Text, meta)

-- Add to global UIComponent table
---@type UIComponentTextConstructor
UIComponent.Text = Text

return Text
