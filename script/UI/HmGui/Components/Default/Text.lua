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
local BLACK = Color(0, 0, 0, 1)

--- The outlined text variant: the font's outlined glyphs (`Gui:textOutlined`)
--- with a halo, and a 1 px outline as well when `mode` is "outline", or
--- `true` from `Text.OutlineMinSize` on.
local function addOutlined(text, font, size, color, mode, outlineColor, haloColor)
    local withOutline = mode == "outline" or (mode == true and size >= Text.OutlineMinSize)
    Gui:textOutlined(text, font, color, withOutline and outlineColor or NO_OUTLINE, haloColor)
end

--- Adds a text widget as `outline = true` draws it (a black halo, plus a
--- black 1 px outline from `Text.OutlineMinSize`), for text placed by hand
--- that reads like every outlined text on any background. `glow` adds a
--- soft light in its colour round the glyphs, under the halo, so the text
--- stays readable (`Gui:setTextGlow`; alpha = strength). Halo and glow
--- reach past the widget: turn clipping off on the container around it.
---@param text string
---@param font Font
---@param size number the font's size, real pixels
---@param color Color
---@param glow Color|nil
function Text.addOutlined(text, font, size, color, glow)
    addOutlined(text, font, size, color, true, BLACK, BLACK)
    if glow and glow.a > 0 then
        Gui:setTextGlow(glow)
    end
end

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
---@field wrap number|nil break the text into lines no wider than this (pixels), between words; a "\n" in the text always starts a new line
---@field textAlign AlignHorizontal|nil where each line sits: Left, Right or centred (the default)

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
        outlineColor = args.outlineColor or BLACK,
        haloColor = args.haloColor or BLACK,
        wrap = args.wrap,
        textAlign = args.textAlign
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
            addOutlined(text, font, self.state.size(), self.state.color(), outline, self.state.outlineColor(),
                self.state.haloColor())
        else
            Gui:text(text, font, self.state.color())
        end
        Gui:setAlignment(self.state.align()[1], self.state.align()[2])
        if self.state.wrap then
            Gui:setMaxWidth(self.state.wrap())
            Gui:setTextWrap(true)
        end
        if self.state.textAlign then
            Gui:setTextAlign(self.state.textAlign())
        end

        Gui:endContainer()
    end

    return newText
end

setmetatable(Text, meta)

-- Add to global UIComponent table
---@type UIComponentTextConstructor
UIComponent.Text = Text

return Text
