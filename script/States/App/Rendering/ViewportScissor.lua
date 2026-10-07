local Application = require("States.Application")

---@class RenderingViewportScissor: Application
local RenderingViewportScissor = Subclass("RenderingViewportScissor", Application)

local BACKGROUND = {0.02, 0.04, 0.08, 1.0}
local OUTER = {x = 160, y = 90, w = 960, h = 540}
local INNER = {x = 240, y = 150, w = 800, h = 420}

function RenderingViewportScissor:onInit()
    self.shader = Cache.Shader("ui", "gradient")
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
    RenderState.PushAllDefaults()
    if not self.passDesc then
        self.passDesc = RenderPassDesc.Create("ViewportScissor")
        self.passDesc:backbuffer(self.resX, self.resY, LoadOp.Clear,
            BACKGROUND[1], BACKGROUND[2], BACKGROUND[3], BACKGROUND[4])
    end
    local pass = Renderer:beginPass(self.passDesc)

    if self.frames <= 30 then
        Viewport.Push(OUTER.x, OUTER.y, OUTER.w, OUTER.h, true)
        ClipRect.PushDisabled()
        self.shader:start()
        Draw.Rect(0, 0, OUTER.w, OUTER.h)
        self.shader:stop()
        ClipRect.Pop()
        Viewport.Pop()
    else
        Viewport.Push(0, 0, self.resX, self.resY, true)
        ClipRect.Push(INNER.x, INNER.y, INNER.w, INNER.h)
        self.shader:start()
        Draw.Rect(0, 0, self.resX, self.resY)
        self.shader:stop()
        ClipRect.Pop()
        Viewport.Pop()
    end

    pass:finish()
    RenderState.PopAll()
end

return RenderingViewportScissor
