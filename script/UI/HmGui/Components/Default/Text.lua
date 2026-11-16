local Text = {}
Text.__index = Text

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

--- From this size (the text's own size, before any UI scale) `outline = true`
--- draws the 1 px outline as well as the halo; smaller text gets the halo alone.
Text.OutlineMinSize = 24

local NO_OUTLINE = Color(0, 0, 0, 0)

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
---@field padding { paddingX: number, paddingY: number }|nil
---@field margin { marginX: number, marginY: number }|nil
---@field color Color
---@field text string
---@field widthInLayout number
---@field heightInLayout number
---@field align table<AlignHorizontal, AlignVertical>
---@field outline boolean|"halo"|"outline"|nil keeps the text readable on any background (Gui:textOutlined): `true` = a 3 px halo, plus a 1 px outline from `Text.OutlineMinSize`; `"halo"` = the halo alone; `"outline"` = outline and halo at any size; nil/false = plain text
---@field outlineColor Color|nil the outline's colour, black by default
---@field haloColor Color|nil the halo's colour, black by default (alpha 1 = full strength)

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
        font = args.font or "Unageo-Medium",
        size = args.size or 14,
        padding = args.padding or { 0, 0 },
        margin = args.margin or { 0, 0 },
        color = args.color or Color(1, 1, 1, 1),
        text = args.text or "undefined text",
        width = args.width,
        height = args.height,
        widthInLayout = args.widthInLayout,
        heightInLayout = args.heightInLayout,
        align = args.align or { AlignHorizontal.Default, AlignVertical.Default },
        outline = args.outline or false,
        outlineColor = args.outlineColor or Color(0, 0, 0, 1),
        haloColor = args.haloColor or Color(0, 0, 0, 1)
    }

    newText.render = function(self)
        if not self.state.visible() then
            return
        end

        Gui:beginStackContainer()
        Gui:setAlignment(self.state.align()[1], self.state.align()[2])

        if self.state.width then Gui:setFixedWidth(self.state.width()) end
        if self.state.height then Gui:setFixedHeight(self.state.height()) end

        if self.state.padding then Gui:setPadding(self.state.padding()[1], self.state.padding()[2]) end
        if self.state.margin then Gui:setMargin(self.state.margin()[1], self.state.margin()[2]) end

        local text = tostring(self.state.text())
        local font = Cache.Font(self.state.font(), self.state.size())
        local outline = self.state.outline()
        if outline then
            Gui:setClipping(false) -- the halo reaches past the text's box
            local withOutline = outline == "outline" or (outline == true and self.state.size() >= Text.OutlineMinSize)
            Gui:textOutlined(text, font, self.state.color(), withOutline and self.state.outlineColor() or NO_OUTLINE,
                self.state.haloColor())
        else
            Gui:text(text, font, self.state.color())
        end
        Gui:setAlignment(self.state.align()[1], self.state.align()[2])

        Gui:endContainer()
    end

    return newText
end

setmetatable(Text, meta)

-- Add to global UIComponent table
---@type UIComponentTextConstructor
UIComponent.Text = Text

return Text
