local MenuStyle = require("UI.HmGui.Styles.MenuStyle")
local Common = require("UI.HmGui.Components.MainMenu.MenuWidgets.Common")
local C = MenuStyle.color
local makeToolTip, widget = Common.makeToolTip, Common.widget

---A checkbox: a dark square with a blue square inside when on.
---@param args { x: number, y: number, get: function, set: function, toolTip: string|function|nil }
local function Checkbox(args)
    local toolTip = makeToolTip(args.toolTip)
    return widget(function(self, ctx)
        local s = ctx.s
        MenuStyle.beginAt(ctx, args.x, args.y, 40, 40)
        local hover = Gui:isMouseOver(FocusType.Mouse)
        Gui:setBackgroundColor(hover and C.boxHover or C.box)
        Gui:setBorderWidth(MenuStyle.border(2, s))
        Gui:setBorderColor(C.fieldBorder)
        if args.get() then
            Gui:rect()
            Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)
            Gui:setFixedSize(22 * s, 22 * s)
            Gui:setBackgroundColor(C.check)
        end
        Gui:endContainer()
        if toolTip then toolTip:render() end
        if hover and MenuStyle.clicked() then
            MenuStyle.playClick()
            args.set(not args.get())
        end
    end)
end

-- Add to the global UIComponent table
UIComponent.MenuCheckbox = Checkbox

return Checkbox
