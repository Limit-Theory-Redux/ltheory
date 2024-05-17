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
---@field sound SFXObject|nil
---@field color UIComponentButtonColors
---@field font UIComponentFont
---@field callback function
---@field render fun(self: UIComponentButton) renders the button

---@class UIComponentButtonConstructor
---@field visible boolean
---@field title string
---@field width number
---@field height number
---@field padding { paddingX: number, paddingY: number }|nil
---@field margin { marginX: number, marginY: number }|nil
---@field textAlign { h: AlignHorizontal, v: AlignVertical }|{ h: AlignHorizontal.Center, v: AlignVertical.Center}
---@field color UIComponentButtonColors
---@field font UIComponentFont
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
        padding = args.padding,
        margin = args.margin,
        textAlign = args.textAlign or { AlignHorizontal.Center, AlignVertical.Center },
        color = {
            text = args.color and args.color.text or Color(1.0, 1.0, 1.0, 1.0),
            background = args.color and args.color.background or Color(0.0, 0.0, 0.0, 1.0),
            highlight = args.color and args.color.highlight or Color(0.1, 0.1, 0.1, 1.0)
        },
        font = args.font or { name = "Exo2", size = 12 },
        sound = args.sound,
        callback = args.callback or function() Log.Warn("undefined button callback function: " .. args.title) end
    }

    newButton.render = function(self)
        if not self.state.visible() then
            return
        end

        Gui:setProperty(GuiProperties.Opacity, 1.0)
        Gui:setProperty(GuiProperties.BackgroundColor, self.state.color().background)
        Gui:setProperty(GuiProperties.HighlightColor, self.state.color().highlight)
        Gui:beginStackContainer()

        local buttonClicked = Gui:isMouseOver(FocusType.Mouse) and InputInstance:mouse():isPressed(MouseControl.Left)

        -- no need for an if check, since we always have a default defined
        Gui:setProperty(GuiProperties.TextFont, Cache.Font(self.state.font().name, self.state.font().size))
        Gui:setProperty(GuiProperties.TextColor, self.state.color().text)

        Gui:text(self.state.title())
        Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)

        Gui:endContainer()

        if self.state.width then Gui:setFixedWidth(self.state.width()) end
        if self.state.height then Gui:setFixedHeight(self.state.height()) end

        if self.state.padding then Gui:setPadding(self.state.padding()[1], self.state.padding()[2]) end
        if self.state.margin then Gui:setMargin(self.state.margin()[1], self.state.margin()[2]) end

        Gui:clearStyle() -- clear style so it doesn´t affect other components

        if buttonClicked then
            if self.state.sound then
                self.state.sound():Play(1.0)
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
