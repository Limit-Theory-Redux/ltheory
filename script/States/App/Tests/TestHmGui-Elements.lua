local Test = require('States.Application')
local rng = RNG.FromTime()

local useRenderer = true


function Test:onInit()
    self.renderer = Renderer()
end

function Test:onInput() end

function Test:scrollArea()
    Gui:setPropertyBool(GuiProperties.ScrollAreaHScrollShowId, false)
    Gui:beginScrollArea(ScrollDirection.Vertical)
    Gui:setChildrenAlignment(AlignHorizontal.Stretch, AlignVertical.Stretch)

    Gui:button("Button1")
    Gui:button("Button2")
    Gui:button("Button3")
    Gui:button("Button4")
    Gui:button("Button5")
    Gui:button("Button6")
    Gui:button("Button7")
    Gui:button("Button8")
    Gui:button("Button9")
    Gui:button("Button10")
    Gui:button("Button11")
    Gui:button("Button12")
    Gui:button("Button13")
    Gui:button("Button14")
    Gui:button("Button15")
    Gui:button("Button16")
    Gui:button("Button17")
    Gui:button("Button18")
    Gui:button("Button19")
    Gui:button("Button20")
    Gui:button("Button21")

    Gui:endScrollArea(InputInstance)
    Gui:setBorder(3, Color(0, 0, 1, 1));
    Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)
    Gui:setFixedSize(500, 500)
end

function Test:onUpdate(dt)
    Profiler.Begin('Gui:update')
    Gui:beginGui(self.resX, self.resY, InputInstance)
    self:scrollArea()
    Gui:endGui(InputInstance)
    Profiler.End()
end

function Test:onDraw()
    if useRenderer then
        self.renderer:start(self.resX, self.resY)
        Viewport.Push(0, 0, self.resX, self.resY, true)
        Gui:draw()
        Viewport.Pop()
        self.renderer:stop()
        self.renderer:present(0, 0, self.resX, self.resY)
    else
        Gui:draw()
    end
end

return Test
