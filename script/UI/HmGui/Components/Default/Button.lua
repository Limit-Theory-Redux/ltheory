local Button = {}
Button.__index = Button

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class UIComponentButton: UIComponent
---@field visible boolean
---@field title string
---@field width number
---@field height number
---@field size ResponsiveSize -- if size is defined it will overwrite width, height
---@field sound SFXObject|nil
---@field color UIComponentButtonColors
---@field font UIComponentFont
---@field toolTip UIComponentToolTip
---@field render fun(self: UIComponentButton) renders the button

---@class UIComponentButtonConstructor
---@field visible boolean
---@field title string
---@field width number
---@field height number
---@field size ResponsiveSize -- if size is defined it will overwrite width, height
---@field padding { paddingX: number, paddingY: number }|nil
---@field margin { marginX: number, marginY: number }|nil
---@field align { h: AlignHorizontal, v: AlignVertical }|{ h: AlignHorizontal.Default, v: AlignVertical.Default}
---@field textAlign { h: AlignHorizontal, v: AlignVertical }|{ h: AlignHorizontal.Center, v: AlignVertical.Center}
---@field color UIComponentButtonColors
---@field font UIComponentFont
---@field toolTip string
---@field sound SFXObject|nil
---@field callback function

---@class UIComponentButtonColors
---@field text Color|nil
---@field background Color|nil
---@field highlight Color|nil

---@class UIComponentFont
---@field name string
---@field size number

---returns a button object
---@param args UIComponentButtonConstructor
---@return UIComponentButton|nil
function Button:new(args)
    if not args then
        return
    end

    local newButton = {}
    newButton.state = UICore.ComponentState {
        visible = args.visible,
        title = args.title,
        width = args.width,
        height = args.height,
        size = args.size,
        padding = args.padding,
        margin = args.margin,
        align = args.align or { AlignHorizontal.Default, AlignVertical.Default },
        textAlign = args.textAlign or { AlignHorizontal.Center, AlignVertical.Center },
        color = {
            text = args.color and args.color.text or Color(1.0, 1.0, 1.0, 1.0),
            background = args.color and args.color.background or Color(0.0, 0.0, 0.0, 1.0),
            highlight = args.color and args.color.highlight or Color(0.3, 0.3, 0.3, 1.0)
        },
        font = args.font or { name = "Exo2", size = 12 },
        toolTip = UIComponent.ToolTip { text = args.toolTip },
        sound = args.sound,
        callback = args.callback or function() Log.Warn("undefined button callback function: " .. tostring(args.title)) end
    }

    newButton.render = function(self)
        if not self.state.visible() then
            return
        end

        Gui:beginStackContainer()
        Gui:setAlignment(self.state.align()[1], self.state.align()[2])
        Gui:setOpacity(1.0)

        local isMouseOver = Gui:isMouseOver(FocusType.Mouse)
        if isMouseOver then
            Gui:setBackgroundColor(self.state.color().highlight)
        else
            Gui:setBackgroundColor(self.state.color().background)
        end


        if self.state.size then
            local size = self.state.size()
            Gui:setFixedSize(size.x, size.y)
        else
            if self.state.width then Gui:setFixedWidth(self.state.width()) end
            if self.state.height then Gui:setFixedHeight(self.state.height()) end
        end

        if self.state.padding then Gui:setPadding(self.state.padding()[1], self.state.padding()[2]) end
        if self.state.margin then Gui:setMargin(self.state.margin()[1], self.state.margin()[2]) end

        -- no need for an if check, since we always have a default defined
        Gui:text(self.state.title(), Cache.Font(self.state.font().name, self.state.font().size), self.state.color().text)
        Gui:setAlignment(self.state.textAlign()[1], self.state.textAlign()[2])

        Gui:endContainer()

        self.state.toolTip():render()

        local buttonClicked = isMouseOver and Input:mouse():isPressed(MouseControl.Left)
        if buttonClicked then
            if self.state.sound then
                self.state.sound():play(1.0)
            end

            self.state.callback()
        end
    end

    return newButton
end

setmetatable(Button, meta)

-- Add to global UIComponent table
---@type UIComponentButtonConstructor
UIComponent.Button = Button

return Button
