local MenuStyle = require("UI.HmGui.Styles.MenuStyle")
local Common = require("UI.HmGui.Components.MainMenu.MenuWidgets.Common")
local widget = Common.widget

---Any Gui code, placed at a design position: `fn(ctx)` runs inside a stack
---container of design size w x h.
---@param args { x: number, y: number, w: number, h: number, fn: function }
local function Raw(args)
    return widget(function(self, ctx)
        MenuStyle.beginAt(ctx, args.x, args.y, args.w, args.h)
        args.fn(MenuStyle.inside(ctx))
        Gui:endContainer()
    end)
end

-- Add to the global UIComponent table
UIComponent.MenuRaw = Raw

return Raw
