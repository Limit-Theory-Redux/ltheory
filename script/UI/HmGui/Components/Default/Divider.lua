local Divider = {}
Divider.__index = Divider

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class UIComponentDivider: UIComponent
---@field render fun(self: UIComponentDivider) renders the divider

---@class UIComponentDividerConstructor
---@field visible boolean|nil
---@field layoutType GuiLayoutType|nil Horizontal (a horizontal line, the default) or Vertical (a vertical line)
---@field thickness number|nil thickness of the line in pixels (default 1)
---@field length number|nil length of the line in pixels (default: the parent's full width, or height for a vertical line)
---@field color Color|nil colour of the line (default white at 25% opacity)
---@field margin table<number, number>|nil space around the line, `{ x, y }`; a full-length line's margin is taken from its length
---@field align table<AlignHorizontal, AlignVertical>|nil

---returns a divider object: a straight line to separate parts of a UI, such as the groups
---of a table.
---@param args UIComponentDividerConstructor
---@return UIComponentDivider|nil
function Divider:new(args)
    if not args then
        return
    end

    local newDivider = {}
    newDivider.state = UICore.ComponentState {
        visible = args.visible,
        layoutType = args.layoutType or GuiLayoutType.Horizontal,
        thickness = args.thickness or 1,
        length = args.length,
        color = args.color or Color(1, 1, 1, 0.25),
        margin = args.margin or { 0, 0 },
        align = args.align or { AlignHorizontal.Default, AlignVertical.Default }
    }

    newDivider.render = function(self)
        if not self.state.visible() then
            return
        end

        Gui:rect()
        Gui:setBackgroundColor(self.state.color())
        Gui:setAlignment(self.state.align()[1], self.state.align()[2])

        local thickness = self.state.thickness()
        local length = self.state.length and self.state.length()

        if self.state.layoutType() == GuiLayoutType.Vertical then
            Gui:setFixedWidth(thickness)
            if length then Gui:setFixedHeight(length) else Gui:setPercentHeight(100) end
        else
            if length then Gui:setFixedWidth(length) else Gui:setPercentWidth(100) end
            Gui:setFixedHeight(thickness)
        end

        Gui:setMargin(self.state.margin()[1], self.state.margin()[2])
    end

    return newDivider
end

setmetatable(Divider, meta)

-- Add to global UIComponent table
---@type UIComponentDividerConstructor
UIComponent.Divider = Divider

return Divider
