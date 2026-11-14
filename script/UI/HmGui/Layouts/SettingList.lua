local SettingList = {}
SettingList.__index = SettingList

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class UILayoutSettingList: UILayoutGrid

---@class UILayoutSettingListConstructor
---@field rows table<UILayoutSettingRow> the settings, top to bottom

---A vertical `UILayout.Grid` of `UILayout.SettingRow`s, as wide as its parent and centred
---vertically in it. Every row gets the same height (the list's height shared evenly), and
---the labels form one left-aligned block, centred in the label column.
---@param args UILayoutSettingListConstructor
---@return UILayoutSettingList|nil
function SettingList:new(args)
    if not args then
        return
    end

    local rows = args.rows or {}

    local newList = UILayout.Grid {
        align = { AlignHorizontal.Stretch, AlignVertical.Center },
        layoutType = GuiLayoutType.Vertical,
        contents = rows
    }

    local renderGrid = newList.render
    local labelBlockWidth

    newList.render = function(self)
        if not labelBlockWidth then
            labelBlockWidth = 0
            for _, row in ipairs(rows) do
                labelBlockWidth = math.max(labelBlockWidth, row:measureLabel())
            end
            for _, row in ipairs(rows) do
                row.labelBlockWidth = labelBlockWidth
            end
        end

        renderGrid(self)
    end

    return newList
end

setmetatable(SettingList, meta)

-- Add to global UILayout table
---@type UILayoutSettingListConstructor
UILayout.SettingList = SettingList

return SettingList
