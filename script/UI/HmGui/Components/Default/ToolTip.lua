local ToolTip = {}
ToolTip.__index = ToolTip

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class UIComponentToolTip: UIComponent
---@field visible boolean
---@field text string
---@field font string
---@field size number
---@field color Color
---@field padding { paddingX: number, paddingY: number }
---@field backgroundColor Color
---@field borderWidth number
---@field borderColor Color
---@field render fun(self: UIComponentToolTip) renders the text

---@class UIComponentToolTipConstructor
---@field visible boolean
---@field text string
---@field font string
---@field size number
---@field color Color
---@field padding { paddingX: number, paddingY: number }|nil
---@field backgroundColor Color
---@field borderWidth number
---@field borderColor Color
---@field align table<AlignHorizontal, AlignVertical>

-- Shows a tool tip message in a separate layout if mouse is hovering over previous element.
-- This component cannot be used on its own but as a part of another one. See Button for example.
-- Add ToolTip as a parameter to the existing component, initialize it (at least text field)
-- and then call its render function after element for wich you want to show a tool tip,
-- in case of Button that its outer container.
---@param args UIComponentToolTipConstructor
---@return UIComponentToolTip|nil
function ToolTip:new(args)
    if not args then
        return
    end

    local newToolTip = {}
    newToolTip.state = UICore.ComponentState {
        visible = args.visible,
        text = args.text or nil,
        font = args.font or "Exo2Bold",
        size = args.size or 14,
        color = args.color or Color(1, 1, 1, 1),
        padding = args.padding or { 10, 10 },
        backgroundColor = args.backgroundColor or Color(0.7, 0.7, 0.7, 0.8),
        borderWidth = args.borderWidth or 1,
        borderColor = args.borderColor or Color(1, 1, 1, 1),
        align = args.align or { AlignHorizontal.Center, AlignVertical.Center }
    }

    newToolTip.render = function(self)
        if not self.state.visible() or not self.state.text or not Gui:isMouseOver(FocusType.Mouse) then
            return
        end

        -- shift tool tip position to not overlap mouse cursor
        -- TODO: use actual cursor size
        local pos = InputInstance:mouse():position() + Vec2f(15, 20)
        Gui:beginLayerAtPos(pos)

        Gui:beginStackContainer()
        Gui:setAlignment(AlignHorizontal.Left, AlignVertical.Top)
        Gui:setPadding(self.state.padding()[1], self.state.padding()[2])
        Gui:setBackgroundColor(self.state.backgroundColor())
        Gui:setBorderWidth(self.state.borderWidth())
        Gui:setBorderColor(self.state.borderColor())

        Gui:text(tostring(self.state.text()), Cache.Font(self.state.font(), self.state.size()), self.state.color())
        Gui:setAlignment(self.state.align()[1], self.state.align()[2])

        Gui:endContainer()

        Gui:endLayer()
    end

    return newToolTip
end

setmetatable(ToolTip, meta)

-- Add to global UIComponent table
---@type UIComponentToolTipConstructor
UIComponent.ToolTip = ToolTip

return ToolTip
