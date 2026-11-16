local MenuStyle = require("UI.HmGui.Styles.MenuStyle")
local Common = require("UI.HmGui.Components.MainMenu.MenuWidgets.Common")
local C = MenuStyle.color
local value, makeToolTip, widget = Common.value, Common.makeToolTip, Common.widget

---A boxed button. `style` "go" is the green one (Start).
---@param args { x: number, y: number, w: number, h: number, title: string|function, size: number|nil, style: string|nil, uppercase: boolean|nil, enabled: function|nil, toolTip: string|function|nil, callback: function }
local function Button(args)
    local toolTip = makeToolTip(args.toolTip)
    return widget(function(self, ctx)
        local s = ctx.s
        local enabled = not args.enabled or args.enabled()
        MenuStyle.beginAt(ctx, args.x, args.y, args.w, args.h)
        local hover = enabled and Gui:isMouseOver(FocusType.Mouse)
        local go = args.style == "go"
        if go then
            Gui:setBackgroundColor(hover and C.goHover or C.go)
            Gui:setBorderColor(C.goBorder)
        else
            Gui:setBackgroundColor(hover and C.buttonHover or C.button)
            Gui:setBorderColor(C.buttonBorder)
        end
        Gui:setBorderWidth(MenuStyle.border(1, s))
        local title = value(args.title)
        if args.uppercase ~= false then
            title = string.upper(title)
        end
        Gui:text(title, MenuStyle.fontOf("light", args.size or 23, s), enabled and C.text or C.disabled)
        Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)
        Gui:endContainer()
        if toolTip then toolTip:render() end
        if hover and MenuStyle.clicked() then
            MenuStyle.playClick()
            args.callback()
        end
    end)
end

-- Add to the global UIComponent table
UIComponent.MenuButton = Button

return Button
