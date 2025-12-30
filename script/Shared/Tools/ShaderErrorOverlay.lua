--[[
    ShaderErrorOverlay - Error display for shader compilation failures

    Displays shader compilation errors in a top banner.
    Automatically shows when errors occur and can be dismissed.

    Usage:
        local ShaderErrorOverlay = require('Shared.Tools.ShaderErrorOverlay')

        -- In your render loop (after all other rendering):
        ShaderErrorOverlay:draw()

        -- In your input handling:
        ShaderErrorOverlay:handleInput()
]]

local DrawEx = require('UI.DrawEx')

---@class ShaderErrorOverlay
local ShaderErrorOverlay = {}

-- Configuration
ShaderErrorOverlay.font = 'Unageo-Medium'
ShaderErrorOverlay.padding = 12
ShaderErrorOverlay.margin = 10
ShaderErrorOverlay.lineHeight = 18

-- State
local visible = false
local autoShow = true

-- Colors
local colors = {
    bg = Color(0.4, 0.08, 0.08, 1.0),
    border = Color(1.0, 0.3, 0.3, 1.0),
    header = Color(1.0, 1.0, 1.0, 1.0),
    label = Color(0.9, 0.9, 0.9, 1.0),
    error = Color(1.0, 1.0, 0.7, 1.0),
    shader = Color(0.7, 0.9, 1.0, 1.0),
    dismiss = Color(0.7, 0.7, 0.7, 1.0),
}

function ShaderErrorOverlay:show()
    visible = true
end

function ShaderErrorOverlay:hide()
    visible = false
    if ShaderError then
        ShaderError.AcknowledgeErrors()
    end
end

function ShaderErrorOverlay:toggle()
    if visible then
        self:hide()
    else
        self:show()
    end
end

function ShaderErrorOverlay:isVisible()
    return visible
end

function ShaderErrorOverlay:setAutoShow(enabled)
    autoShow = enabled
end

function ShaderErrorOverlay:draw()
    -- Check for new errors if auto-show is enabled
    if autoShow and ShaderError and ShaderError.HasNewErrors() then
        self:show()
    end

    if not visible then return end

    local errorCount = ShaderError and ShaderError.GetCount() or 0
    if errorCount == 0 then
        self:hide()
        return
    end

    -- Set up render state
    Window:beginDraw()
    RenderState.PushAllDefaults()
    ClipRect.PushDisabled()

    local win = Window:size()
    local resX, resY = win.x, win.y

    -- Get latest error info (safely handle null pointers)
    local function safeString(ptr, default)
        if ptr == nil then return default end
        local ok, result = pcall(ffi.string, ptr)
        return ok and result or default
    end

    local shaderKey = safeString(ShaderError.GetShaderKey(errorCount - 1), "unknown")
    local errorType = safeString(ShaderError.GetErrorType(errorCount - 1), "error")
    local message = safeString(ShaderError.GetMessage(errorCount - 1), "Unknown error")

    -- Truncate message to first line for banner display
    local firstLine = message:match("^[^\n]+") or message
    if #firstLine > 100 then
        firstLine = firstLine:sub(1, 97) .. "..."
    end

    -- Calculate banner dimensions
    local bannerHeight = self.lineHeight * 3 + self.padding * 2
    local bannerWidth = resX - self.margin * 2
    local bannerX = self.margin
    local bannerY = self.margin

    -- Draw solid background using Panel (uses Alpha blend)
    DrawEx.Panel(bannerX, bannerY, bannerWidth, bannerHeight, colors.bg, 1.0)

    -- Draw border
    local borderWidth = 3
    RenderState.PushBlendMode(BlendMode.Alpha)
    Draw.Color(colors.border.r, colors.border.g, colors.border.b, colors.border.a)
    Draw.Rect(bannerX, bannerY, bannerWidth, borderWidth)  -- Top
    Draw.Rect(bannerX, bannerY + bannerHeight - borderWidth, bannerWidth, borderWidth)  -- Bottom
    Draw.Rect(bannerX, bannerY, borderWidth, bannerHeight)  -- Left
    Draw.Rect(bannerX + bannerWidth - borderWidth, bannerY, borderWidth, bannerHeight)  -- Right
    RenderState.PopBlendMode()

    -- Draw text content
    local textX = bannerX + self.padding + borderWidth
    local textY = bannerY + self.padding + borderWidth

    -- Header line
    local headerText = string.format("SHADER ERROR: %s (%s)", shaderKey, errorType)
    if errorCount > 1 then
        headerText = headerText .. string.format(" [+%d more]", errorCount - 1)
    end

    DrawEx.TextAlpha(
        self.font, headerText, 14,
        textX, textY, bannerWidth, self.lineHeight,
        colors.header.r, colors.header.g, colors.header.b, colors.header.a,
        0, 0.5
    )
    textY = textY + self.lineHeight

    -- Error message
    DrawEx.TextAlpha(
        self.font, firstLine, 12,
        textX, textY, bannerWidth - self.padding * 2, self.lineHeight,
        colors.error.r, colors.error.g, colors.error.b, colors.error.a,
        0, 0.5
    )
    textY = textY + self.lineHeight

    -- Dismiss hint
    DrawEx.TextAlpha(
        self.font, "Press ESC to dismiss | Fix shader to auto-clear", 11,
        textX, textY, bannerWidth - self.padding * 2, self.lineHeight,
        colors.dismiss.r, colors.dismiss.g, colors.dismiss.b, colors.dismiss.a,
        0, 0.5
    )

    -- Clean up
    ClipRect.Pop()
    RenderState.PopAll()
    Window:endDraw()
end

function ShaderErrorOverlay:handleInput()
    if not visible then return false end

    -- ESC to dismiss
    if Input:keyboard():isPressed(Button.KeyboardEscape) then
        self:hide()
        return true
    end

    -- Click anywhere to dismiss
    if Input:mouse():isPressed(Button.MouseLeft) then
        self:hide()
        return true
    end

    return false
end

return ShaderErrorOverlay
