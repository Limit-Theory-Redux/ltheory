local RadioGroup = {}
RadioGroup.__index = RadioGroup

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class UIComponentRadioGroup: UIComponent
---@field visible boolean
---@field lastIndex number|nil
---@field selectedIndex number|1
---@field selections table
---@field width number
---@field height number
---@field size ResponsiveSize -- if size is defined it will overwrite width, height
---@field padding { paddingX: number, paddingY: number }|{ paddingX: 0, paddingY: 0 }
---@field margin { marginX: number, marginY: number }|{ marginX: 0, marginY: 0 }
---@field align { h: AlignHorizontal, v: AlignVertical }|{ h: AlignHorizontal.Default, v: AlignVertical.Default}
---@field textAlign { h: AlignHorizontal, v: AlignVertical }|{ h: AlignHorizontal.Left, v: AlignVertical.Center}
---@field font UIComponentFont
---@field color UIComponentRadioGroupColors
---@field sound SFXObject|nil
---@field callback fun(selectedIndex: number)
---@field render fun(self: UIComponentRadioGroup) renders the radio group

---@class UIComponentRadioGroupConstructor
---@field visible boolean
---@field lastIndex number|nil
---@field selectedIndex number|1
---@field selections table
---@field width number
---@field height number
---@field size ResponsiveSize -- if size is defined it will overwrite width, height
---@field padding { paddingX: number, paddingY: number }|nil
---@field margin { marginX: number, marginY: number }|nil
---@field align { h: AlignHorizontal, v: AlignVertical }|nil
---@field textAlign { h: AlignHorizontal, v: AlignVertical }|nil
---@field font UIComponentFont
---@field color UIComponentRadioGroupColors
---@field sound SFXObject|nil
---@field callback fun(selectedIndex: number)

---@class UIComponentRadioGroupColors
---@field text Color|nil
---@field background Color|nil
---@field highlight Color|nil
---@field clickArea { border: Color, checked: Color, notChecked: Color}|nil

---@class UIComponentFont
---@field name string
---@field size number

-- Component draws vertically list of radio button elements and allows to select one of them
-- using mouse. Index of the selected item is being sent into the `callback` function.
---@param args UIComponentRadioGroupConstructor
---@return UIComponentRadioGroup|nil
function RadioGroup:new(args)
    if not args then
        return
    end

    local newRadioGroup = {
        selectionChanged = false
    }
    newRadioGroup.state = UICore.ComponentState {
        visible = args.visible,
        lastIndex = args.lastIndex,
        selectedIndex = args.selectedIndex or 1,
        selections = args.selections or {},
        width = args.width,
        height = args.height,
        size = args.size,
        padding = args.padding,
        margin = args.margin,
        align = args.align or { AlignHorizontal.Default, AlignVertical.Default },
        textAlign = args.textAlign or { AlignHorizontal.Left, AlignVertical.Center },
        font = args.font or { name = "Exo2", size = 12 },
        clickArea = {
            size = { 10, 10 },
            borderWidth = 3,
        },
        color = {
            text = args.color and args.color.text or Color(1.0, 1.0, 1.0, 1.0),
            background = args.color and args.color.background or Color(0.0, 0.0, 0.0, 1.0),
            highlight = args.color and args.color.highlight or Color(0.1, 0.1, 0.1, 1.0),
            clickArea = {
                border = args.color and args.color.clickArea and args.color.clickArea.border or Color(1, 1, 1, 1),
                checked = args.color and args.color.clickArea and args.color.clickArea.checked or Color(0.1, 0.5, 1, 1),
                notChecked = args.color and args.color.clickArea and args.color.clickArea.notChecked or Color(0, 0, 0, 0),
            }
        },
        sound = args.sound,
        callback = args.callback or function(selectedIndex)
            Log.Warn("undefined radio group callback function: " .. args.selections)
        end
    }

    newRadioGroup.render = function(self)
        if not self.state.visible() or #self.state.selections() == 0 then
            return
        end

        self.selectionChanged = false

        Gui:beginVerticalContainer()
        Gui:setAlignment(self.state.align()[1], self.state.align()[2])
        Gui:setChildrenHorizontalAlignment(AlignHorizontal.Stretch)

        -- draw radio buttons
        for i, name in ipairs(self.state.selections()) do
            Gui:beginHorizontalContainer()
            Gui:setOpacity(1.0)

            local isMouseOver = Gui:isMouseOver(FocusType.Mouse)

            -- draw background depending on if radio button hovered or not
            if isMouseOver then
                Gui:setBackgroundColor(self.state.color().highlight)
            else
                Gui:setBackgroundColor(self.state.color().background)
            end

            if not self.state.lastIndex then
                self.state.lastIndex = self.state.selectedIndex
            end

            -- save selection if it was clicked
            local triggered = isMouseOver and InputInstance:mouse():isPressed(MouseControl.Left)
            if triggered then
                self.selectionChanged = self.state.selectedIndex ~= i
                self.state.selectedIndex = i
            end

            -- radio button text
            Gui:text(name, Cache.Font(self.state.font().name, self.state.font().size), self.state.color().text)
            Gui:setAlignment(self.state.textAlign()[1], self.state.textAlign()[2])

            Gui:spacer()

            -- radio button selection box
            Gui:rect()
            Gui:setFixedSize(self.state.clickArea().size[1], self.state.clickArea().size[2])
            Gui:setBorderWidth(self.state.clickArea().borderWidth)
            Gui:setVerticalAlignment(AlignVertical.Center)
            Gui:setBorderColor(self.state.color().clickArea.border)

            if self.state.selectedIndex == i then
                Gui:setBackgroundColor(self.state.color().clickArea.checked)
            else
                Gui:setBackgroundColor(self.state.color().clickArea.notChecked)
            end

            Gui:endContainer()

            if self.state.size then
                local size = self.state.size()
                Gui:setFixedSize(size.x, size.y)
            else
                if self.state.width then Gui:setFixedWidth(self.state.width()) end
                if self.state.height then Gui:setFixedHeight(self.state.height()) end
            end

            if self.state.padding then Gui:setPadding(self.state.padding()[1], self.state.padding()[2]) end
            if self.state.margin then Gui:setMargin(self.state.margin()[1], self.state.margin()[2]) end
        end

        Gui:endContainer()

        if self.selectionChanged then
            if self.state.sound then
                self.state.sound():Play(1.0)
            end

            self.state.callback(self.state.selectedIndex)
        end
    end

    return newRadioGroup
end

setmetatable(RadioGroup, meta)

-- Add to global UIComponent table
---@type UIComponentRadioGroupConstructor
UIComponent.RadioGroup = RadioGroup

return RadioGroup
