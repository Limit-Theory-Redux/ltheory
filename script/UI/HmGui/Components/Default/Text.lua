local Text = {}
Text.__index = Text

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

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

---returns a text object
---@param args UIComponentTextConstructor
---@return UIComponentText|nil
function Text:new(args)
    if not args then
        return
    end

    local newText = {}
    newText.state = UICore.ComponentState {
        font = args.font or "Exo2Bold",
        size = args.size or 14,
        color = args.color or Color(1, 1, 1, 1),
        text = args.text or "undefined text",
    }

    newText.render = function(self)
        if self.state.font() then
            Gui:setPropertyFont(GuiProperties.TextFont, Cache.Font(self.state.font(), self.state.size()))
        end

        Gui:setPropertyColor(GuiProperties.TextColor, self.state.color())
        Gui:text(tostring(self.state.text()))

        Gui:clearStyle() -- clear style so it doesn´t affect other components
    end

    return newText
end

setmetatable(Text, meta)

-- Add to global UIComponent table
---@type UIComponentTextConstructor
UIComponent.Text = Text

return Text
