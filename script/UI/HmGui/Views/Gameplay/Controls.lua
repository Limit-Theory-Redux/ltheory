---@type UIView
local Controls = UICore.View {
    name = "Controls"
}

---@type ResponsiveSize
local ResponsiveSize = require("Types.ResponsiveSize")

-- NOTE: Escape (back to the pause menu) is handled by LimitTheoryRedux:onInput,
-- like the Paused view; this view only draws the list and the Back button.

-- Built from the bindings each time the view opens. Required here rather than
-- at load: views load at startup, before the game state and its bindings.
---@type ControlsHelpGroup[]
Controls.groups = {}

function Controls:onViewOpen(isPageOpen)
    self.groups = require("Input.ControlsHelp").groups()
end

local WHITE = Color(1.0, 1.0, 1.0, 1.0)
local GREY = Color(0.75, 0.77, 0.80, 1.0)
local DIM = Color(0.51, 0.54, 0.59, 1.0)
local ACCENT = Color(0.10, 0.60, 1.00, 1.0)        -- HUD blue (meterBarLight without its alpha)
local RULE = Color(0.10, 0.60, 1.00, 0.35)

-- Column widths at 1920 px window width; scaled down on smaller windows.
local ACTION_WIDTH, KEYS_WIDTH, GAMEPAD_WIDTH, COLUMN_GAP = 320, 260, 190, 80

---@return number
local function scale()
    return math.max(0.7, math.min(1.0, Window:width() / 1920))
end

--- One left-aligned text cell, optionally of a fixed width. The vertical padding
--- keeps descenders (g, p, y) from being clipped.
local function cell(text, font, color, width, padY)
    Gui:beginStackContainer()
    if width then Gui:setFixedWidth(width) end
    Gui:setPadding(0, padY or 3)
    Gui:text(text, font, color)
    Gui:setAlignment(AlignHorizontal.Left, AlignVertical.Center)
    Gui:endContainer()
end

--- Empty vertical space.
local function gap(height)
    Gui:rect()
    Gui:setFixedSize(1, height)
end

--- Split the groups over two columns of about equal height (in rows).
---@param groups ControlsHelpGroup[]
---@return ControlsHelpGroup[] left, ControlsHelpGroup[] right
local function splitColumns(groups)
    local total = 0
    for _, group in ipairs(groups) do total = total + #group.rows + 2 end -- + header and rule
    local left, right, used = {}, {}, 0
    for _, group in ipairs(groups) do
        if used < total / 2 then
            left[#left + 1] = group
            used = used + #group.rows + 2
        else
            right[#right + 1] = group
        end
    end
    return left, right
end

local function column(groups, s)
    local actionWidth, keysWidth, gamepadWidth = ACTION_WIDTH * s, KEYS_WIDTH * s, GAMEPAD_WIDTH * s
    local headerFont = Cache.Font("Unageo-Medium", math.floor(14 * s))
    local groupFont = Cache.Font("Unageo-Semibold", math.floor(22 * s))
    local rowFont = Cache.Font("Unageo-Regular", math.floor(17 * s))

    Gui:beginVerticalContainer()
    Gui:setSpacing(math.floor(2 * s))

    Gui:beginHorizontalContainer()
    cell("", headerFont, DIM, actionWidth)
    cell("KEYBOARD & MOUSE", headerFont, DIM, keysWidth)
    cell("GAMEPAD", headerFont, DIM, gamepadWidth)
    Gui:endContainer()

    for _, group in ipairs(groups) do
        gap(math.floor(12 * s))
        cell(group.title, groupFont, ACCENT, nil, 4)
        Gui:rect()
        Gui:setFixedSize(actionWidth + keysWidth + gamepadWidth, 1)
        Gui:setBackgroundColor(RULE)
        gap(math.floor(4 * s))
        for _, row in ipairs(group.rows) do
            Gui:beginHorizontalContainer()
            Gui:setFixedHeight(math.floor(23 * s)) -- same height whether or not the text has descenders
            Gui:setChildrenAlignment(AlignHorizontal.Left, AlignVertical.Center)
            cell(row.action, rowFont, GREY, actionWidth)
            cell(row.keys, rowFont, WHITE, keysWidth)
            cell(row.gamepad, rowFont, WHITE, gamepadWidth)
            Gui:endContainer()
        end
    end

    Gui:endContainer()
end

local backButton = UIComponent.Button_MainMenu {
    title = "Back",
    size = ResponsiveSize(200, 40, true),
    font = { name = "Unageo-Medium", size = 20 },
    align = { AlignHorizontal.Center, AlignVertical.Center },
    callback = function() LimitTheoryRedux:closeControls() end
}

--- The whole screen: title, the groups in two columns, Back button. Drawn with
--- plain Gui calls: UIComponent.Container always fills its parent, which doesn't
--- suit fixed-width columns.
local controlsList = {
    state = {},
    render = function(self)
        local s = scale()
        local left, right = splitColumns(Controls.groups)

        Gui:beginVerticalContainer()
        Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Stretch)
        Gui:setChildrenAlignment(AlignHorizontal.Center, AlignVertical.Center)
        Gui:setBackgroundColor(Color(0, 0, 0, 0.75))
        Gui:setPadding(0, math.floor(40 * s))
        Gui:setSpacing(math.floor(28 * s))

        cell("Controls", Cache.Font("Unageo-Semibold", math.floor(40 * s)), WHITE)

        Gui:beginHorizontalContainer()
        Gui:setSpacing(COLUMN_GAP * s)
        column(left, s)
        column(right, s)
        Gui:endContainer()

        backButton:render()

        Gui:endContainer()
    end
}

Controls:addContent(controlsList)

return Controls
