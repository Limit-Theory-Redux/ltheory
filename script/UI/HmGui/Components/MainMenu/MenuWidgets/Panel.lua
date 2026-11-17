local MenuStyle = require("UI.HmGui.Styles.MenuStyle")
local Common = require("UI.HmGui.Components.MainMenu.MenuWidgets.Common")
local C = MenuStyle.color
local renderChildren, widget = Common.renderChildren, Common.widget

---A dark bordered panel, or a modal when it has a `band` (a header strip of
---that height). Its contents are placed relative to its top left corner.
---@param args { x: number, y: number, w: number, h: number, band: number|nil, contents: table|function }
local function Panel(args)
    return widget(function(self, ctx)
        MenuStyle.beginAt(ctx, args.x, args.y, args.w, args.h)
        Gui:setBackgroundColor(C.panel)
        Gui:setBorderWidth(2)
        Gui:setBorderColor(C.panelBorder)
        if args.band then
            Gui:rect()
            Gui:setPosition(0, 0)
            Gui:setFixedSize(args.w, args.band)
            Gui:setBackgroundColor(C.band)
            Gui:rect()
            Gui:setPosition(0, args.band)
            Gui:setFixedSize(args.w, MenuStyle.line(1, ctx.s))
            Gui:setBackgroundColor(C.bandRule)
        end
        renderChildren(args.contents, MenuStyle.inside(ctx))
        Gui:endContainer()
    end)
end

-- Add to the global UIComponent table
UIComponent.MenuPanel = Panel

return Panel
