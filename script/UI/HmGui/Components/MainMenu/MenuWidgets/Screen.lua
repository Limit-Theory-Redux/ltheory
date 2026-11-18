local MenuStyle = require("UI.HmGui.Styles.MenuStyle")
local Common = require("UI.HmGui.Components.MainMenu.MenuWidgets.Common")
local renderChildren, widget = Common.renderChildren, Common.widget

---The whole window as a stack container, scaled to the window (the design area
---fits it, `MenuStyle.context`); its contents are placed in it in design pixels.
---@param args { contents: table|function }
local function Screen(args)
    return widget(function(self, ctx)
        ctx = ctx or MenuStyle.context()
        Gui:beginStackContainer()
        Gui:setScale(ctx.s)
        Gui:setAlignment(AlignHorizontal.Left, AlignVertical.Top)
        Gui:setPercentSize(100, 100) -- the window: percentages aren't scaled
        renderChildren(args.contents, ctx)
        Gui:endContainer()
    end)
end

-- Add to the global UIComponent table
UIComponent.MenuScreen = Screen

return Screen
