local MenuStyle = require("UI.HmGui.Styles.MenuStyle")
local Common = require("UI.HmGui.Components.MainMenu.MenuWidgets.Common")
local Label = require("UI.HmGui.Components.MainMenu.MenuWidgets.Label")
local C = MenuStyle.color
local makeToolTip, widget = Common.makeToolTip, Common.widget

---A slider with its label above and its value to the right of the bar. Drag
---anywhere on the bar; the value snaps to `increment`.
---@param args { x: number, y: number, label: string, min: number, max: number, increment: number|nil, get: function, set: function, format: function|nil, toolTip: string|function|nil }
local function Slider(args)
    local toolTip = makeToolTip(args.toolTip)
    local BAR_W, BAR_H = 372, 36
    local label = Label { x = args.x, y = args.y - 4, h = 36, text = args.label }
    local format = args.format or function(v) return string.format("%d%%", math.floor(v * 100 + 0.5)) end
    local valueLabel = Label { x = args.x + BAR_W + 20, y = args.y + 36, h = BAR_H,
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
            local t = (mouse:position().x - pos.x) / (BAR_W * s) -- window pixels
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
        Gui:setBorderWidth(2)
        Gui:setBorderColor(C.fieldBorder)
        if frac > 0 then
            Gui:rect()
            Gui:setAlignment(AlignHorizontal.Left, AlignVertical.Stretch)
            Gui:setFixedWidth(frac * (BAR_W - 4))
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

-- Add to the global UIComponent table
UIComponent.MenuSlider = Slider

return Slider
