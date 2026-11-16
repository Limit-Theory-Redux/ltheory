local MenuStyle = require("UI.HmGui.Styles.MenuStyle")
local Common = require("UI.HmGui.Components.MainMenu.MenuWidgets.Common")
local C = MenuStyle.color
local renderChildren, widget = Common.renderChildren, Common.widget

---A dark bordered panel, or a modal when it has a `band` (a header strip of
---that height). Its contents are placed relative to its top left corner.
---@param args { x: number, y: number, w: number, h: number, band: number|nil, contents: table|function }
local function Panel(args)
    return widget(function(self, ctx)
        local s = ctx.s
        MenuStyle.beginAt(ctx, args.x, args.y, args.w, args.h)
        Gui:setBackgroundColor(C.panel)
        Gui:setBorderWidth(MenuStyle.border(2, s))
        Gui:setBorderColor(C.panelBorder)
        if args.band then
            Gui:rect()
            Gui:setPosition(0, 0)
            Gui:setFixedSize(args.w * s, args.band * s)
            Gui:setBackgroundColor(C.band)
            Gui:rect()
            Gui:setPosition(0, args.band * s)
            Gui:setFixedSize(args.w * s, MenuStyle.border(1, s))
            Gui:setBackgroundColor(C.bandRule)
        end
        renderChildren(args.contents, MenuStyle.inside(ctx))
        Gui:endContainer()
    end)
end

-- Add to the global UIComponent table
UIComponent.MenuPanel = Panel

return Panel
