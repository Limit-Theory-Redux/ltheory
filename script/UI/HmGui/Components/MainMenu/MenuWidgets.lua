--- Widgets for the main menu screens, in the style of `UI/HmGui/Styles/MenuStyle.lua`.
---
--- Every widget is placed at a design position (x, y in design pixels, see
--- MenuStyle) inside its parent: `MenuScreen` at the top, then `MenuSidebar`,
--- `MenuPanel` (a panel or a modal) and their children. A widget's `render`
--- takes the parent's `MenuContext`; at the top of a view it computes its own.
---
--- Values are read every frame through `get`/`getIndex` functions and changed
--- through `set`/`setIndex`, so a widget can be made once and kept.
local MenuStyle = require("UI.HmGui.Styles.MenuStyle")
local C = MenuStyle.color

local MenuWidgets = {}

---Read a value that may be given as a function.
local function value(v, ...)
    if type(v) == "function" then
        return v(...)
    end
    return v
end

---A tooltip for a widget, or nil. Render it right after the widget's container.
local function makeToolTip(text)
    if not text then
        return nil
    end
    return UIComponent.ToolTip { text = function() return value(text) end }
end

---Render children that are given as a list, or as a function returning one.
local function renderChildren(children, ctx)
    for _, child in ipairs(value(children) or {}) do
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
local function widget(render)
    return { state = {}, render = render }
end

-- SCREEN -----------------------------------------------------------------------

---The whole window as a stack container; its contents are placed in it.
---@param args { contents: table|function }
function MenuWidgets.Screen(args)
    return widget(function(self, ctx)
        ctx = ctx or MenuStyle.context()
        Gui:beginStackContainer()
        Gui:setAlignment(AlignHorizontal.Left, AlignVertical.Top)
        Gui:setFixedSize(Window:width(), Window:height())
        renderChildren(args.contents, ctx)
        Gui:endContainer()
    end)
end

---The sidebar: full window height at the left edge. Its contents are placed
---relative to its left edge and to the top of the design area.
---@param args { contents: table|function }
function MenuWidgets.Sidebar(args)
    return widget(function(self, ctx)
        local s = ctx.s
        Gui:beginStackContainer()
        Gui:setPosition(MenuStyle.SIDEBAR_X * s, 0)
        Gui:setFixedSize(MenuStyle.SIDEBAR_WIDTH * s, Window:height())
        Gui:setBackgroundColor(C.sidebar)
        renderChildren(args.contents, { s = s, ox = 0, oy = ctx.oy })
        Gui:endContainer()
    end)
end

---A dark bordered panel, or a modal when it has a `band` (a header strip of
---that height). Its contents are placed relative to its top left corner.
---@param args { x: number, y: number, w: number, h: number, band: number|nil, contents: table|function }
function MenuWidgets.Panel(args)
    return widget(function(self, ctx)
        local s = ctx.s
        MenuStyle.beginAt(ctx, args.x, args.y, args.w, args.h)
        Gui:setBackgroundColor(C.panel)
        Gui:setBorderWidth(MenuStyle.border(2, s))
        Gui:setBorderColor(C.panelBorder)
        if args.band then
            Gui:rect()
            Gui:setPosition(0, 0)
            Gui:setFixedSize(args.w * s, args.band * s)
            Gui:setBackgroundColor(C.band)
            Gui:rect()
            Gui:setPosition(0, args.band * s)
            Gui:setFixedSize(args.w * s, MenuStyle.border(1, s))
            Gui:setBackgroundColor(C.bandRule)
        end
        renderChildren(args.contents, MenuStyle.inside(ctx))
        Gui:endContainer()
    end)
end

---A layer over the whole window that dims what is under it and takes the mouse
---from it, for a modal on top of a screen. Its contents are placed like the
---screen's. Nothing is drawn while `visible()` is false.
---@param args { visible: function, contents: table|function }
function MenuWidgets.Overlay(args)
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

-- TEXT AND IMAGES -------------------------------------------------------------

---A line of text, centred vertically in `h` (default: a line of its size), so
---labels line up with a control of the same y and h. `align` "left" puts its
---left edge at x (and cuts it at `w`, if given), "right" its right edge at x
---(within `w` to the left of x, default: from the parent's left edge), and
---"center" centres it in `w` from x. A text longer than its box ends with "…".
---@param args { x: number, y: number, text: string|function, size: number|nil, font: string|nil, color: Color|function|nil, align: string|nil, w: number|nil, h: number|nil, visible: function|nil }
function MenuWidgets.Label(args)
    local align = args.align or "left"
    local hAlign = ({ left = AlignHorizontal.Left, right = AlignHorizontal.Right, center = AlignHorizontal.Center })[align]
    return widget(function(self, ctx)
        if args.visible and not args.visible() then
            return
        end
        local s = ctx.s
        local size = args.size or 27
        local h = args.h or size * 1.5
        local w = args.w
        if align == "right" then
            w = w or args.x
            MenuStyle.beginAt(ctx, args.x - w, args.y, w, h)
        elseif w then
            MenuStyle.beginAt(ctx, args.x, args.y, w, h)
        else
            MenuStyle.beginAt(ctx, args.x, args.y)
            Gui:setFixedHeight(h * s)
        end
        Gui:text(tostring(value(args.text)), MenuStyle.fontOf(args.font or "light", size, s), value(args.color) or C.label)
        Gui:setAlignment(hAlign, AlignVertical.Center)
        if w then
            Gui:setMaxWidth(w * s)
        end
        Gui:endContainer()
    end)
end

---An image at a design position and width (height from the image's aspect).
---@param args { x: number, y: number, w: number, image: Tex2D }
function MenuWidgets.Image(args)
    local size = args.image:getSize()
    local h = args.w * size.y / size.x
    return widget(function(self, ctx)
        MenuStyle.beginAt(ctx, args.x, args.y, args.w, h)
        Gui:image(args.image)
        Gui:setFixedSize(args.w * ctx.s, h * ctx.s)
        Gui:endContainer()
    end)
end

-- NAVIGATION AND BUTTONS -------------------------------------------------------

---The sidebar's nav list: uppercase items, the selected (or hovered) one
---between two thin rules.
---@param args { y: number, items: { title: string, callback: function }[], selected: integer|function|nil, align: string|nil, itemHeight: number|nil, size: number|nil, indent: number|nil }
function MenuWidgets.Nav(args)
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

---The main menu's nav (New Game … Exit Game), shared by the Main, New Game and
---Load Game screens.
---@param args { y: number|nil, selected: integer|nil }
function MenuWidgets.MainNav(args)
    local function goTo(view)
        return function()
            require("UI.HmGui.UICore.UIRouter"):getCurrentPage():setView(view)
        end
    end
    return MenuWidgets.Nav {
        y = args.y or 225,
        selected = args.selected,
        items = {
            { title = "New Game",  callback = goTo("Newgame") },
            { title = "Load Game", callback = goTo("Loadgame") },
            { title = "Settings",  callback = goTo("Settings") },
            { title = "Credits",   callback = goTo("Credits") },
            { title = "Exit Game", callback = function() LimitTheoryRedux:exit() end }, -- pre-exit operations, then exit
        }
    }
end

---The game version, bottom left in the sidebar.
function MenuWidgets.Version()
    return MenuWidgets.Label {
        x = 12, y = 1082, size = 18, font = "regular", color = C.dim,
        text = function() return "v. " .. tostring(Config.gameVersion) end
    }
end

---A boxed button. `style` "go" is the green one (Start).
---@param args { x: number, y: number, w: number, h: number, title: string|function, size: number|nil, style: string|nil, uppercase: boolean|nil, enabled: function|nil, toolTip: string|function|nil, callback: function }
function MenuWidgets.Button(args)
    local toolTip = makeToolTip(args.toolTip)
    return widget(function(self, ctx)
        local s = ctx.s
        local enabled = not args.enabled or args.enabled()
        MenuStyle.beginAt(ctx, args.x, args.y, args.w, args.h)
        local hover = enabled and Gui:isMouseOver(FocusType.Mouse)
        local go = args.style == "go"
        if go then
            Gui:setBackgroundColor(hover and C.goHover or C.go)
            Gui:setBorderColor(C.goBorder)
        else
            Gui:setBackgroundColor(hover and C.buttonHover or C.button)
            Gui:setBorderColor(C.buttonBorder)
        end
        Gui:setBorderWidth(MenuStyle.border(1, s))
        local title = value(args.title)
        if args.uppercase ~= false then
            title = string.upper(title)
        end
        Gui:text(title, MenuStyle.fontOf("light", args.size or 23, s), enabled and C.text or C.disabled)
        Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)
        Gui:endContainer()
        if toolTip then toolTip:render() end
        if hover and MenuStyle.clicked() then
            MenuStyle.playClick()
            args.callback()
        end
    end)
end

---The × in a modal's top right corner.
---@param args { x: number, y: number, callback: function }
function MenuWidgets.Close(args)
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

-- CONTROLS ---------------------------------------------------------------------

---A checkbox: a dark square with a blue square inside when on.
---@param args { x: number, y: number, get: function, set: function, toolTip: string|function|nil }
function MenuWidgets.Checkbox(args)
    local toolTip = makeToolTip(args.toolTip)
    return widget(function(self, ctx)
        local s = ctx.s
        MenuStyle.beginAt(ctx, args.x, args.y, 40, 40)
        local hover = Gui:isMouseOver(FocusType.Mouse)
        Gui:setBackgroundColor(hover and C.boxHover or C.box)
        Gui:setBorderWidth(MenuStyle.border(2, s))
        Gui:setBorderColor(C.fieldBorder)
        if args.get() then
            Gui:rect()
            Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)
            Gui:setFixedSize(22 * s, 22 * s)
            Gui:setBackgroundColor(C.check)
        end
        Gui:endContainer()
        if toolTip then toolTip:render() end
        if hover and MenuStyle.clicked() then
            MenuStyle.playClick()
            args.set(not args.get())
        end
    end)
end

---A slider with its label above and its value to the right of the bar. Drag
---anywhere on the bar; the value snaps to `increment`.
---@param args { x: number, y: number, label: string, min: number, max: number, increment: number|nil, get: function, set: function, format: function|nil, toolTip: string|function|nil }
function MenuWidgets.Slider(args)
    local toolTip = makeToolTip(args.toolTip)
    local BAR_W, BAR_H = 372, 36
    local label = MenuWidgets.Label { x = args.x, y = args.y - 4, h = 36, text = args.label }
    local format = args.format or function(v) return string.format("%d%%", math.floor(v * 100 + 0.5)) end
    local valueLabel = MenuWidgets.Label { x = args.x + BAR_W + 20, y = args.y + 36, h = BAR_H,
        text = function() return format(args.get()) end }
    local dragging = false

    return widget(function(self, ctx)
        local s = ctx.s
        local range = args.max - args.min
        label:render(ctx)

        local barX, barY = args.x, args.y + 36
        MenuStyle.beginAt(ctx, barX, barY, BAR_W, BAR_H)
        local hover = Gui:isMouseOver(FocusType.Mouse)
        local pos = Gui:containerPos()
        local mouse = Input:mouse()
        if hover and mouse:isPressed(MouseControl.Left) then
            dragging = true
        elseif not mouse:isDown(MouseControl.Left) then
            dragging = false
        end
        if dragging and range > 0 then
            local t = (mouse:position().x - pos.x) / (BAR_W * s)
            t = math.max(0, math.min(1, t))
            local v = args.min + t * range
            if args.increment and args.increment > 0 then
                v = args.min + math.floor((v - args.min) / args.increment + 0.5) * args.increment
                v = math.max(args.min, math.min(args.max, v))
            end
            if v ~= args.get() then
                args.set(v)
            end
        end

        local frac = 0
        if range > 0 then
            frac = math.max(0, math.min(1, (args.get() - args.min) / range))
        end
        Gui:setBackgroundColor(C.track)
        Gui:setBorderWidth(MenuStyle.border(2, s))
        Gui:setBorderColor(C.fieldBorder)
        if frac > 0 then
            Gui:rect()
            Gui:setAlignment(AlignHorizontal.Left, AlignVertical.Stretch)
            Gui:setFixedWidth(frac * (BAR_W - 4) * s)
            Gui:setBackgroundColor((hover or dragging) and C.fillHover or C.fill)
        end
        Gui:endContainer()
        if toolTip then toolTip:render() end

        -- the thumb, on top of the bar and sticking out of it
        MenuStyle.beginAt(ctx, barX + frac * BAR_W - 4.5, barY - 3, 9, BAR_H + 6)
        Gui:setBackgroundColor(C.thumb)
        Gui:endContainer()

        valueLabel:render(ctx)
    end)
end

---A dropdown: the choice in a field, a dark box with a white triangle at its
---right end, and the list below it when open.
---@param args { x: number, y: number, w: number, selections: string[]|function, getIndex: function, setIndex: function, enabled: function|nil, toolTip: string|function|nil }
function MenuWidgets.Dropdown(args)
    local toolTip = makeToolTip(args.toolTip)
    local H, ROW_H = 40, 34
    local open = false

    local dropdown
    dropdown = widget(function(self, ctx)
        local s = ctx.s
        local enabled = not args.enabled or args.enabled()
        local selections = value(args.selections) or {}
        local index = args.getIndex()
        local w = args.w

        MenuStyle.beginAt(ctx, args.x, args.y, w, H)
        local hoverField = enabled and Gui:isMouseOver(FocusType.Mouse)
        Gui:setBackgroundColor(hoverField and C.fieldHover or C.field)
        Gui:setBorderWidth(MenuStyle.border(2, s))
        Gui:setBorderColor(C.fieldBorder)
        Gui:text(selections[index] or "", MenuStyle.fontOf("light", 26, s), enabled and C.text or C.disabled)
        Gui:setAlignment(AlignHorizontal.Left, AlignVertical.Center)
        Gui:setMarginEx(10 * s, 0, 0, 0)
        Gui:setMaxWidth((w - 60) * s) -- left of the arrow box; a longer name ends with "…"
        Gui:rect()
        Gui:setAlignment(AlignHorizontal.Right, AlignVertical.Stretch)
        Gui:setFixedWidth(38 * s)
        Gui:setBackgroundColor(C.arrowBox)
        Gui:triangle(enabled and C.arrow or C.disabled, 0, 0, 1, 0, 0.5, 1, false)
        Gui:setAlignment(AlignHorizontal.Right, AlignVertical.Center)
        Gui:setFixedSize(22 * s, 17 * s)
        Gui:setMarginEx(0, 0, 8 * s, 0)
        Gui:endContainer()
        if toolTip and not open then toolTip:render() end

        if hoverField and MenuStyle.clicked() then
            MenuStyle.playClick()
            open = not open
        elseif not enabled then
            open = false
        elseif open then
            self:renderList(s, w, selections, index)
        end
    end)

    ---The open list, on a layer of its own below the field; as wide as the
    ---field, or wider when a choice's name needs it.
    dropdown.renderList = function(self, s, w, selections, index)
        local listFont = MenuStyle.fontOf("light", 24, s)
        Gui:beginLayerBelow()
        Gui:beginVerticalContainer()
        Gui:setAlignment(AlignHorizontal.Left, AlignVertical.Top)
        Gui:setMinWidth(w * s)
        Gui:setPadding(0, 4 * s)
        Gui:setBackgroundColor(C.field)
        Gui:setBorderWidth(MenuStyle.border(2, s))
        Gui:setBorderColor(C.fieldBorder)
        local picked
        for i, name in ipairs(selections) do
            Gui:beginStackContainer()
            Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Top)
            Gui:setFixedHeight(ROW_H * s)
            local hover = Gui:isMouseOver(FocusType.Mouse)
            if hover then
                Gui:setBackgroundColor(C.listHover)
            elseif i == index then
                Gui:setBackgroundColor(C.listSelected)
            end
            Gui:text(name, listFont, hover and C.textHover or C.text)
            Gui:setAlignment(AlignHorizontal.Left, AlignVertical.Center)
            Gui:setMarginEx(10 * s, 0, 20 * s, 0)
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

---Any Gui code, placed at a design position: `fn(ctx)` runs inside a stack
---container of design size w x h.
---@param args { x: number, y: number, w: number, h: number, fn: function }
function MenuWidgets.Raw(args)
    return widget(function(self, ctx)
        MenuStyle.beginAt(ctx, args.x, args.y, args.w, args.h)
        args.fn(MenuStyle.inside(ctx))
        Gui:endContainer()
    end)
end

-- Add to the global UIComponent table
UIComponent.MenuScreen = MenuWidgets.Screen
UIComponent.MenuSidebar = MenuWidgets.Sidebar
UIComponent.MenuPanel = MenuWidgets.Panel
UIComponent.MenuOverlay = MenuWidgets.Overlay
UIComponent.MenuLabel = MenuWidgets.Label
UIComponent.MenuImage = MenuWidgets.Image
UIComponent.MenuNav = MenuWidgets.Nav
UIComponent.MenuMainNav = MenuWidgets.MainNav
UIComponent.MenuVersion = MenuWidgets.Version
UIComponent.MenuButton = MenuWidgets.Button
UIComponent.MenuClose = MenuWidgets.Close
UIComponent.MenuCheckbox = MenuWidgets.Checkbox
UIComponent.MenuSlider = MenuWidgets.Slider
UIComponent.MenuDropdown = MenuWidgets.Dropdown
UIComponent.MenuRaw = MenuWidgets.Raw

return MenuWidgets
