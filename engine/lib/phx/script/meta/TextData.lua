---@meta

---@class TextData
TextData = {}

---@param text string
---@param defaultStyle TextStyle
---@param alignment TextAlignment
---@return TextData
function TextData.Create(text, defaultStyle, alignment) end

-- Set style of the text section beginning at 'start_pos' position and up to 'end_pos'.
---@param startPos integer
---@param endPos integer
---@param style TextStyle
function TextData:setSectionStyle(startPos, endPos, style) end

