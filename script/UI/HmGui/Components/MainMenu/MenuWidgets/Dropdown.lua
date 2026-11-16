local MenuStyle = require("UI.HmGui.Styles.MenuStyle")
local Common = require("UI.HmGui.Components.MainMenu.MenuWidgets.Common")
local C = MenuStyle.color
local value, makeToolTip, widget = Common.value, Common.makeToolTip, Common.widget

---A dropdown: the choice in a field, a dark box with a white triangle at its
---right end, and the list below it when open.
---@param args { x: number, y: number, w: number, selections: string[]|function, getIndex: function, setIndex: function, enabled: function|nil, toolTip: string|function|nil }
local function Dropdown(args)
    local toolTip = makeToolTip(args.toolTip)
    local H, ROW_H = 40, 34
    local open = false

    local dropdown
    dropdown = widget(function(self, ctx)
        local enabled = not args.enabled or args.enabled()
        local selections = value(args.selections) or {}
        local index = args.getIndex()
        local w = args.w

        MenuStyle.beginAt(ctx, args.x, args.y, w, H)
        local hoverField = enabled and Gui:isMouseOver(FocusType.Mouse)
        Gui:setBackgroundColor(hoverField and C.fieldHover or C.field)
        Gui:setBorderWidth(2)
        Gui:setBorderColor(C.fieldBorder)
        Gui:text(selections[index] or "", MenuStyle.fontOf("light", 26), enabled and C.text or C.disabled)
        Gui:setAlignment(AlignHorizontal.Left, AlignVertical.Center)
        Gui:setMarginEx(10, 0, 0, 0)
        Gui:setMaxWidth(w - 60) -- left of the arrow box; a longer name ends with "…"
        Gui:rect()
        Gui:setAlignment(AlignHorizontal.Right, AlignVertical.Stretch)
        Gui:setFixedWidth(38)
        Gui:setBackgroundColor(C.arrowBox)
        Gui:triangle(enabled and C.arrow or C.disabled, 0, 0, 1, 0, 0.5, 1, false)
        Gui:setAlignment(AlignHorizontal.Right, AlignVertical.Center)
        Gui:setFixedSize(22, 17)
        Gui:setMarginEx(0, 0, 8, 0)
        Gui:endContainer()
        if toolTip and not open then toolTip:render() end

        if hoverField and MenuStyle.clicked() then
            MenuStyle.playClick()
            open = not open
        elseif not enabled then
            open = false
        elseif open then
            self:renderList(w, selections, index)
        end
    end)

    ---The open list, on a layer of its own below the field; as wide as the
    ---field, or wider when a choice's name needs it.
    dropdown.renderList = function(self, w, selections, index)
        local listFont = MenuStyle.fontOf("light", 24)
        Gui:beginLayerBelow()
        Gui:beginVerticalContainer()
        Gui:setAlignment(AlignHorizontal.Left, AlignVertical.Top)
        Gui:setMinWidth(w)
        Gui:setPadding(0, 4)
        Gui:setBackgroundColor(C.field)
        Gui:setBorderWidth(2)
        Gui:setBorderColor(C.fieldBorder)
        local picked
        for i, name in ipairs(selections) do
            Gui:beginStackContainer()
            Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Top)
            Gui:setFixedHeight(ROW_H)
            local hover = Gui:isMouseOver(FocusType.Mouse)
            if hover then
                Gui:setBackgroundColor(C.listHover)
            elseif i == index then
                Gui:setBackgroundColor(C.listSelected)
            end
            Gui:text(name, listFont, hover and C.textHover or C.text)
            Gui:setAlignment(AlignHorizontal.Left, AlignVertical.Center)
            Gui:setMarginEx(10, 0, 20, 0)
            Gui:endContainer()
            if hover and MenuStyle.clicked() then
                picked = i
            end
        end
        Gui:endContainer()
        local hoverList = Gui:isMouseOver(FocusType.Mouse)
        Gui:endLayer()

        if picked then
            MenuStyle.playClick()
            open = false
            args.setIndex(picked)
        elseif MenuStyle.clicked() and not hoverList then
            open = false
        end
    end

    return dropdown
end

-- Add to the global UIComponent table
UIComponent.MenuDropdown = Dropdown

return Dropdown
