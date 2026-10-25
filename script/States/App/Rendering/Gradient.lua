local Gradient = require("States.Application")
local ProbePaths = require("States.App.Rendering.ProbePaths")
local ProbeRead = require("States.App.Rendering.ProbeRead")

---@class RenderingGradient: Application
local RenderingGradient = Subclass("RenderingGradient", Gradient)

function RenderingGradient:onInit()
    self.shader = Cache.Shader("fullscreen", "gradient")
    local desc = PipelineDesc.Create(self.shader)
    desc:vertex(VertexLayout.Fullscreen)
    self.pipeline = Pipeline.Get(desc)
    self.readbackPath = ProbePaths.file("gradient-readback.png")
    self.frames = 0
end

function RenderingGradient:eventLoop()
    Gradient.eventLoop(self)
    self.frames = self.frames + 1
    if self.frames == 30 then
        local probe = Tex2D.Create(self.resX, self.resY, TexFormat.RGBA8)
        local probeDesc = RenderPassDesc.Create("Gradient.probe")
        probeDesc:color(0, probe:view(), LoadOp.Clear, 0, 0, 0, 1)
        local probePass = Renderer:beginPass(probeDesc)
        probePass:setPipeline(self.pipeline)
        probePass:drawFullscreen()
        probePass:finish()
        local tl = ProbeRead.sample(probe, 0, 0)
        local tr = ProbeRead.sample(probe, self.resX - 1, 0)
        local bl = ProbeRead.sample(probe, 0, self.resY - 1)
        local br = ProbeRead.sample(probe, self.resX - 1, self.resY - 1)
        local center = ProbeRead.sample(probe, math.floor(self.resX / 2), math.floor(self.resY / 2))
        Log.Info(string.format(
            "[GradientProbe] samples tl=(%.3f,%.3f,%.3f) tr=(%.3f,%.3f,%.3f) bl=(%.3f,%.3f,%.3f) center=(%.3f,%.3f,%.3f) br=(%.3f,%.3f,%.3f)",
            tl.x, tl.y, tl.z, tr.x, tr.y, tr.z, bl.x, bl.y, bl.z,
            center.x, center.y, center.z, br.x, br.y, br.z
        ))
        probe:save(self.readbackPath)
        Log.Info("[GradientProbe] offscreen renderer readback saved to %s", self.readbackPath)
    end
    if self.frames >= 300 then
        self:quit()
    end
end

function RenderingGradient:onRender()
    if not self.passDesc then
        self.passDesc = RenderPassDesc.Create("Gradient")
        self.passDesc:backbuffer(self.resX, self.resY, LoadOp.Clear, 0, 0, 0, 1)
    end
    local pass = Renderer:beginPass(self.passDesc)
    pass:setPipeline(self.pipeline)
    pass:drawFullscreen()
    pass:finish()
end

return RenderingGradient
