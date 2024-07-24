local ListSelector = {}
ListSelector.__index = ListSelector

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class UIComponentListSelector: UIComponent
---@field visible boolean
---@field lastIndex number|nil
---@field selectedIndex number|1
---@field selections table
---@field layoutType GuiLayoutType|GuiLayoutType.Vertical
---@field width number
---@field height number
---@field size ResponsiveSize -- if size is defined it will overwrite width, height
---@field padding { paddingX: number, paddingY: number }|{ paddingX: 0, paddingY: 0 }
---@field margin { marginX: number, marginY: number }|{ marginX: 0, marginY: 0 }
---@field align { h: AlignHorizontal, v: AlignVertical }|{ h: AlignHorizontal.Default, v: AlignVertical.Default}
---@field textAlign { h: AlignHorizontal, v: AlignVertical }|{ h: AlignHorizontal.Left, v: AlignVertical.Center}
---@field font UIComponentFont
---@field color UIComponentListSelectorColors
---@field sound SFXObject|nil
---@field callback fun(selectedIndex: number)
---@field renderItem fun(self: UIComponentListSelector, selectedIndex: number)
---@field render fun(self: UIComponentListSelector) renders the list selector

---@class UIComponentListSelectorConstructor
---@field visible boolean
---@field lastIndex number|nil
---@field selectedIndex number|nil
---@field selections table
---@field layoutType GuiLayoutType|nil
---@field width number
---@field height number
---@field size ResponsiveSize -- if size is defined it will overwrite width, height
---@field padding { paddingX: number, paddingY: number }|nil
---@field margin { marginX: number, marginY: number }|nil
---@field align { h: AlignHorizontal, v: AlignVertical }|nil
---@field textAlign { h: AlignHorizontal, v: AlignVertical }|nil
---@field font UIComponentFont
---@field color UIComponentListSelectorColors
---@field sound SFXObject|nil
---@field renderItem fun(self: UIComponentListSelector, selectedIndex: number)
---@field callback fun(selectedIndex: number)

---@class UIComponentListSelectorColors
---@field text Color|nil
---@field background Color|nil
---@field highlight Color|nil
---@field selected Color|nil

---@class UIComponentFont
---@field name string
---@field size number

-- Component draws vertical or horizontal elements using `renderItem` function and
-- allows to select one of them using mouse. Index of the selected item is being sent into the `callback` function.
---@param args UIComponentListSelectorConstructor
---@return UIComponentListSelector|nil
function ListSelector:new(args)
    if not args then
        return
    end

    local newListSelector = {
        selectionChanged = false
    }
    newListSelector.state = UICore.ComponentState {
        lastIndex = args.lastIndex,
        selectedIndex = args.selectedIndex,
        selections = args.selections or {},
        visible = args.visible,
        layoutType = args.layoutType or GuiLayoutType.Vertical,
        width = args.width,
        height = args.height,
        size = args.size,
        padding = args.padding,
        margin = args.margin,
        align = args.align or { AlignHorizontal.Default, AlignVertical.Default },
        textAlign = args.textAlign or { AlignHorizontal.Left, AlignVertical.Center },
        font = args.font or { name = "Exo2", size = 12 },
        color = {
            text = args.color and args.color.text or Color(1.0, 1.0, 1.0, 1.0),
            background = args.color and args.color.background or Color(0.4, 0.4, 0.4, 1.0),
            highlight = args.color and args.color.highlight or Color(0.6, 0.6, 0.6, 1.0),
            selected = args.color and args.color.selected or nil,
        },
        sound = args.sound,
        renderItem = args.renderItem or function(selectedIndex)
            Log.Warn("undefined list selector item rendering function: " .. args.selections)
        end,
        callback = args.callback,
    }

    newListSelector.render = function(self)
        if not self.state.visible() or #self.state.selections() == 0 then
            return
        end

        self.selectionChanged = false

        Gui:beginContainer(self.state.layoutType())
        Gui:setAlignment(self.state.align()[1], self.state.align()[2])
        Gui:setChildrenHorizontalAlignment(AlignHorizontal.Stretch)

        if self.state.size then
            local size = self.state.size()
            Gui:setFixedSize(size.x, size.y)
        else
            if self.state.width then Gui:setFixedWidth(self.state.width()) end
            if self.state.height then Gui:setFixedHeight(self.state.height()) end
        end

        if self.state.padding then Gui:setPadding(self.state.padding()[1], self.state.padding()[2]) end
        if self.state.margin then Gui:setMargin(self.state.margin()[1], self.state.margin()[2]) end

        -- render elements
        for i, _ in ipairs(self.state.selections()) do
            self.state.renderItem(self, i)

            if self.state.layoutType() == GuiLayoutType.Horizontal then
                Gui:setVerticalAlignment(AlignVertical.Stretch)
            elseif self.state.layoutType() == GuiLayoutType.Vertical then
                Gui:setHorizontalAlignment(AlignHorizontal.Stretch)
            end

            local isMouseOver = Gui:isMouseOver(FocusType.Mouse)

            -- highlight elements depending on if it selected, hovered over or not
            if isMouseOver then
                Gui:setBackgroundColor(self.state.color().highlight)
            elseif self.state.selectedIndex == i and self.state.color().selected then
                Gui:setBackgroundColor(self.state.color().selected)
            else
                Gui:setBackgroundColor(self.state.color().background)
            end

            if not self.state.lastIndex then
                self.state.lastIndex = self.state.selectedIndex
            end

            -- save selection if it was clicked
            local elementClicked = isMouseOver and Input:mouse():isPressed(MouseControl.Left)
            if elementClicked then
                self.selectionChanged = self.state.selectedIndex ~= i
                self.state.selectedIndex = i
            end
        end

        Gui:endContainer()

        if self.selectionChanged then
            if self.state.sound then
                self.state.sound():play(1.0)
            end

            if self.state.callback then
                self.state.callback(self.state.selectedIndex)
            end
        end
    end

    return newListSelector
end

setmetatable(ListSelector, meta)

-- Add to global UIComponent table
---@type UIComponentListSelectorConstructor
UIComponent.ListSelector = ListSelector

return ListSelector
