local Dropdown = {}
Dropdown.__index = Dropdown

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class UIComponentDropdown: UIComponent
---@field selectedIndex number|nil
---@field selections table
---@field visible boolean
---@field width number
---@field height number
---@field padding { paddingX: number, paddingY: number }|{ paddingX: 0, paddingY: 0 }
---@field margin { marginX: number, marginY: number }|{ marginX: 0, marginY: 0 }
---@field align { h: AlignHorizontal, v: AlignVertical }|{ h: AlignHorizontal.Default, v: AlignVertical.Default}
---@field textAlign { h: AlignHorizontal, v: AlignVertical }|{ h: AlignHorizontal.Left, v: AlignVertical.Center}
---@field font UIComponentFont
---@field color UIComponentDropdownColors
---@field sound SFXObject|nil
---@field toolTip UIComponentToolTip
---@field callback function
---@field render fun(self: UIComponentDropdown) renders the dropdown

---@class UIComponentDropdownConstructor
---@field lastIndex number|nil
---@field selectedIndex number|nil
---@field selections table
---@field visible boolean
---@field width number
---@field height number
---@field padding { paddingX: number, paddingY: number }|nil
---@field margin { marginX: number, marginY: number }|nil
---@field align { h: AlignHorizontal, v: AlignVertical }|nil
---@field textAlign { h: AlignHorizontal, v: AlignVertical }|nil
---@field font UIComponentFont
---@field toolTip string
---@field color UIComponentDropdownColors
---@field sound SFXObject|nil
---@field callback function

---@class UIComponentDropdownColors
---@field text Color|nil
---@field background Color|nil
---@field highlight Color|nil
---@field clickArea { border: Color, checked: Color, notChecked: Color}|nil

---@class UIComponentFont
---@field name string
---@field size number

-- Dropdown component. Set selections list to choose from.
---@param args UIComponentDropdownConstructor
---@return UIComponentDropdown|nil
function Dropdown:new(args)
    if not args then
        return
    end

    local newDropdown = {
        isDroppedDown = false,
        dropdownIcon = Tex2D.Load("./res/ui/dropdown.png"),
    }
    newDropdown.state = UICore.ComponentState {
        visible = args.visible,
        lastIndex = args.lastIndex,
        selectedIndex = args.selectedIndex or 1,
        selections = args.selections or {},
        width = args.width,
        height = args.height,
        padding = args.padding,
        margin = args.margin,
        align = args.align or { AlignHorizontal.Default, AlignVertical.Default },
        textAlign = args.textAlign or { AlignHorizontal.Left, AlignVertical.Center },
        font = args.font or { name = "Exo2", size = 12 },
        color = {
            text = args.color and args.color.text or Color(1.0, 1.0, 1.0, 1.0),
            background = args.color and args.color.background or Color(0.7, 0.7, 0.7, 1.0),
            highlight = args.color and args.color.highlight or Color(0.9, 0.9, 0.9, 1.0),
        },
        toolTip = UIComponent.ToolTip { text = args.toolTip },
        sound = args.sound,
        callback = args.callback or function(selectedIndex)
            Log.Warn("undefined dropdown callback function: " .. tostring(selectedIndex))
        end,
        listSelector = UIComponent.ListSelector {
            selections = args.selections,
            align = { AlignHorizontal.Stretch, AlignVertical.Default },
            color = { selected = Color(0.8, 0.8, 0.8, 1) },
            renderItem = function(self, selectedIndex)
                Gui:beginStackContainer()
                Gui:setPadding(5, 5)

                local text = self.state.selections()[selectedIndex]
                Gui:text(text, Cache.Font(self.state.font().name, self.state.font().size), self.state.color().text)

                Gui:endContainer()
            end,
        },
        dropdownComponent = UIComponent.ScrollArea {
            borderWidth = 2,
            height = 90,
        },
    }

    newDropdown.render = function(self)
        if not self.state.visible() or #self.state.selections() == 0 then
            return
        end

        -- Set the dropdown text that is displayed when no selection has yet been made
        local selectedText = "--Select--"
        if self.state.listSelector().state.selectedIndex then
            selectedText = self.state.selections()[self.state.listSelector().state.selectedIndex]
        end

        -- main dropdown area
        Gui:beginHorizontalContainer()
        Gui:setAlignment(self.state.align()[1], self.state.align()[2])
        Gui:setBorderWidth(1)
        Gui:setBorderColor(Color(1, 1, 1, 1))
        Gui:setPadding(1, 1)
        Gui:setSpacing(1)

        -- selected text
        Gui:beginStackContainer()
        Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Center)
        Gui:setBackgroundColor(self.state.color().background)
        Gui:setPadding(5, 5)

        Gui:text(selectedText, Cache.Font(self.state.font().name, self.state.font().size), self.state.color().text)
        Gui:setAlignment(AlignHorizontal.Left, AlignVertical.Center)

        Gui:endContainer()

        local isMouseOverText = Gui:isMouseOver(FocusType.Mouse)

        self.state.toolTip():render()

        -- drop down button
        Gui:image(self.dropdownIcon)
        Gui:setAlignment(AlignHorizontal.Right, AlignVertical.Center)

        -- highlight dropdown button if hovered
        local isMouseOverButton = Gui:isMouseOver(FocusType.Mouse)
        if isMouseOverButton then
            Gui:setBackgroundColor(self.state.color().highlight)
        else
            Gui:setBackgroundColor(self.state.color().background)
        end

        Gui:endContainer() -- end of main dropdown area

        -- store main area size to make dropdown list the same width
        local mainContainerSize = Gui:elementSize()

        local isMouseOverMainContainer = Gui:isMouseOver(FocusType.Mouse)

        -- show/hide dropdown list if either text area or dropdown button was clicked
        local dropdownClicked = (isMouseOverText or isMouseOverButton) and
            Input:mouse():isPressed(MouseControl.Left)
        if dropdownClicked then
            if self.state.sound then
                self.state.sound():play(1.0)
            end

            self.isDroppedDown = not self.isDroppedDown
        end

        -- dropdown list layer
        if self.isDroppedDown then
            -- TODO: do this at the initialization step
            self.state.dropdownComponent().state.contents = function() return { self.state.listSelector() } end

            Gui:beginLayerBelow()

            self.state.dropdownComponent():render()
            Gui:setFixedWidth(mainContainerSize.x) -- make dropdown list the same width as a main area

            -- hide drop down list if user clicked outside list itself
            local clickedOutside = not isMouseOverText and not isMouseOverButton and not isMouseOverMainContainer
                and not Gui:isMouseOver(FocusType.Mouse) and Input:mouse():isPressed(MouseControl.Left)
            if clickedOutside then
                self.isDroppedDown = false
            end

            Gui:endLayer()

            if self.state.listSelector().selectionChanged then
                self.state.callback(self.state.listSelector().state.selectedIndex)

                -- hide drop down list if user made selection
                self.isDroppedDown = false
            end
        end
    end

    return newDropdown
end

setmetatable(Dropdown, meta)

-- Add to global UIComponent table
---@type UIComponentDropdownConstructor
UIComponent.Dropdown = Dropdown

return Dropdown
