local Application = require("States.Application")
local ProbePaths = require("States.App.Rendering.ProbePaths")

---@class RenderingUpscale: Application
local RenderingUpscale = Subclass("RenderingUpscale", Application)

local SOURCE_W = 320
local SOURCE_H = 180

function RenderingUpscale:onInit()
    self.source = Tex2D.Create(SOURCE_W, SOURCE_H, TexFormat.RGBA8)
    self.source:setMagFilter(TexFilter.Linear)
    self.source:setMinFilter(TexFilter.Linear)
    self.source:setWrapMode(TexWrapMode.Clamp)
    self.gradient = Cache.Shader("ui", "gradient")
    self.identity = Cache.Shader("ui", "filter/identity")
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
        local tl = self.source:sample(0, 0)
        local tr = self.source:sample(SOURCE_W - 1, 0)
        local bl = self.source:sample(0, SOURCE_H - 1)
        local center = self.source:sample(math.floor(SOURCE_W / 2), math.floor(SOURCE_H / 2))
        local br = self.source:sample(SOURCE_W - 1, SOURCE_H - 1)
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
    RenderState.PushAllDefaults()

    self.source:push()
    Viewport.Push(0, 0, SOURCE_W, SOURCE_H, false)
    Draw.Clear(0, 0, 0, 1)
    self.gradient:start()
    -- The legacy GL target push uses target-local UI coordinates. The current
    -- wgpu immediate path preserves the logical compositor extent for this
    -- diagnostic, so choose the geometry that covers the active projection
    -- instead of changing renderer-wide matrix semantics mid-rung.
    local sourceDrawW, sourceDrawH = SOURCE_W, SOURCE_H
    if os.getenv("LTHEORY_WGPU") then
        sourceDrawW, sourceDrawH = self.resX, self.resY
    end
    Draw.Rect(0, 0, sourceDrawW, sourceDrawH)
    self.gradient:stop()
    Viewport.Pop()
    self.source:pop()

    Viewport.Push(0, 0, self.resX, self.resY, true)
    self.identity:start()
    self.identity:setTex2D("src", self.source)
    Draw.Rect(0, self.resY, self.resX, -self.resY)
    self.identity:stop()
    Viewport.Pop()

    RenderState.PopAll()
end

function RenderingUpscale:onExit()
    self.source = nil
    self.gradient = nil
    self.identity = nil
    Log.Info("[UpscaleProbe] source and composite resources released")
end

return RenderingUpscale
