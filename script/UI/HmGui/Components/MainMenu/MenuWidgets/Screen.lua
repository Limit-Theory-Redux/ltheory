local MenuStyle = require("UI.HmGui.Styles.MenuStyle")
local Common = require("UI.HmGui.Components.MainMenu.MenuWidgets.Common")
local renderChildren, widget = Common.renderChildren, Common.widget

---The whole window as a stack container; its contents are placed in it.
---@param args { contents: table|function }
local function Screen(args)
    return widget(function(self, ctx)
        ctx = ctx or MenuStyle.context()
        Gui:beginStackContainer()
        Gui:setAlignment(AlignHorizontal.Left, AlignVertical.Top)
        Gui:setFixedSize(Window:width(), Window:height())
        renderChildren(args.contents, ctx)
        Gui:endContainer()
    end)
end

-- Add to the global UIComponent table
UIComponent.MenuScreen = Screen

return Screen
