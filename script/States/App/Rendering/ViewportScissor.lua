local Application = require("States.Application")

---@class RenderingViewportScissor: Application
local RenderingViewportScissor = Subclass("RenderingViewportScissor", Application)

local BACKGROUND = {0.02, 0.04, 0.08, 1.0}
local OUTER = {x = 160, y = 90, w = 960, h = 540}
local INNER = {x = 240, y = 150, w = 800, h = 420}

function RenderingViewportScissor:onInit()
    self.shader = Cache.Shader("fullscreen", "gradient")
    local desc = PipelineDesc.Create(self.shader)
    desc:vertex(VertexLayout.Fullscreen)
    self.pipeline = Pipeline.Get(desc)
    self.frames = 0
end

function RenderingViewportScissor:eventLoop()
    Application.eventLoop(self)
    self.frames = self.frames + 1
    if self.frames == 1 then
        Log.Info("[ViewportProbe] phase=viewport outer=(160,90,960,540) scissor=disabled")
    elseif self.frames == 31 then
        Log.Info("[ViewportProbe] phase=scissor viewport=(0,0,1280,720) inner=(240,150,800,420)")
    end
    if self.frames >= 60 then
        self:quit()
    end
end

function RenderingViewportScissor:onRender()
    if not self.passDesc then
        self.passDesc = RenderPassDesc.Create("ViewportScissor")
        self.passDesc:backbuffer(self.resX, self.resY, LoadOp.Clear,
            BACKGROUND[1], BACKGROUND[2], BACKGROUND[3], BACKGROUND[4])
    end
    local pass = Renderer:beginPass(self.passDesc)

    pass:setPipeline(self.pipeline)
    if self.frames <= 30 then
        pass:setViewport(OUTER.x, OUTER.y, OUTER.w, OUTER.h)
        ClipRect.PushDisabled()
        pass:drawFullscreen()
        ClipRect.Pop()
    else
        ClipRect.Push(INNER.x, INNER.y, INNER.w, INNER.h)
        pass:drawFullscreen()
        ClipRect.Pop()
    end

    pass:finish()
end

return RenderingViewportScissor
