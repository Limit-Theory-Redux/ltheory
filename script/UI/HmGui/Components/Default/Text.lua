local Text = {}
Text.__index = Text

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class UIComponentText: UIComponent
---@field visible boolean
---@field font string
---@field size number
---@field color Color
---@field text string
---@field widthInLayout number
---@field heightInLayout number
---@field render fun(self: UIComponentText) renders the text

---@class UIComponentTextConstructor
---@field visible boolean
---@field font string
---@field size number
---@field color Color
---@field text string
---@field widthInLayout number
---@field heightInLayout number
---@field align table<AlignHorizontal, AlignVertical>

---returns a text object
---@param args UIComponentTextConstructor
---@return UIComponentText|nil
function Text:new(args)
    if not args then
        return
    end

    local newText = {}
    newText.state = UICore.ComponentState {
        visible = args.visible,
        font = args.font or "Exo2Bold",
        size = args.size or 14,
        color = args.color or Color(1, 1, 1, 1),
        text = args.text or "undefined text",
        widthInLayout = args.widthInLayout,
        heightInLayout = args.heightInLayout,
        align = args.align or { AlignHorizontal.Default, AlignVertical.Default }
    }

    newText.render = function(self)
        if not self.state.visible() then
            return
        end

        Gui:text(tostring(self.state.text()), Cache.Font(self.state.font(), self.state.size()), self.state.color())
        Gui:setAlignment(self.state.align()[1], self.state.align()[2])
    end

    return newText
end

setmetatable(Text, meta)

-- Add to global UIComponent table
---@type UIComponentTextConstructor
UIComponent.Text = Text

return Text
