local Test = require('States.Application')
local rng = RNG.FromTime()

local useRenderer = true


function Test:onInit()
    self.renderer = Renderer()
end

function Test:onInput() end

function Test:scrollArea()
    Gui:setPropertyBool(GuiProperties.ScrollAreaHScrollShowId, false)
    Gui:beginScrollArea()
    Gui:setBorder(3, Color(0,1,0,1));

    Gui:beginVerticalContainer()
    Gui:setBorder(3, Color(1,0,0,1))
    Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)
    Gui:setFixedSize(600, 700)
    Gui:setChildrenAlignment(AlignHorizontal.Stretch, AlignVertical.Stretch)

    Gui:button("Button1")
    Gui:button("Button2")
    Gui:button("Button3")

    Gui:endContainer()

    Gui:endScrollArea(InputInstance)
    Gui:setBorder(3, Color(0,0,1,1));
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
