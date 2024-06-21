local Switch = {}
Switch.__index = Switch

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class UIComponentSwitch: UIComponent
---@field visible boolean
---@field title string
---@field width number
---@field height number
---@field size ResponsiveSize -- if size is defined it will overwrite width, height
---@field sound SFXObject|nil
---@field color UIComponentSwitchColors
---@field borderWidth number
---@field borderColor Color
---@field font UIComponentFont
---@field toolTip UIComponentToolTip
---@field callback function
---@field render fun(self: UIComponentSwitch) renders the button

---@class UIComponentSwitchConstructor
---@field visible boolean
---@field title string
---@field width number
---@field height number
---@field size ResponsiveSize -- if size is defined it will overwrite width, height
---@field padding { paddingX: number, paddingY: number }|nil
---@field margin { marginX: number, marginY: number }|nil
---@field align { h: AlignHorizontal, v: AlignVertical }|{ h: AlignHorizontal.Default, v: AlignVertical.Default}
---@field textAlign { h: AlignHorizontal, v: AlignVertical }|{ h: AlignHorizontal.Center, v: AlignVertical.Center}
---@field color UIComponentSwitchColors
---@field font UIComponentFont
---@field toolTip string
---@field sound SFXObject|nil
---@field currentValue boolean
---@field callback function

---@class UIComponentSwitchColors
---@field text Color|nil
---@field background Color|nil
---@field highlight Color|nil

---@class UIComponentFont
---@field name string
---@field size number

---returns a button object
---@param args UIComponentSwitchConstructor
---@return UIComponentSwitch|nil
function Switch:new(args)
    if not args then
        return
    end

    local newSwitch = {}
    newSwitch.state = UICore.ComponentState {
        visible = args.visible,
        title = args.title,
        width = args.width or 40,
        height = args.height or 10,
        size = args.size,
        padding = args.padding,
        margin = args.margin or { 0, 10 },
        borderWidth = args.borderWidth or 2,
        borderColor = args.borderColor or Color(0.0, 0.0, 0.0, 1.0),
        align = args.align or { AlignHorizontal.Default, AlignVertical.Default },
        textAlign = args.textAlign or { AlignHorizontal.Center, AlignVertical.Center },
        color = {
            text = args.color and args.color.text or Color(0.7, 0.7, 0.7, 1.0),
            background = args.color and args.color.background or Color(0.3, 0.3, 0.3, 0.7),
            highlight = args.color and args.color.highlight or Color(0.95, 0.95, 0.95, 1.0),
            thumb = args.color and args.color.thumb or Color(1.0, 1.0, 1.0, 1.0)
        },
        font = args.font or { name = "Unageo-Medium", size = 12 },
        toolTip = UIComponent.ToolTip { text = args.toolTip },
        sound = args.sound,
        currentValue = args.currentValue or false,
        callback = args.callback or function() Log.Warn("undefined switch callback function: " .. tostring(args.title)) end
    }

    newSwitch.render = function(self)
        if not self.state.visible() then
            return
        end

        Gui:beginHorizontalContainer()
        Gui:setAlignment(self.state.textAlign()[1], self.state.textAlign()[2])

        if self.state.title and self.state.title() then
            Gui:text(self.state.title(), Cache.Font(self.state.font().name, self.state.font().size),
                self.state.color().text)
            Gui:setAlignment(self.state.textAlign()[1], self.state.textAlign()[2])
        end

        Gui:beginStackContainer()
        Gui:setAlignment(self.state.align()[1], self.state.align()[2])

        local isMouseOver = Gui:isMouseOver(FocusType.Mouse)

        if self.state.size then
            local size = self.state.size()
            Gui:setFixedSize(size.x, size.y)
        else
            if self.state.width then Gui:setFixedWidth(self.state.width()) end
            if self.state.height then Gui:setFixedHeight(self.state.height()) end
        end

        if self.state.padding then Gui:setPadding(self.state.padding()[1], self.state.padding()[2]) end
        if self.state.margin then Gui:setMargin(self.state.margin()[1], self.state.margin()[2]) end

        Gui:setBorderWidth(self.state.borderWidth())
        Gui:setBorderColor(self.state.borderColor())

        Gui:beginHorizontalContainer()
        Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Stretch)
        Gui:rect()

        if self.state.currentValue() then
            Gui:setBackgroundColor(self.state.color().background)
        else
            Gui:setBackgroundColor(self.state.color().thumb)
        end
        Gui:setPercentSize(50, 100)

        Gui:rect()

        if self.state.currentValue() then
            Gui:setBackgroundColor(self.state.color().thumb)
        else
            Gui:setBackgroundColor(self.state.color().background)
        end
        Gui:setPercentSize(50, 100)

        Gui:endContainer()
        Gui:endContainer()

        self.state.toolTip():render()

        Gui:endContainer()

        local switchClicked = isMouseOver and InputInstance:mouse():isPressed(MouseControl.Left)
        if switchClicked then
            local valueState = self.state.currentValue()
            self.state.currentValue = function() return not valueState end

            if self.state.sound then
                self.state.sound():Play(1.0)
            end

            self.state.callback(self.state.currentValue())
        end
    end

    return newSwitch
end

setmetatable(Switch, meta)

-- Add to global UIComponent table
---@type UIComponentSwitchConstructor
UIComponent.Switch = Switch

return Switch
