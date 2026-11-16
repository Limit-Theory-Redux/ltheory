local MenuStyle = require("UI.HmGui.Styles.MenuStyle")
local Common = require("UI.HmGui.Components.MainMenu.MenuWidgets.Common")
local C = MenuStyle.color
local value, widget = Common.value, Common.widget

---The sidebar's nav list: uppercase items, the selected (or hovered) one
---between two thin rules.
---@param args { y: number, items: { title: string, callback: function }[], selected: integer|function|nil, align: string|nil, itemHeight: number|nil, size: number|nil, indent: number|nil }
local function Nav(args)
    local itemHeight = args.itemHeight or 66
    local size = args.size or 30
    local indent = args.indent or 86
    return widget(function(self, ctx)
        local s = ctx.s
        local width = MenuStyle.SIDEBAR_WIDTH
        local selected = value(args.selected)
        for index, item in ipairs(args.items) do
            MenuStyle.beginAt(ctx, 0, args.y + (index - 1) * itemHeight, width, itemHeight)
            local hover = Gui:isMouseOver(FocusType.Mouse)
            if hover or index == selected then
                for _, edge in ipairs({ AlignVertical.Top, AlignVertical.Bottom }) do
                    Gui:rect()
                    Gui:setAlignment(AlignHorizontal.Right, edge)
                    Gui:setFixedSize((width - 14) * s, MenuStyle.border(1, s))
                    Gui:setBackgroundColor(C.rule)
                end
            end
            Gui:text(string.upper(item.title), MenuStyle.fontOf("light", size, s), hover and C.textHover or C.text)
            if args.align == "left" then
                Gui:setAlignment(AlignHorizontal.Left, AlignVertical.Center)
                Gui:setMarginEx(indent * s, 0, 0, 0)
            else
                Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)
            end
            Gui:endContainer()
            if hover and MenuStyle.clicked() then
                MenuStyle.playClick()
                item.callback()
            end
        end
    end)
end

-- Add to the global UIComponent table
UIComponent.MenuNav = Nav

return Nav
