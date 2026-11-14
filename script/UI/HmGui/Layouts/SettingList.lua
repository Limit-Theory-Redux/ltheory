local SettingList = {}
SettingList.__index = SettingList

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class UILayoutSettingList
---@field render fun(self: UILayoutSettingList) renders the list

---@class UILayoutSettingListConstructor
---@field rows table<UILayoutSettingRow> the settings, top to bottom

---A vertical list of `UILayout.SettingRow`s, as wide as its parent and centred vertically
---in it. Every row gets the same height (the list's height shared evenly), and the labels
---form one left-aligned block, centred in the label column.
---@param args UILayoutSettingListConstructor
---@return UILayoutSettingList|nil
function SettingList:new(args)
    if not args then
        return
    end

    local newList = { rows = args.rows or {} }

    newList.render = function(self)
        if not self.labelBlockWidth then
            self.labelBlockWidth = 0
            for _, row in ipairs(self.rows) do
                self.labelBlockWidth = math.max(self.labelBlockWidth, row:measureLabel())
            end
        end

        Gui:beginContainer(GuiLayoutType.Vertical)
        Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Center)

        for _, row in ipairs(self.rows) do
            row:render(self.labelBlockWidth)
            Gui:setPercentHeight(100 / #self.rows)
        end

        Gui:endContainer()
    end

    return newList
end

setmetatable(SettingList, meta)

-- Add to global UILayout table
---@type UILayoutSettingListConstructor
UILayout.SettingList = SettingList

return SettingList
