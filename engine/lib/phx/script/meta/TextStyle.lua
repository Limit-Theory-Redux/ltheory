---@meta

-- Contains collection of different text styling properties.
---@class TextStyle
TextStyle = {}

---@return TextStyle
function TextStyle.Create() end

-- Font family list in CSS format.
---@param family string
function TextStyle:setFontFamily(family) end

---@param size number
function TextStyle:setFontSize(size) end

-- Visual width of a font-- a relative change from the normal aspect
-- ratio, typically in the range 0.5 to 2.0.
---@param stretch number
function TextStyle:setFontStretch(stretch) end

-- Specify whether font italic or normal.
---@param italic boolean
function TextStyle:setFontItalic(italic) end

-- Visual weight class of a font, typically on a scale from 1.0 to 1000.0.
---@param weight number
function TextStyle:setFontWeight(weight) end

---@param locale string?
function TextStyle:setLocale(locale) end

-- Brush for rendering text.
---@param color Color
function TextStyle:setBrush(color) end

-- Underline decoration.
---@param underline boolean
function TextStyle:setUnderline(underline) end

-- Offset of the underline decoration.
---@param offset number
function TextStyle:setUnderlineOffset(offset) end

-- Size of the underline decoration.
---@param size number
function TextStyle:setUnderlineSize(size) end

-- Brush for rendering the underline decoration.
---@param color Color?
function TextStyle:setUnderlineBrush(color) end

-- Strikethrough decoration.
---@param strikethrough boolean
function TextStyle:setStrikethrough(strikethrough) end

-- Offset of the strikethrough decoration.
---@param offset number
function TextStyle:setStrikethroughOffset(offset) end

-- Size of the strikethrough decoration.
---@param size number
function TextStyle:setStrikethroughSize(size) end

-- Brush for rendering the strikethrough decoration.
---@param color Color?
function TextStyle:setStrikethroughBrush(color) end

-- Line height multiplier.
---@param height number
function TextStyle:setLineHeight(height) end

-- Extra spacing between words.
---@param size number
function TextStyle:setWordSpacing(size) end

-- Extra spacing between letters.
---@param size number
function TextStyle:setLetterSpacing(size) end

