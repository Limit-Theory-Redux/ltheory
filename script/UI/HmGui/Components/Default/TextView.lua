local TextView = {}
TextView.__index = TextView

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class UIComponentTextView: UIComponent
---@field visible boolean
---@field textData TextData
---@field widthInLayout number
---@field heightInLayout number
---@field width number
---@field height number
---@field align table<AlignHorizontal, AlignVertical>
---@field render fun(self: UIComponentTextView) renders the text
---@field showContainer boolean

---@class UIComponentTextViewConstructor
---@field visible boolean
---@field text string
---@field alignment TextAlignment
---@field style UIComponentTextViewStyle
---@field widthInLayout number
---@field heightInLayout number
---@field width number
---@field height number
---@field align table<AlignHorizontal, AlignVertical>
---@field showContainer boolean

---@class UIComponentTextViewStyle
---@field font UIComponentTextViewFont
---@field locale string|nil
---@field brush Color
---@field lineHeight number
---@field wordSpacing number
---@field letterSpacing number
---@field underline UIComponentTextViewDecoration|nil
---@field strikethrough UIComponentTextViewDecoration|nil

---@class UIComponentTextViewFont
---@field family string
---@field size number
---@field stretch number
---@field italic boolean
---@field weight number

---@class UIComponentTextViewDecoration
---@field offset number|nil
---@field size number|nil
---@field brush Color|nil

-- build text style from arguments style
---@param style UIComponentTextViewStyle
---@return TextStyle
local function buildStyle(style)
    local textStyle = TextStyle.Create()

    local fontFamily = "Exo2Bold"
    local fontSize = 14
    local brush = Color(1, 1, 1, 1)

    if style then
        if style.font then
            fontFamily = style.font.family or fontFamily
            fontSize = style.font.size or fontSize

            if style.font.stretch then
                textStyle:setFontStretch(style.font.stretch)
            end

            if style.font.italic then
                textStyle:setFontItalic(style.font.italic)
            end

            if style.font.weight then
                textStyle:setFontWeight(style.font.weight)
            end
        end

        if style.locale then
            textStyle:setLocale(style.locale)
        end

        brush = style.brush or brush

        if style.underline then
            textStyle:setUnderline(true)
            textStyle:setUnderlineOffset(style.underline.offset)
            textStyle:setUnderlineSize(style.underline.size)
            textStyle:setUnderlineBrush(style.underline.brush)
        end

        if style.strikethrough then
            textStyle:setStrikethrough(true)
            textStyle:setStrikethroughOffset(style.strikethrough.offset)
            textStyle:setStrikethroughSize(style.strikethrough.size)
            textStyle:setStrikethroughBrush(style.strikethrough.brush)
        end

        if style.lineHeight then
            textStyle:setLineHeight(style.lineHeight)
        end

        if style.wordSpacing then
            textStyle:setWordSpacing(style.wordSpacing)
        end

        if style.letterSpacing then
            textStyle:setLetterSpacing(style.letterSpacing)
        end
    end

    textStyle:setFontFamily("Exo2Bold")
    textStyle:setFontSize(fontSize)
    textStyle:setBrush(brush)

    return textStyle
end

-- returns a text object
---@param args UIComponentTextViewConstructor
---@return UIComponentTextView|nil
function TextView:new(args)
    if not args then
        return
    end

    local newTextView = {}
    newTextView.state = UICore.ComponentState {
        visible = args.visible,
        textData = TextData.Create(args.text or "undefined text", buildStyle(args.style), args.alignment or TextAlignment.Start),
        widthInLayout = args.widthInLayout,
        heightInLayout = args.heightInLayout,
        width = args.width,
        height = args.height,
        align = args.align or { AlignHorizontal.Default, AlignVertical.Default },
        showContainer = args.showContainer or function() return GameState.debug.metricsEnabled end,
        showContainerColor = Color((math.random() + math.random(50, 99)) / 100, (math.random() + math.random(50, 99)) / 100, (math.random() + math.random(50, 99)) / 100, .4)
    }

    newTextView.render = function(self)
        if not self.state.visible() then
            return
        end

        Gui:textView(self.state.textData())
        Gui:setAlignment(self.state.align()[1], self.state.align()[2])

        if self.state.width then Gui:setFixedWidth(self.state.width()) end
        if self.state.height then Gui:setFixedHeight(self.state.height()) end

        if self.state.showContainer() then
            Gui:setBorderColor(self.state.showContainerColor())
            Gui:setBorderWidth(1)
        end
    end

    return newTextView
end

setmetatable(TextView, meta)

-- Add to global UIComponent table
---@type UIComponentTextViewConstructor
UIComponent.TextView = TextView

return TextView
