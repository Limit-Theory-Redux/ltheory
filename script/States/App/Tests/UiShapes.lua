--- Deterministic UI coverage: every `DrawEx` shape, both text blends, nested
--- clipping and alpha, an icon, and an HmGui panel (rects, text, images). Used
--- to compare UI output across renderer changes (capture with LTHEORY_CAPTURE).
local Test = require('States.Application')
local DrawEx = require('UI.DrawEx')

function Test:getTitle() return 'UI Shapes' end

function Test:onInit()
    self.icon = Tex2D.Load('./res/images/LTR-logo-icon.png')
end

function Test:onInput() end

function Test:onUpdate(dt)
    Gui:beginGui(self.resX, self.resY)

    Gui:beginVerticalContainer()
    Gui:text('HmGui text sample', Cache.Font('Unageo-Medium', 24), Color(1.0, 1.0, 1.0, 1.0))
    Gui:text('Second line, smaller', Cache.Font('Unageo-Medium', 14), Color(0.7, 0.9, 1.0, 1.0))
    Gui:button('Button A')
    Gui:button('Button B')
    Gui:checkbox('Checkbox', true)
    Gui:image(self.icon)
    Gui:endContainer()
    Gui:setAlignment(AlignHorizontal.Right, AlignVertical.Bottom)

    Gui:endGui()
end

function Test:onRender()
    self:immediateUI(function()
        local c = Color(0.3, 0.7, 1.0, 0.9)
        local g = Color(0.4, 1.0, 0.4, 0.8)
        local o = Color(1.0, 0.6, 0.2, 1.0)

        DrawEx.SimpleRect(20, 20, 1240, 40, Color(0.05, 0.05, 0.1, 0.8))
        DrawEx.TextAlpha('Unageo-Medium', 'DrawEx shapes: alpha text', 14, 30, 24, 400, 20, 1, 1, 1, 1, 0, 0.5)
        DrawEx.TextAdditive('Unageo-Medium', 'additive text 0123456789', 14, 500, 24, 400, 20, 0.9, 0.9, 0.2, 1, 0, 0.5)

        DrawEx.Rect(40, 100, 160, 80, c)
        DrawEx.Panel(240, 100, 160, 80, Color(0.2, 0.2, 0.3, 1.0), 0.8)
        DrawEx.PanelGlow(440, 100, 160, 80, g)
        DrawEx.Grid(640, 100, 160, 80, o)
        DrawEx.Circle(880, 140, 30, c)
        DrawEx.Hex(1000, 140, 30, g)
        DrawEx.Ring(1120, 140, 30, o, false)
        DrawEx.Ring(1200, 140, 30, c, true)

        DrawEx.RingDim(100, 300, 40, c)
        DrawEx.Wedge(240, 300, 30, 50, 0.1, 0.25, g)
        DrawEx.Tri(340, 260, 400, 340, 300, 340, o)
        DrawEx.Line(440, 260, 560, 340, c, false)
        DrawEx.Line(440, 340, 560, 260, g, true)
        DrawEx.Point(620, 300, 12, c)
        DrawEx.PointGlow(680, 300, 12, o)
        DrawEx.Cross(740, 300, 20, g)
        DrawEx.RectOutline(800, 260, 80, 80, c)
        DrawEx.Icon(self.icon, 960, 260, 80, 80, Color(1, 0.8, 0.4, 1))
        DrawEx.Arrow(Vec2f(1100, 300), Vec2f(20, 0), g)

        -- Clipping and alpha.
        ClipRect.Push(400, 420, 200, 100)
        DrawEx.SimpleRect(380, 400, 300, 150, Color(0.8, 0.2, 0.2, 0.6))
        DrawEx.Circle(500, 470, 60, c)
        DrawEx.TextAlpha('Unageo-Medium', 'clipped text that is too long to fit', 16, 380, 450, 300, 20, 1, 1, 1, 1, 0, 0.5)
        ClipRect.Pop()

        DrawEx.PushAlpha(0.5)
        DrawEx.SimpleRect(700, 420, 160, 100, Color(0.2, 0.8, 0.2, 1.0))
        DrawEx.TextAlpha('Unageo-Medium', 'half alpha', 16, 710, 450, 140, 20, 1, 1, 1, 1, 0, 0.5)
        DrawEx.PopAlpha()

        for i = 0, 9 do
            DrawEx.SimpleRect(900 + i * 34, 420, 30, 20 + i * 8, Color(i / 9, 0.5, 1 - i / 9, 1))
        end
        DrawEx.TextAlpha('Unageo-Medium', 'The quick brown fox jumps over the lazy dog', 11, 40, 600, 600, 16, 1, 1, 1, 1, 0, 0.5)
        DrawEx.TextAlpha('Unageo-Medium', 'The quick brown fox jumps over the lazy dog', 22, 40, 630, 800, 28, 1, 1, 1, 1, 0, 0.5)
        DrawEx.TextAlpha('Unageo-Medium', 'Kerning: AVATAR WAVE To Ty', 18, 40, 670, 800, 24, 1, 0.8, 0.6, 1, 0, 0.5)

        Gui:draw()
    end)
end

return Test
