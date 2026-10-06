local Gradient = require("States.Application")
local ProbePaths = require("States.App.Rendering.ProbePaths")

---@class RenderingGradient: Application
local RenderingGradient = Subclass("RenderingGradient", Gradient)

function RenderingGradient:onInit()
    self.shader = Cache.Shader("ui", "gradient")
    self.readbackPath = ProbePaths.file("gradient-readback.png")
    self.frames = 0
end

function RenderingGradient:eventLoop()
    Gradient.eventLoop(self)
    self.frames = self.frames + 1
    if self.frames == 30 then
        Viewport.Push(0, 0, self.resX, self.resY, true)
        RenderState.PushAllDefaults()
        local probe = Tex2D.Create(self.resX, self.resY, TexFormat.RGBA8)
        probe:push()
        Draw.Clear(0, 0, 0, 1)
        Draw.ClearDepth(1)
        self.shader:start()
        Draw.Rect(0, 0, self.resX, self.resY)
        self.shader:stop()
        probe:pop()
        local tl = probe:sample(0, 0)
        local tr = probe:sample(self.resX - 1, 0)
        local bl = probe:sample(0, self.resY - 1)
        local br = probe:sample(self.resX - 1, self.resY - 1)
        local center = probe:sample(math.floor(self.resX / 2), math.floor(self.resY / 2))
        Log.Info(string.format(
            "[GradientProbe] samples tl=(%.3f,%.3f,%.3f) tr=(%.3f,%.3f,%.3f) bl=(%.3f,%.3f,%.3f) center=(%.3f,%.3f,%.3f) br=(%.3f,%.3f,%.3f)",
            tl.x, tl.y, tl.z, tr.x, tr.y, tr.z, bl.x, bl.y, bl.z,
            center.x, center.y, center.z, br.x, br.y, br.z
        ))
        probe:save(self.readbackPath)
        Viewport.Pop()
        RenderState.PopAll()
        Log.Info("[GradientProbe] offscreen renderer readback saved to %s", self.readbackPath)
    end
    if self.frames >= 300 then
        self:quit()
    end
end

function RenderingGradient:onRender()
    RenderState.PushAllDefaults()
    Draw.Clear(0, 0, 0, 1)

    Viewport.Push(0, 0, self.resX, self.resY, true)
    self.shader:start()
    Draw.Rect(0, 0, self.resX, self.resY)
    self.shader:stop()
    Viewport.Pop()

    RenderState.PopAll()
end

return RenderingGradient
