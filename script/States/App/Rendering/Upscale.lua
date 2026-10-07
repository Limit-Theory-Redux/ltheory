local Application = require("States.Application")
local ProbePaths = require("States.App.Rendering.ProbePaths")
local ProbeRead = require("States.App.Rendering.ProbeRead")

---@class RenderingUpscale: Application
local RenderingUpscale = Subclass("RenderingUpscale", Application)

local SOURCE_W = 320
local SOURCE_H = 180

function RenderingUpscale:onInit()
    self.source = Tex2D.Create(SOURCE_W, SOURCE_H, TexFormat.RGBA8)
    self.passDesc = RenderPassDesc.Create("Upscale.source")
    self.passDesc:color(0, self.source:view(), LoadOp.Clear, 0, 0, 0, 1)
    self.presentDesc = RenderPassDesc.Create("Upscale.present")
    self.gradient = Cache.Shader("fullscreen", "gradient")
    self.blit = Cache.Shader("fullscreen_flip", "blit")
    local gradient = PipelineDesc.Create(self.gradient)
    gradient:vertex(VertexLayout.Fullscreen)
    self.gradientPipeline = Pipeline.Get(gradient)
    local blit = PipelineDesc.Create(self.blit)
    blit:vertex(VertexLayout.Fullscreen)
    self.blitPipeline = Pipeline.Get(blit)
    self.sourceArtifactPath = ProbePaths.file("upscale-source.png")
    self.frames = 0
    self.holdFrames = tonumber(os.getenv("UPSCALE_HOLD_FRAMES")) or 40
end

function RenderingUpscale:eventLoop()
    Application.eventLoop(self)
    self.frames = self.frames + 1
    if self.frames == 1 then
        local sourceGeometry = os.getenv("LTHEORY_WGPU") and "1280x720 logical" or "320x180 target"
        Log.Info(string.format(
            "[UpscaleProbe] source=320x180 destination=1280x720 filter=Linear wrap=Clamp geometry=%s hold=%d",
            sourceGeometry,
            self.holdFrames
        ))
    elseif self.frames == 10 then
        local tl = ProbeRead.sample(self.source, 0, 0)
        local tr = ProbeRead.sample(self.source, SOURCE_W - 1, 0)
        local bl = ProbeRead.sample(self.source, 0, SOURCE_H - 1)
        local center = ProbeRead.sample(self.source, math.floor(SOURCE_W / 2), math.floor(SOURCE_H / 2))
        local br = ProbeRead.sample(self.source, SOURCE_W - 1, SOURCE_H - 1)
        Log.Info(string.format(
            "[UpscaleProbe] source samples tl=(%.3f,%.3f,%.3f) tr=(%.3f,%.3f,%.3f) bl=(%.3f,%.3f,%.3f) center=(%.3f,%.3f,%.3f) br=(%.3f,%.3f,%.3f)",
            tl.x, tl.y, tl.z, tr.x, tr.y, tr.z, bl.x, bl.y, bl.z,
            center.x, center.y, center.z, br.x, br.y, br.z
        ))
        self.source:save(self.sourceArtifactPath)
        Log.Info("[UpscaleProbe] source artifact saved to %s", self.sourceArtifactPath)
    end
    if self.frames >= self.holdFrames then
        self:quit()
    end
end

function RenderingUpscale:onRender()
    local pass = Renderer:beginPass(self.passDesc)
    pass:setPipeline(self.gradientPipeline)
    pass:drawFullscreen()
    pass:finish()

    self.presentDesc:backbuffer(self.resX, self.resY, LoadOp.Load, 0, 0, 0, 1)
    pass = Renderer:beginPass(self.presentDesc)
    pass:setPipeline(self.blitPipeline)
    pass:setInputs(self.source:view(), Samplers.LinearClamp)
    pass:drawFullscreen()
    pass:finish()
end

function RenderingUpscale:onExit()
    self.source = nil
    self.gradient = nil
    self.blit = nil
    Log.Info("[UpscaleProbe] source and composite resources released")
end

return RenderingUpscale
