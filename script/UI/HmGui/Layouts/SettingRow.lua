local SettingRow = {}
SettingRow.__index = SettingRow

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class UILayoutSettingRow: UILayoutGrid
---@field labelBlockWidth number|nil set by `UILayout.SettingList`: the width of the list's widest label
---@field measureLabel fun(self: UILayoutSettingRow): number width in pixels the label needs

---@class UILayoutSettingRowConstructor
---@field label string the setting's name, shown on the left
---@field control UIComponent the control for the setting (slider, switch, dropdown, ...)
---@field labelSize number|nil font size of the label (default 24)
---@field labelWidth number|nil share of the row width the label takes (default 0.35)

---A row for one setting: a `UILayout.Grid` with its label on the left and its control
---centred in the rest of the row. Label and control share the row, so they line up
---whatever the font or control size. The row is as tall as its taller part; the label has
---no margin, so the control's margin sets the spacing between rows. Put rows in a
---`UILayout.SettingList`, which lines up their labels.
---@param args UILayoutSettingRowConstructor
---@return UILayoutSettingRow|nil
function SettingRow:new(args)
    if not args then
        return
    end

    local labelFont = "Unageo-Medium" -- the Text component's default font
    local labelSize = args.labelSize or 24
    local labelWidth = args.labelWidth or 0.35

    ---@type UILayoutSettingRow
    local newRow

    -- left-aligned in a block as wide as the list's widest label, so the labels of a
    -- list line up with each other
    local label = UIComponent.Text {
        text = args.label,
        font = labelFont,
        size = labelSize,
        width = function() return newRow.labelBlockWidth or newRow:measureLabel() end,
        align = { AlignHorizontal.Left, AlignVertical.Center }
    }

    newRow = UILayout.Grid {
        align = { AlignHorizontal.Stretch, AlignVertical.Center },
        layoutType = GuiLayoutType.Horizontal,
        cells = { UILayout.Grid.Fraction(labelWidth), UILayout.Grid.Fraction(1 - labelWidth) },
        cellAlign = { AlignHorizontal.Center, AlignVertical.Center },
        contents = { label, args.control }
    }

    newRow.measureLabel = function(self)
        return Cache.Font(labelFont, labelSize):getSize2(args.label).x
    end

    return newRow
end

setmetatable(SettingRow, meta)

-- Add to global UILayout table
---@type UILayoutSettingRowConstructor
UILayout.SettingRow = SettingRow

return SettingRow
