local Text = {}
Text.__index = Text

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class Text
---@field render function renders the text

---returns a text object
---@param args {font: string, size: number, color: table<{r: number, g: number, b:number, a:number}>, text: string}
---@return Text|nil
function Text:new(args)
    if not args then
        return
    end

    local newText = {}
    newText.group = args.group
    newText.font = args.font or "Exo2Bold"
    newText.size = args.size or 14
    newText.color = args.color or { 1, 1, 1, 1 }
    newText.text = args.text or "undefined text"

    newText.render = function ()
        if newText.font then
            Gui:pushFont(Cache.Font(newText.font, newText.size))
        end

        Gui:textColored(newText.text, newText.color.r, newText.color.g, newText.color.b, newText.color.a)
    end

    return newText
end

setmetatable(Text, meta)

-- Add to global UIComponent table
UIComponent.Text = Text

return Text
