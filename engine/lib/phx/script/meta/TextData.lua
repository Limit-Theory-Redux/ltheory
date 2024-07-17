---@meta

---@class TextData
TextData = {}

---@param text string
---@param defaultStyle TextStyle
---@param cursorColor Color
---@param selectionColor Color
---@param alignment TextAlignment
---@param multiline boolean
---@return TextData
function TextData.Create(text, defaultStyle, cursorColor, selectionColor, alignment, multiline) end

---@return string
function TextData:text() end

-- Set style of the text section beginning at 'start_pos' position and up to 'end_pos'.
---@param startPos integer
---@param endPos integer
---@param style TextStyle
function TextData:setSectionStyle(startPos, endPos, style) end

-- Sets cursor position in a text before character at position `pos`.
-- If pos >= text size then cursor is placed after the latest text character.
---@param pos integer
function TextData:setCursorPos(pos) end

---@param color Color
function TextData:setSelectionColor(color) end

---@param startPos integer
---@param endPos integer
function TextData:setSelection(startPos, endPos) end

