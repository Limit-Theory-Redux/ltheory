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
---@field color table<{r: number, g: number, b:number, a:number}>
---@field text string
---@field render fun(self: UIComponentText) renders the text

---@class UIComponentTextConstructor
---@field font string
---@field size number
---@field color table<{r: number, g: number, b:number, a:number}>
---@field text string

---returns a text object
---@param args UIComponentTextConstructor
---@return UIComponentText|nil
function Text:new(args)
    if not args then
        return
    end

    local newText = {}
    newText.group = args.group
    newText.font = args.font or "Exo2Bold"
    newText.size = args.size or 14
    newText.color = args.color or Color(1, 1, 1, 1)
    newText.text = args.text or "undefined text"

    newText.render = function(self)
        if self.font then
            Gui:setPropertyFont(GuiProperties.TextFont, Cache.Font(self.font, self.size))
        end

        Gui:setPropertyColor(GuiProperties.TextColor, self.color)
        Gui:text(self.text)
    end

    return newText
end

setmetatable(Text, meta)

-- Add to global UIComponent table
---@type UIComponentTextConstructor
UIComponent.Text = Text

return Text
