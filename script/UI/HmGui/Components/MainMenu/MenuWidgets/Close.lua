local MenuStyle = require("UI.HmGui.Styles.MenuStyle")
local Common = require("UI.HmGui.Components.MainMenu.MenuWidgets.Common")
local C = MenuStyle.color
local widget = Common.widget

---The × in a modal's top right corner.
---@param args { x: number, y: number, callback: function }
local function Close(args)
    return widget(function(self, ctx)
        MenuStyle.beginAt(ctx, args.x, args.y, 36, 36)
        local hover = Gui:isMouseOver(FocusType.Mouse)
        Gui:text("×", MenuStyle.fontOf("regular", 30, ctx.s), hover and C.textHover or C.dim)
        Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)
        Gui:endContainer()
        if hover and MenuStyle.clicked() then
            MenuStyle.playClick()
            args.callback()
        end
    end)
end

-- Add to the global UIComponent table
UIComponent.MenuClose = Close

return Close
