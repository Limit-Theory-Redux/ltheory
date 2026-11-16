--- The look of the main menu screens, after BFett's start menu mockups: a fixed
--- sidebar on the left, content in one dark bordered panel or a centred modal.
---
--- Sizes are design pixels on a 2000 x 1125 screen and are scaled to the window
--- (`MenuStyle.context`). The widgets that use this live in
--- `UI/HmGui/Components/MainMenu/MenuWidgets.lua`.
local MenuStyle = {}

MenuStyle.DESIGN_WIDTH = 2000
MenuStyle.DESIGN_HEIGHT = 1125

-- Sidebar: left edge and width in design pixels; it is always as tall as the window.
MenuStyle.SIDEBAR_X = 45
MenuStyle.SIDEBAR_WIDTH = 333

MenuStyle.font = {
    light = "Unageo-Light",
    regular = "Unageo-Regular",
    medium = "Unageo-Medium",
}

MenuStyle.color = {
    sidebar      = Color(0.19, 0.20, 0.33, 0.55),
    rule         = Color(1.00, 1.00, 1.00, 0.16),
    text         = Color(0.89, 0.90, 0.92, 1.00),
    textHover    = Color(1.00, 1.00, 1.00, 1.00),
    label        = Color(0.84, 0.85, 0.87, 1.00),
    dim          = Color(0.56, 0.59, 0.64, 1.00),
    disabled     = Color(0.49, 0.51, 0.55, 1.00),
    heading      = Color(0.18, 0.50, 0.92, 1.00),
    subheading   = Color(0.18, 0.48, 0.87, 1.00),
    panel        = Color(0.12, 0.13, 0.16, 0.96),
    panelBorder  = Color(0.255, 0.275, 0.306, 1.00),
    band         = Color(0.15, 0.165, 0.19, 1.00),
    bandRule     = Color(0.23, 0.25, 0.28, 1.00),
    button       = Color(0.17, 0.18, 0.21, 1.00),
    buttonHover  = Color(0.25, 0.27, 0.31, 1.00),
    buttonBorder = Color(0.27, 0.29, 0.33, 1.00),
    go           = Color(0.18, 0.42, 0.24, 1.00),
    goHover      = Color(0.22, 0.52, 0.30, 1.00),
    goBorder     = Color(0.33, 0.63, 0.40, 1.00),
    field        = Color(0.21, 0.23, 0.255, 1.00),
    fieldHover   = Color(0.25, 0.27, 0.30, 1.00),
    fieldBorder  = Color(0.08, 0.085, 0.10, 1.00),
    arrowBox     = Color(0.10, 0.11, 0.125, 1.00),
    arrow        = Color(0.95, 0.955, 0.97, 1.00),
    listHover    = Color(0.23, 0.29, 0.40, 1.00),
    listSelected = Color(0.17, 0.21, 0.29, 1.00),
    track        = Color(0.17, 0.19, 0.21, 1.00),
    fill         = Color(0.14, 0.41, 0.87, 1.00),
    fillHover    = Color(0.20, 0.50, 0.95, 1.00),
    thumb        = Color(0.84, 0.85, 0.87, 1.00),
    box          = Color(0.16, 0.17, 0.19, 1.00),
    boxHover     = Color(0.21, 0.22, 0.25, 1.00),
    check        = Color(0.20, 0.52, 1.00, 1.00),
    overlay      = Color(0.00, 0.00, 0.00, 0.55),
    cell         = Color(0.15, 0.165, 0.19, 1.00),
    cellRule     = Color(0.23, 0.31, 0.42, 1.00),
}

---@class MenuContext
---@field s number design pixels to window pixels
---@field ox number x of the current parent's design origin, in window pixels
---@field oy number y of the current parent's design origin, in window pixels

---The context of a full screen: the design area is scaled to fit the window and
---centred in it (on a 16:9 window it fills it).
---@return MenuContext
function MenuStyle.context()
    local width, height = Window:width(), Window:height()
    local s = math.min(width / MenuStyle.DESIGN_WIDTH, height / MenuStyle.DESIGN_HEIGHT)
    return {
        s = s,
        ox = math.floor((width - MenuStyle.DESIGN_WIDTH * s) / 2),
        oy = math.floor((height - MenuStyle.DESIGN_HEIGHT * s) / 2),
    }
end

---The context for the children of a widget placed in `ctx`: they are placed
---relative to its top left corner.
---@param ctx MenuContext
---@return MenuContext
function MenuStyle.inside(ctx)
    return { s = ctx.s, ox = 0, oy = 0 }
end

---A font at a design size.
---@param kind "light"|"regular"|"medium"
---@param size number design pixels
---@param s number scale
function MenuStyle.fontOf(kind, size, s)
    return Cache.Font(MenuStyle.font[kind] or MenuStyle.font.light, math.max(6, math.floor(size * s + 0.5)))
end

---Begin a stack container at a design position in its parent (`Gui:setPosition`:
---out of the parent's layout), with a design size if given. Close it with
---`Gui:endContainer()`.
---@param ctx MenuContext
---@param x number
---@param y number
---@param w number|nil
---@param h number|nil
function MenuStyle.beginAt(ctx, x, y, w, h)
    Gui:beginStackContainer()
    Gui:setPosition(ctx.ox + x * ctx.s, ctx.oy + y * ctx.s)
    if w and h then
        Gui:setFixedSize(w * ctx.s, h * ctx.s)
    end
end

---A border width that stays visible on small windows.
---@param width number design pixels
---@param s number scale
---@return number
function MenuStyle.border(width, s)
    return math.max(1, math.floor(width * s + 0.5))
end

---Whether the left mouse button went down this frame: test it right after
---`Gui:isMouseOver` on the widget that should react.
---@return boolean
function MenuStyle.clicked()
    return Input:mouse():isPressed(MouseControl.Left)
end

function MenuStyle.playClick()
    if Config.audio.sounds.click then
        Config.audio.sounds.click:play(1.0)
    end
end

---A text in sentence case ("MAINTAINERS" -> "Maintainers").
---@param text string
---@return string
function MenuStyle.sentenceCase(text)
    return (string.lower(text):gsub("^%l", string.upper))
end

return MenuStyle
