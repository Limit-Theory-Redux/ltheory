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
---@field padding table<number, number>
---@field margin table<number, number>
---@field color UIComponentButtonColors
---@field font UIComponentFont
---@field callback function
---@field render fun(self: UIComponentButton) renders the button

---@class UIComponentButtonConstructor
---@field visible boolean
---@field title string
---@field width number
---@field height number
---@field padding table<number, number>
---@field margin table<number, number>
---@field color UIComponentButtonColors
---@field font UIComponentFont
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
        margin = args.margin,
        padding = args.padding,
        color = {
            text = args.color and args.color.text or Color(1.0, 1.0, 1.0, 1.0),
            background = args.color and args.color.background or Color(1.0, 1.0, 1.0, 1.0), -- stubbed but not yet implemented in HmGui
            highlight = args.color and args.color.highlight or Color(1.0, 1.0, 1.0, 1.0)    -- stubbed but not yet implemented in HmGui
        },
        font = args.font or { name = "Unageo-Regular", size = 13 },
        callback = args.callback or function() Log.Warn("undefined button callback function: " .. args.title) end
    }

    newButton.render = function(self)
        if not self.state.visible() then
            return
        end

        -- no need for an if check, since we always have a default defined
        Gui:setPropertyFont(GuiProperties.TextFont, Cache.Font(self.state.font().name, self.state.font().size))
        Gui:setPropertyColor(GuiProperties.ButtonTextColor, self.state.color().text)
        Gui:setPropertyColor(GuiProperties.ButtonBackgroundColor, self.state.color().background)
        Gui:setPropertyColor(GuiProperties.ButtonHighlightColor, self.state.color().highlight)

        if Gui:button(self.state.title()) then
            -- hardcoding the sound
            if Config.audio.sounds.click then
                Config.audio.sounds.click:Play(1.0)
            end

            self.state.callback()
        end

        if self.state.width then Gui:setFixedWidth(self.state.width()) end
        if self.state.height then Gui:setFixedHeight(self.state.height()) end

        if self.state.padding then Gui:setPadding(self.state.padding()[1], self.state.padding()[2]) end
        if self.state.margin then Gui:setMargin(self.state.margin()[1], self.state.margin()[2]) end

        Gui:clearStyle() -- clear style so it doesn´t affect other components
    end

    return newButton
end

setmetatable(Button, meta)

-- Add to global UIComponent table
---@type UIComponentButtonConstructor
UIComponent.Button_MainMenu = Button

return Button
