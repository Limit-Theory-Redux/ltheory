--- Widgets for the main menu screens, in the style of `UI/HmGui/Styles/MenuStyle.lua`:
--- one file per widget in this folder, each adding itself to `UIComponent` as
--- `UIComponent.Menu<Name>`. This file holds what they share.
---
--- Every widget is placed at a design position (x, y in design pixels, see
--- MenuStyle) inside its parent: `MenuScreen` at the top, then `MenuSidebar`,
--- `MenuPanel` (a panel or a modal) and their children. A widget's `render`
--- takes the parent's `MenuContext`; at the top of a view it computes its own.
---
--- Values are read every frame through `get`/`getIndex` functions and changed
--- through `set`/`setIndex`, so a widget can be made once and kept.
local Common = {}

---Read a value that may be given as a function.
function Common.value(v, ...)
    if type(v) == "function" then
        return v(...)
    end
    return v
end

---A tooltip for a widget, or nil. Render it right after the widget's container.
function Common.makeToolTip(text)
    if not text then
        return nil
    end
    return UIComponent.ToolTip { text = function() return Common.value(text) end }
end

---Render children that are given as a list, or as a function returning one.
function Common.renderChildren(children, ctx)
    for _, child in ipairs(Common.value(children) or {}) do
        if type(child) == "function" then
            child = child()
        end
        if child then
            child:render(ctx)
        end
    end
end

---A widget table the views can hold: `render(self, ctx)`, and a `state` so a
---view can render it at its top level.
function Common.widget(render)
    return { state = {}, render = render }
end

return Common
