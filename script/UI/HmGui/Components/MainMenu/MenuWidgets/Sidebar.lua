local MenuStyle = require("UI.HmGui.Styles.MenuStyle")
local Common = require("UI.HmGui.Components.MainMenu.MenuWidgets.Common")
local C = MenuStyle.color
local renderChildren, widget = Common.renderChildren, Common.widget

---The sidebar: full window height at the left edge. Its contents are placed
---relative to its left edge and to the top of the design area.
---@param args { contents: table|function }
local function Sidebar(args)
    return widget(function(self, ctx)
        Gui:beginStackContainer()
        Gui:setPosition(MenuStyle.SIDEBAR_X, 0)
        Gui:setFixedWidth(MenuStyle.SIDEBAR_WIDTH)
        Gui:setPercentHeight(100)
        Gui:setBackgroundColor(C.sidebar)
        renderChildren(args.contents, { s = ctx.s, ox = 0, oy = ctx.oy })
        Gui:endContainer()
    end)
end

-- Add to the global UIComponent table
UIComponent.MenuSidebar = Sidebar

return Sidebar
