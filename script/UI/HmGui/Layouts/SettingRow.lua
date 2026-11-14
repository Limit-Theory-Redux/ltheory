local SettingRow = {}
SettingRow.__index = SettingRow

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class UILayoutSettingRow
---@field render fun(self: UILayoutSettingRow) renders the row

---@class UILayoutSettingRowConstructor
---@field label string the setting's name, shown on the left
---@field control UIComponent the control for the setting (slider, switch, dropdown, ...)
---@field labelSize number|nil font size of the label (default 24)
---@field labelWidth number|nil share of the row width the label takes (default 0.35)

---A row for one setting: its label on the left and its control centred in the rest
---of the row. Label and control share the row, so they line up whatever the font or
---control size. The row is as tall as its taller part; the label has no margin, so the
---control's margin sets the spacing between rows. Put rows in a `UILayout.SettingList`.
---
---Written with plain `Gui:` calls: a `UIComponent.Container` always takes 100% of its
---parent's width and height, which would make every row as tall as the whole list.
---@param args UILayoutSettingRowConstructor
---@return UILayoutSettingRow|nil
function SettingRow:new(args)
    if not args then
        return
    end

    local newRow = {
        labelText = args.label,
        labelFont = "Unageo-Medium", -- the Text component's default font
        labelSize = args.labelSize or 24,
        labelWidth = args.labelWidth or 0.35,
        control = args.control
    }
    newRow.label = UIComponent.Text {
        text = newRow.labelText,
        font = newRow.labelFont,
        size = newRow.labelSize,
        align = { AlignHorizontal.Left, AlignVertical.Center }
    }

    ---Width in pixels the label needs.
    newRow.measureLabel = function(self)
        return Cache.Font(self.labelFont, self.labelSize):getSize2(self.labelText).x
    end

    ---@param labelBlockWidth number|nil when given, the label sits left-aligned in a block
    ---this wide that is centred in the label's share of the row, so the labels of a list
    ---line up with each other
    newRow.render = function(self, labelBlockWidth)
        Gui:beginContainer(GuiLayoutType.Horizontal)
        Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Center)

        -- label
        Gui:beginStackContainer()
        Gui:setAlignment(AlignHorizontal.Default, AlignVertical.Center)
        if labelBlockWidth then
            Gui:beginStackContainer()
            Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)
            self.label:render()
            Gui:endContainer()
            Gui:setFixedWidth(labelBlockWidth)
        else
            self.label:render()
        end
        Gui:endContainer()
        Gui:setPercentWidth(self.labelWidth * 100)

        -- control, in a container as wide as the control and centred in the rest of the
        -- row (components don't all honour their own `align`)
        Gui:beginStackContainer()
        Gui:setAlignment(AlignHorizontal.Default, AlignVertical.Center)
        Gui:beginContainer(GuiLayoutType.Horizontal)
        Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)
        self.control:render()
        Gui:endContainer()
        Gui:endContainer()
        Gui:setPercentWidth((1 - self.labelWidth) * 100)

        Gui:endContainer()
    end

    return newRow
end

setmetatable(SettingRow, meta)

-- Add to global UILayout table
---@type UILayoutSettingRowConstructor
UILayout.SettingRow = SettingRow

return SettingRow
