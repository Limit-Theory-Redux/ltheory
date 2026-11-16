local MenuStyle = require("UI.HmGui.Styles.MenuStyle")
local Common = require("UI.HmGui.Components.MainMenu.MenuWidgets.Common")
local C = MenuStyle.color
local renderChildren, widget = Common.renderChildren, Common.widget

---A layer over the whole window that dims what is under it and takes the mouse
---from it, for a modal on top of a screen. Its contents are placed like the
---screen's. Nothing is drawn while `visible()` is false.
---@param args { visible: function, contents: table|function }
local function Overlay(args)
    return widget(function(self, ctx)
        if not args.visible() then
            return
        end
        Gui:beginLayer()
        Gui:beginStackContainer()
        Gui:setAlignment(AlignHorizontal.Left, AlignVertical.Top)
        Gui:setFixedSize(Window:width(), Window:height())
        Gui:setBackgroundColor(C.overlay)
        Gui:isMouseOver(FocusType.Mouse) -- the widgets under the overlay don't get the mouse
        renderChildren(args.contents, ctx)
        Gui:endContainer()
        Gui:endLayer()
    end)
end

-- Add to the global UIComponent table
UIComponent.MenuOverlay = Overlay

return Overlay
