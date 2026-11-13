-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Font
Font = {}

---@param r Renderer
---@param name string
---@param size integer
---@return Font
function Font.Load(r, name, size) end

-- Draw `text` with its baseline-relative top-left corner at `x, y`, alpha
-- blended. Every glyph is a quad of the font's atlas: a string is one
-- draw per atlas page it touches.
---@param r Renderer
---@param text string
---@param x number
---@param y number
---@param color Color
function Font:draw(r, text, x, y, color) end

-- Distance between the baselines of two lines of this font.
---@return integer
function Font:getLineHeight() end

-- The box the glyphs of `text` actually cover (their "ink"), relative to
-- the start of the line on the baseline: `(x, y, width, height)`, y down.
---@param r Renderer
---@param text string
---@param out Vec4i
function Font:getSize(r, text, out) end

-- The size of `text` as one line of layout: `x` is its advance width,
-- `y` the font's line box (ascent + descent). The height is the same for
-- every text of this font, so descenders fit and lines line up; the
-- baseline is `ascent()` below the top of the box.
---@param r Renderer
---@param text string
---@return Vec2i
function Font:getSize2(r, text) end
