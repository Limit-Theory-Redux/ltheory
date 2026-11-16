local ToolTip = {}
ToolTip.__index = ToolTip

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

-- Where the tool tip opens: this far right of and below the cursor.
local CURSOR_OFFSET_X, CURSOR_OFFSET_Y = 15, 20
-- Gap between the cursor and a tool tip moved to its left or above it.
local FLIPPED_GAP = 4

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
-- The text may have several lines separated by "\n". The tool tip stays inside the window.
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

        local font = Cache.Font(self.state.font(), self.state.size())
        local padding = self.state.padding()
        local borderWidth = self.state.borderWidth()
        local align = self.state.align()

        -- A text widget draws a single line, so every "\n" starts a text widget
        -- of its own. Each is measured like the text widget measures itself.
        local lines = {}
        local textWidth, textHeight = 0, 0
        for line in (tostring(self.state.text()) .. "\n"):gmatch("(.-)\n") do
            local size = font:getSize2(line)
            lines[#lines + 1] = line
            textWidth = math.max(textWidth, size.x)
            textHeight = textHeight + size.y
        end

        local boxWidth = textWidth + 2 * (padding[1] + borderWidth)
        local boxHeight = textHeight + 2 * (padding[2] + borderWidth)

        -- Below and right of the cursor so it is not covered (TODO: use the
        -- actual cursor size); on the other side of the cursor where the box
        -- would leave the window, and never past its left/top edge.
        local mouse = Input:mouse():position()
        local window = Window:size()
        local x = mouse.x + CURSOR_OFFSET_X
        local y = mouse.y + CURSOR_OFFSET_Y
        if x + boxWidth > window.x then
            x = mouse.x - FLIPPED_GAP - boxWidth
        end
        if y + boxHeight > window.y then
            y = mouse.y - FLIPPED_GAP - boxHeight
        end
        Gui:beginLayerAtPos(Vec2f(math.max(0, x), math.max(0, y)))

        Gui:beginStackContainer()
        Gui:setAlignment(AlignHorizontal.Left, AlignVertical.Top)
        Gui:setPadding(padding[1], padding[2])
        Gui:setBackgroundColor(self.state.backgroundColor())
        Gui:setBorderWidth(borderWidth)
        Gui:setBorderColor(self.state.borderColor())

        Gui:beginVerticalContainer()
        for _, line in ipairs(lines) do
            Gui:text(line, font, self.state.color())
            Gui:setAlignment(align[1], align[2])
        end
        Gui:endContainer()

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
