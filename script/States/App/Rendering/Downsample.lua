local Application = require("States.Application")
local ProbePaths = require("States.App.Rendering.ProbePaths")

---@class RenderingDownsample: Application
local RenderingDownsample = Subclass("RenderingDownsample", Application)

local OUTPUT_W = 1280
local OUTPUT_H = 720
local SOURCE_W = OUTPUT_W
local SOURCE_H = OUTPUT_H
local TARGET_W = 320
local TARGET_H = 180

local function configureTexture(texture, filter)
    texture:setMagFilter(filter)
    texture:setMinFilter(filter)
    texture:setWrapMode(TexWrapMode.Clamp)
end

function RenderingDownsample:onInit()
    self.source = Tex2D.Create(SOURCE_W, SOURCE_H, TexFormat.RGBA8)
    self.linear = Tex2D.Create(TARGET_W, TARGET_H, TexFormat.RGBA8)
    self.nearest = Tex2D.Create(TARGET_W, TARGET_H, TexFormat.RGBA8)
    self.upscaled = Tex2D.Create(OUTPUT_W, OUTPUT_H, TexFormat.RGBA8)

    configureTexture(self.source, TexFilter.Linear)
    configureTexture(self.linear, TexFilter.Linear)
    configureTexture(self.nearest, TexFilter.Point)
    configureTexture(self.upscaled, TexFilter.Linear)

    self.backend = os.getenv("LTHEORY_WGPU") and "wgpu" or "opengl"
    self.pattern = Cache.Shader("ui", "downsample_pattern")
    self.downsample = Cache.Shader("ui", self.backend == "wgpu" and "downsample_filter_wgpu" or "downsample_filter")
    self.identity = Cache.Shader("ui", "filter/identity")
    self.frames = 0
    self.rendered = false
    self.probed = false
    self.holdFrames = tonumber(os.getenv("DOWNSAMPLE_HOLD_FRAMES")) or 40
end

function RenderingDownsample:eventLoop()
    Application.eventLoop(self)
    self.frames = self.frames + 1

    if self.frames == 10 and not self.probed then
        local sourceCenter = self.source:sample(math.floor(SOURCE_W / 2), math.floor(SOURCE_H / 2))
        local linearCenter = self.linear:sample(math.floor(TARGET_W / 2), math.floor(TARGET_H / 2))
        local nearestCenter = self.nearest:sample(math.floor(TARGET_W / 2), math.floor(TARGET_H / 2))
        local upscaledCenter = self.upscaled:sample(math.floor(OUTPUT_W / 2), math.floor(OUTPUT_H / 2))
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

    RenderState.PushAllDefaults()
    RenderState.PushDepthTest(false)
    RenderState.PushDepthWritable(false)
    local logicalW, logicalH = self.resX, self.resY
    local function drawExtent(width, height)
        if os.getenv("LTHEORY_WGPU") then
            return logicalW, logicalH
        end
        return width, height
    end

    self.source:push()
    Viewport.Push(0, 0, SOURCE_W, SOURCE_H, false)
    Draw.Clear(0, 0, 0, 1)
    self.pattern:start()
    local sourceDrawW, sourceDrawH = drawExtent(SOURCE_W, SOURCE_H)
    Draw.Rect(0, 0, sourceDrawW, sourceDrawH)
    self.pattern:stop()
    Viewport.Pop()
    self.source:pop()

    configureTexture(self.source, TexFilter.Linear)
    self.linear:push()
    Viewport.Push(0, 0, TARGET_W, TARGET_H, false)
    Draw.Clear(0, 0, 0, 1)
    self.downsample:start()
    self.downsample:setTex2D("src", self.source)
    local linearDrawW, linearDrawH = drawExtent(TARGET_W, TARGET_H)
    Draw.Rect(0, 0, linearDrawW, linearDrawH)
    self.downsample:stop()
    Viewport.Pop()
    self.linear:pop()

    configureTexture(self.source, TexFilter.Point)
    self.nearest:push()
    Viewport.Push(0, 0, TARGET_W, TARGET_H, false)
    Draw.Clear(0, 0, 0, 1)
    self.downsample:start()
    self.downsample:setTex2D("src", self.source)
    local nearestDrawW, nearestDrawH = drawExtent(TARGET_W, TARGET_H)
    Draw.Rect(0, 0, nearestDrawW, nearestDrawH)
    self.downsample:stop()
    Viewport.Pop()
    self.nearest:pop()

    configureTexture(self.linear, TexFilter.Linear)
    self.upscaled:push()
    Viewport.Push(0, 0, OUTPUT_W, OUTPUT_H, false)
    Draw.Clear(0, 0, 0, 1)
    self.identity:start()
    self.identity:setTex2D("src", self.linear)
    local upscaleDrawW, upscaleDrawH = drawExtent(OUTPUT_W, OUTPUT_H)
    Draw.Rect(0, 0, upscaleDrawW, upscaleDrawH)
    self.identity:stop()
    Viewport.Pop()
    self.upscaled:pop()

    -- The WGPU surface path owns a Depth24Plus attachment. Keep the
    -- renderer-owned color targets depth-free, then match the surface
    -- attachment for this final presentation pass.
    RenderState.PopDepthWritable()
    RenderState.PopDepthTest()
    RenderState.PushDepthTest(true)
    RenderState.PushDepthWritable(true)

    -- Keep the presentation path explicit and separate from renderer-owned
    -- readback: the target-local texture is flipped into window coordinates.
    Viewport.Push(0, 0, self.resX, self.resY, true)
    self.identity:start()
    self.identity:setTex2D("src", self.linear)
    Draw.Rect(0, self.resY, self.resX, -self.resY)
    self.identity:stop()
    Viewport.Pop()

    RenderState.PopDepthWritable()
    RenderState.PopDepthTest()
    RenderState.PopAll()
    self.rendered = true
end

function RenderingDownsample:onExit()
    self.source = nil
    self.linear = nil
    self.nearest = nil
    self.upscaled = nil
    self.pattern = nil
    self.identity = nil
    self.downsample = nil
    Log.Info("[DownsampleProbe] resources released")
end

return RenderingDownsample
