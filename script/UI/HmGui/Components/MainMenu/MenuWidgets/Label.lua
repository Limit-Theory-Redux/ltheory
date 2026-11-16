local MenuStyle = require("UI.HmGui.Styles.MenuStyle")
local Common = require("UI.HmGui.Components.MainMenu.MenuWidgets.Common")
local C = MenuStyle.color
local value, widget = Common.value, Common.widget

---A line of text, centred vertically in `h` (default: a line of its size), so
---labels line up with a control of the same y and h. `align` "left" puts its
---left edge at x (and cuts it at `w`, if given), "right" its right edge at x
---(within `w` to the left of x, default: from the parent's left edge), and
---"center" centres it in `w` from x. A text longer than its box ends with "…".
---@param args { x: number, y: number, text: string|function, size: number|nil, font: string|nil, color: Color|function|nil, align: string|nil, w: number|nil, h: number|nil, visible: function|nil }
local function Label(args)
    local align = args.align or "left"
    local hAlign = ({ left = AlignHorizontal.Left, right = AlignHorizontal.Right, center = AlignHorizontal.Center })[align]
    return widget(function(self, ctx)
        if args.visible and not args.visible() then
            return
        end
        local s = ctx.s
        local size = args.size or 27
        local h = args.h or size * 1.5
        local w = args.w
        if align == "right" then
            w = w or args.x
            MenuStyle.beginAt(ctx, args.x - w, args.y, w, h)
        elseif w then
            MenuStyle.beginAt(ctx, args.x, args.y, w, h)
        else
            MenuStyle.beginAt(ctx, args.x, args.y)
            Gui:setFixedHeight(h * s)
        end
        Gui:text(tostring(value(args.text)), MenuStyle.fontOf(args.font or "light", size, s), value(args.color) or C.label)
        Gui:setAlignment(hAlign, AlignVertical.Center)
        if w then
            Gui:setMaxWidth(w * s)
        end
        Gui:endContainer()
    end)
end

-- Add to the global UIComponent table
UIComponent.MenuLabel = Label

return Label
