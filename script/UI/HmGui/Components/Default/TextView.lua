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
---@field editable boolean
---@field backgroundColor Color
---@field widthInLayout number
---@field heightInLayout number
---@field margin { marginX: number, marginY: number }
---@field borderWidth number
---@field borderColor Color
---@field width number
---@field height number
---@field align table<AlignHorizontal, AlignVertical>
---@field render fun(self: UIComponentTextView) renders the text
---@field showContainer boolean

---@class UIComponentTextViewConstructor
---@field visible boolean
---@field text string|table<table<string, UIComponentTextViewStyle>|string>
---@field editable boolean
---@field alignment TextAlignment
---@field style UIComponentTextViewStyle Default text style
---@field multiline boolean|true
---@field backgroundColor Color|nil
---@field widthInLayout number
---@field heightInLayout number
---@field margin { marginX: number, marginY: number }|nil
---@field borderWidth number
---@field borderColor Color
---@field width number
---@field height number
---@field align table<AlignHorizontal, AlignVertical>
---@field showContainer boolean

---@class UIComponentTextViewSection
---@field text string
---@field style UIComponentTextViewStyle

---@class UIComponentTextViewStyle
---@field font UIComponentTextViewFont
---@field locale string|nil
---@field brush Color
---@field lineHeight number
---@field wordSpacing number
---@field letterSpacing number
---@field underline boolean|UIComponentTextViewDecoration|nil
---@field strikethrough boolean|UIComponentTextViewDecoration|nil

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
---@param default boolean If true then font family, size and brush will be initialized with default values if they are nil
---@return TextStyle
local function buildStyle(style, default)
    local textStyle = TextStyle.Create()

    local fontFamily = nil
    local fontSize = nil
    local brush = nil

    if style then
        if style.font then
            fontFamily = style.font.family
            fontSize = style.font.size

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

        brush = style.brush
        if not brush and default then
            brush = Color(1, 1, 1, 1)
        end

        if style.underline then
            if type(style.underline) == "boolean" then
                if style.underline and brush then
                    textStyle:setUnderline(true)
                    textStyle:setUnderlineOffset(2)
                    textStyle:setUnderlineSize(2)
                    textStyle:setUnderlineBrush(brush)
                end
            elseif type(style.underline) == "table" then
                textStyle:setUnderline(true)

                if style.underline.offset then
                    textStyle:setUnderlineOffset(style.underline.offset)
                end
                if style.underline.size then
                    textStyle:setUnderlineSize(style.underline.size)
                end
                if style.underline.brush then
                    textStyle:setUnderlineBrush(style.underline.brush)
                end
            else
                Log.Error("Expected underline parameter either as boolean or table but was " ..
                    tostring(type(style.underline)) .. ": " .. tostring(style.underline))
            end
        end

        if style.strikethrough then
            if type(style.strikethrough) == "boolean" then
                if style.strikethrough and brush then
                    textStyle:setStrikethrough(true)
                    textStyle:setStrikethroughOffset(2)
                    textStyle:setStrikethroughSize(2)
                    textStyle:setStrikethroughBrush(brush)
                end
            elseif type(style.strikethrough) == "table" then
                textStyle:setStrikethrough(true)

                if style.strikethrough.offset then
                    textStyle:setStrikethroughOffset(style.strikethrough.offset)
                end
                if style.strikethrough.size then
                    textStyle:setStrikethroughSize(style.strikethrough.size)
                end
                if style.strikethrough.brush then
                    textStyle:setStrikethroughBrush(style.strikethrough.brush)
                end
            else
                Log.Error("Expected strikethrough parameter either as boolean or table but was " ..
                    tostring(type(style.strikethrough)) .. ": " .. tostring(style.strikethrough))
            end
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

    if fontFamily then
        textStyle:setFontFamily(fontFamily)
    elseif default then
        textStyle:setFontFamily("Exo2Bold")
    end

    if fontSize then
        textStyle:setFontSize(fontSize)
    elseif default then
        textStyle:setFontSize(14)
    end

    if brush then
        textStyle:setBrush(brush)
    end

    return textStyle
end

local function buildTextData(args)
    local text = args.text or "undefined text"

    local sections = {}

    if type(text) == "table" then
        local section_text = ""
        for _, section in ipairs(text) do
            if type(section) == "table" then
                if type(section[1]) == "string" and section[2] then
                    table.insert(sections, {
                        startPos = #section_text,
                        endPos = #section_text + #section[1],
                        style = buildStyle(section[2], false)
                    })

                    section_text = section_text .. section[1]
                else
                    Log.Error(
                        "Expected section description as table with 'string' text and 'table' style fields but was " ..
                        tostring(section))
                end
            elseif type(section) == "string" then
                section_text = section_text .. section
            else
                Log.Error("Expected section description as table or string but was " ..
                    tostring(type(section)) .. ": " .. tostring(section))
            end
        end
        text = section_text
    end

    local multiline = false
    if args.multiline ~= nil then
        multiline = args.multiline
    end

    -- TODO: replace all `\n` in text with spaces if not multiline?

    local textData = TextData.Create(text, buildStyle(args.style, true), args.alignment or TextAlignment.Start, multiline)

    for _, section in ipairs(sections) do
        textData:setSectionStyle(section.startPos, section.endPos, section.style)
    end

    return textData
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
        textData = buildTextData(args),
        editable = args.editable or false,
        borderWidth = args.borderWidth or 0,
        widthInLayout = args.widthInLayout,
        heightInLayout = args.heightInLayout,
        margin = args.margin,
        borderColor = args.borderColor or Color(1, 1, 1, 1),
        backgroundColor = args.backgroundColor,
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

        Gui:textView(self.state.textData(), self.state.editable())
        Gui:setAlignment(self.state.align()[1], self.state.align()[2])

        if self.state.backgroundColor then Gui:setBackgroundColor(self.state.backgroundColor()) end
        if self.state.margin then Gui:setMargin(self.state.margin()[1], self.state.margin()[2]) end

        if self.state.borderWidth() > 0 then
            Gui:setBorderWidth(self.state.borderWidth())
            Gui:setBorderColor(self.state.borderColor())
        elseif self.state.showContainer() then
            Gui:setBorderColor(self.state.showContainerColor())
            Gui:setBorderWidth(1)
        end

        if self.state.width then Gui:setFixedWidth(self.state.width()) end
        if self.state.height then Gui:setFixedHeight(self.state.height()) end

        if Gui:isMouseOver(FocusType.Mouse) and InputInstance:isPressed(Button.MouseLeft) then
            Gui:setFocus()
        end
    end

    return newTextView
end

setmetatable(TextView, meta)

-- Add to global UIComponent table
---@type UIComponentTextViewConstructor
UIComponent.TextView = TextView

return TextView
