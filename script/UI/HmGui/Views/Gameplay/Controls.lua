---@type UIView
local Controls = UICore.View {
    name = "Controls"
}

---@type ResponsiveSize
local ResponsiveSize = require("Types.ResponsiveSize")
local ControlsHelp = require("Input.ControlsHelp")

-- NOTE: Escape (back to the pause menu) is handled by LimitTheoryRedux:onInput,
-- like the Paused view; this view only draws the list and the Back button.

-- Built from the bindings each time the view opens.
---@type ControlsHelpGroup[]
Controls.groups = {}

function Controls:onViewOpen(isPageOpen)
    self.groups = ControlsHelp.groups()
    self.screen = nil -- rebuilt on the next render
end

local Grid = UILayout.Grid

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

--- One left-aligned line of text. The vertical padding keeps descenders (g, p, y)
--- from being clipped.
---@return UIComponentText
local function text(value, font, size, color, padY)
    return UIComponent.Text {
        text = value,
        font = font,
        size = size,
        color = color,
        padding = { 0, padY or 3 },
        align = { AlignHorizontal.Left, AlignVertical.Center }
    }
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

--- One column: the column headers, then each group as a title, a rule and a table
--- of action / keyboard & mouse / gamepad rows.
---@param groups ControlsHelpGroup[]
---@param s number scale
---@return UILayoutGrid
local function column(groups, s)
    local columns = { Grid.Fixed(ACTION_WIDTH * s), Grid.Fixed(KEYS_WIDTH * s), Grid.Fixed(GAMEPAD_WIDTH * s) }
    local headerSize = math.floor(14 * s)
    local groupSize = math.floor(22 * s)
    local rowSize = math.floor(17 * s)
    local spacing = math.floor(2 * s)

    local contents = {
        Grid {
            cells = columns,
            contents = {
                text("", "Unageo-Medium", headerSize, DIM),
                text("KEYBOARD & MOUSE", "Unageo-Medium", headerSize, DIM),
                text("GAMEPAD", "Unageo-Medium", headerSize, DIM)
            }
        }
    }

    for _, group in ipairs(groups) do
        local rows = {}
        for _, row in ipairs(group.rows) do
            rows[#rows + 1] = Grid {
                fixedHeight = math.floor(23 * s), -- same height whether or not the text has descenders
                cells = columns,
                contents = {
                    text(row.action, "Unageo-Regular", rowSize, GREY),
                    text(row.keys, "Unageo-Regular", rowSize, WHITE),
                    text(row.gamepad, "Unageo-Regular", rowSize, WHITE)
                }
            }
        end
        contents[#contents + 1] = Grid {
            layoutType = GuiLayoutType.Vertical,
            padding = { 0, math.floor(12 * s) + spacing, 0, 0 },
            spacing = spacing,
            cells = Grid.Fit(),
            contents = {
                text(group.title, "Unageo-Semibold", groupSize, ACCENT, 4),
                UIComponent.Divider { color = RULE },
                Grid {
                    layoutType = GuiLayoutType.Vertical,
                    padding = { 0, math.floor(4 * s) + spacing, 0, 0 },
                    spacing = spacing,
                    cells = Grid.Fit(),
                    contents = rows
                }
            }
        }
    end

    return Grid {
        layoutType = GuiLayoutType.Vertical,
        spacing = spacing,
        cells = Grid.Fit(),
        contents = contents
    }
end

local backButton = UIComponent.Button_MainMenu {
    title = "Back",
    size = ResponsiveSize(200, 40, true),
    font = { name = "Unageo-Medium", size = 20 },
    align = { AlignHorizontal.Center, AlignVertical.Center },
    callback = function() LimitTheoryRedux:closeControls() end
}

--- The whole screen: title, the groups in two columns, Back button.
---@param groups ControlsHelpGroup[]
---@param s number scale
---@return UILayoutGrid
local function screen(groups, s)
    local left, right = splitColumns(groups)

    return Grid {
        layoutType = GuiLayoutType.Vertical,
        align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
        childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
        cellAlign = { AlignHorizontal.Center, AlignVertical.Center },
        color = { background = Color(0, 0, 0, 0.75) },
        padding = { 0, math.floor(40 * s) },
        spacing = math.floor(28 * s),
        cells = Grid.Fit(),
        contents = {
            text("Controls", "Unageo-Semibold", math.floor(40 * s), WHITE),
            Grid {
                spacing = COLUMN_GAP * s,
                cells = Grid.Fit(),
                contents = { column(left, s), column(right, s) }
            },
            backButton
        }
    }
end

-- Built on the first render after the view opens, and again when the window size
-- changes the scale.
Controls:addContent({
    state = {},
    render = function()
        local s = scale()
        if not Controls.screen or Controls.screenScale ~= s then
            Controls.screen = screen(Controls.groups, s)
            Controls.screenScale = s
        end
        Controls.screen:render()
    end
})

return Controls
