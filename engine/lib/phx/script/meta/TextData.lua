-- AUTO GENERATED. DO NOT MODIFY!
---@meta

-- Text information used in the [`TextView`] component.
-- Use `Gui:textView(textData, editable)` to add text view element to the gui hierarchy.
-- To retrieve changes of the editable text made by user, use `Gui:getTextViewChanges(textData)`
-- and `textData:text()` functions.
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

---@param text string
function TextData:setText(text) end

---@param scaleFactor number
function TextData:setScaleFactor(scaleFactor) end

---@return boolean
function TextData:isMultiline() end

---@param multiline boolean
function TextData:setMultiline(multiline) end

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

