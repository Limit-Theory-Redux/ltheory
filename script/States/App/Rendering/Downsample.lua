local Application = require("States.Application")
local ProbePaths = require("States.App.Rendering.ProbePaths")
local ProbeRead = require("States.App.Rendering.ProbeRead")

---@class RenderingDownsample: Application
local RenderingDownsample = Subclass("RenderingDownsample", Application)

local OUTPUT_W = 1280
local OUTPUT_H = 720
local SOURCE_W = OUTPUT_W
local SOURCE_H = OUTPUT_H
local TARGET_W = 320
local TARGET_H = 180

function RenderingDownsample:onInit()
    self.source = Tex2D.Create(SOURCE_W, SOURCE_H, TexFormat.RGBA8)
    self.linear = Tex2D.Create(TARGET_W, TARGET_H, TexFormat.RGBA8)
    self.nearest = Tex2D.Create(TARGET_W, TARGET_H, TexFormat.RGBA8)
    self.upscaled = Tex2D.Create(OUTPUT_W, OUTPUT_H, TexFormat.RGBA8)

    self.backend = os.getenv("LTHEORY_WGPU") and "wgpu" or "opengl"
    self.pattern = Cache.Shader("fullscreen", "downsample_pattern")
    self.downsample = Cache.Shader("fullscreen", "downsample_filter")
    self.blit = Cache.Shader("fullscreen_flip", "blit")
    local function pipelineFor(shader)
        local desc = PipelineDesc.Create(shader)
        desc:vertex(VertexLayout.Fullscreen)
        desc:colorFormat(0, TexFormat.RGBA8)
        return Pipeline.Get(desc)
    end
    self.patternPipeline = pipelineFor(self.pattern)
    self.downsamplePipeline = pipelineFor(self.downsample)
    self.blitPipeline = pipelineFor(self.blit)
    self.frames = 0
    self.rendered = false
    self.probed = false
    self.holdFrames = tonumber(os.getenv("DOWNSAMPLE_HOLD_FRAMES")) or 40

    self.passDescs = {}
    for _, name in ipairs({ "source", "linear", "nearest", "upscaled" }) do
        local desc = RenderPassDesc.Create("Downsample." .. name)
        desc:color(0, self[name]:view(), LoadOp.Clear, 0, 0, 0, 1)
        self.passDescs[name] = desc
    end
    self.presentDesc = RenderPassDesc.Create("Downsample.present")
end

function RenderingDownsample:eventLoop()
    Application.eventLoop(self)
    self.frames = self.frames + 1

    if self.frames == 10 and not self.probed then
        local sourceCenter = ProbeRead.sample(self.source, math.floor(SOURCE_W / 2), math.floor(SOURCE_H / 2))
        local linearCenter = ProbeRead.sample(self.linear, math.floor(TARGET_W / 2), math.floor(TARGET_H / 2))
        local nearestCenter = ProbeRead.sample(self.nearest, math.floor(TARGET_W / 2), math.floor(TARGET_H / 2))
        local upscaledCenter = ProbeRead.sample(self.upscaled, math.floor(OUTPUT_W / 2), math.floor(OUTPUT_H / 2))
        local sourcePath = ProbePaths.file("downsample-source.png")
        local linearPath = ProbePaths.file("downsample-linear.png")
        local nearestPath = ProbePaths.file("downsample-nearest.png")
        local upscaledPath = ProbePaths.file("downsample-upscaled.png")
        self.source:save(sourcePath)
        self.linear:save(linearPath)
        self.nearest:save(nearestPath)
        self.upscaled:save(upscaledPath)
        Log.Info(string.format(
            "[DownsampleProbe] backend=%s source=%dx%d downsample=%dx%d upscale=%dx%d sourceCenter=(%.3f,%.3f,%.3f) linearCenter=(%.3f,%.3f,%.3f) nearestCenter=(%.3f,%.3f,%.3f) upscaledCenter=(%.3f,%.3f,%.3f) artifacts=%s,%s,%s,%s",
            self.backend, SOURCE_W, SOURCE_H, TARGET_W, TARGET_H, OUTPUT_W, OUTPUT_H,
            sourceCenter.x, sourceCenter.y, sourceCenter.z,
            linearCenter.x, linearCenter.y, linearCenter.z,
            nearestCenter.x, nearestCenter.y, nearestCenter.z,
            upscaledCenter.x, upscaledCenter.y, upscaledCenter.z,
            sourcePath, linearPath, nearestPath, upscaledPath
        ))
        self.probed = true
    end

    if self.frames >= self.holdFrames then
        self:quit()
    end
end

function RenderingDownsample:onRender()
    if self.rendered then return end

    local pass = Renderer:beginPass(self.passDescs.source)
    pass:setPipeline(self.patternPipeline)
    pass:drawFullscreen()
    pass:finish()

    pass = Renderer:beginPass(self.passDescs.linear)
    pass:setPipeline(self.downsamplePipeline)
    pass:setInputs(self.source:view(), Samplers.LinearClamp)
    pass:drawFullscreen()
    pass:finish()

    pass = Renderer:beginPass(self.passDescs.nearest)
    pass:setPipeline(self.downsamplePipeline)
    pass:setInputs(self.source:view(), Samplers.Point)
    pass:drawFullscreen()
    pass:finish()

    pass = Renderer:beginPass(self.passDescs.upscaled)
    pass:setPipeline(self.blitPipeline)
    pass:setInputs(self.linear:view(), Samplers.LinearClamp)
    pass:drawFullscreen()
    pass:finish()

    -- Keep the presentation path explicit and separate from renderer-owned
    -- readback: the target-local texture is flipped into window coordinates.
    self.presentDesc:backbuffer(self.resX, self.resY, LoadOp.Load, 0, 0, 0, 1)
    pass = Renderer:beginPass(self.presentDesc)
    pass:setPipeline(self.blitPipeline)
    pass:setInputs(self.linear:view(), Samplers.LinearClamp)
    pass:drawFullscreen()
    pass:finish()

    self.rendered = true
end

function RenderingDownsample:onExit()
    self.source = nil
    self.linear = nil
    self.nearest = nil
    self.upscaled = nil
    self.pattern = nil
    self.blit = nil
    self.downsample = nil
    Log.Info("[DownsampleProbe] resources released")
end

return RenderingDownsample
