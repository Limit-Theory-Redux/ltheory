local Test = require('States.Application')
local rng = RNG.FromTime()

local useRenderer = true


function Test:onInit()
    self.renderer = Renderer()
    self.fading = true
end

function Test:onInput() end

function Test:scrollArea()
    Gui:beginVerticalContainer()

    self.fading = Gui:checkbox("Fading", self.fading)

    Gui:setPropertyBool(GuiProperties.ScrollAreaScrollbarVisibilityFading, self.fading)
    Gui:setPropertyColor(GuiProperties.ScrollAreaScrollbarKnobColor, Color(1, 0, 0, 1))
    Gui:setPropertyBool(GuiProperties.ScrollAreaHScrollShow, false)
    Gui:setPropertyColor(GuiProperties.BackgroundColor, Color(0, 0, 1, 1))
    Gui:beginScrollArea(ScrollDirection.All)

    Gui:setPropertyColor(GuiProperties.BackgroundColor, Color(0, 1, 0, 1))
    Gui:beginVerticalContainer()
    Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Top)
    Gui:setChildrenAlignment(AlignHorizontal.Stretch, AlignVertical.Top)

    for i = 1, 21 do
        Gui:button("Button-" .. i)
    end

    Gui:endContainer()
    Gui:setBorderWidth(3);

    Gui:endScrollArea(InputInstance)
    Gui:setBorderWidth(3);
    Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Stretch)

    Gui:endContainer()
    Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)
    Gui:setChildrenHorizontalAlignment(AlignHorizontal.Stretch)
    Gui:setFixedSize(500, 500)
end

function Test:onUpdate(dt)
    Profiler.Begin('Gui:update')

    Gui:setPropertyColor(GuiProperties.BackgroundColor, Color(0, 0, 0, 1))
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
