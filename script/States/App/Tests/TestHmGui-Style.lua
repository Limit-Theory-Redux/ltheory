local Test = require('States.Application')
local rng = RNG.FromTime()

local useRenderer = true


function Test:onInit()
    self.renderer = Renderer()
end

function Test:onInput() end

function Test:showElements()
    -- Gui:beginWindow('HmGui Elements Test', InputInstance)
    Gui:beginHorizontalContainer()
    Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Stretch)

    Gui:beginVerticalContainer()
    Gui:setVerticalAlignment(AlignVertical.Stretch)

    Gui:button("Button1")
    Gui:button("Button2")
    Gui:button("Button3")

    Gui:endContainer()

    Gui:endContainer()
    -- Gui:endWindow()
end

function Test:onUpdate(dt)
    Profiler.Begin('Gui:update')
    Gui:beginGui(self.resX, self.resY, InputInstance)
    self:showElements()
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
