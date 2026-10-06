local Application = require("States.Application")
local ProbePaths = require("States.App.Rendering.ProbePaths")

---@class RenderingUiComposite: Application
local RenderingUiComposite = Subclass("RenderingUiComposite", Application)

local TARGET_W = 128
local TARGET_H = 128
local OVERLAY = {x = 32, y = 32, w = 64, h = 64}

function RenderingUiComposite:onInit()
    self.target = Tex2D.Create(TARGET_W, TARGET_H, TexFormat.RGBA8)
    self.target:setMinFilter(TexFilter.Linear)
    self.target:setMagFilter(TexFilter.Linear)
    self.target:setWrapMode(TexWrapMode.Clamp)
    self.shader = Cache.Shader("ui", "simple_color")
    self.identity = Cache.Shader("ui", "filter/identity")
    self.backend = os.getenv("LTHEORY_WGPU") and "wgpu" or "opengl"
    self.frames = 0
    self.probed = false
end

function RenderingUiComposite:eventLoop()
    Application.eventLoop(self)
    self.frames = self.frames + 1
    if self.frames >= 3 then
        self:quit()
    end
end

function RenderingUiComposite:onRender()
    if self.probed then return end

    Viewport.Push(0, 0, TARGET_W, TARGET_H, true)
    RenderState.PushAllDefaults()
    RenderState.PushCullFace(CullFace.None)
    RenderState.PushDepthTest(false)
    RenderState.PushDepthWritable(false)
    RenderState.PushBlendMode(BlendMode.Disabled)

    RenderTarget.Push(TARGET_W, TARGET_H)
    RenderTarget.BindTex2D(self.target)
    Draw.Clear(0.0, 0.0, 0.0, 1.0)

    local logicalW, logicalH = self.resX, self.resY
    local drawW, drawH = logicalW, logicalH
    if self.backend == "opengl" then
        drawW, drawH = TARGET_W, TARGET_H
    end

    self.shader:start()
    self.shader:setFloat4("color", 0.0, 0.0, 1.0, 1.0)
    Draw.Rect(0, 0, drawW, drawH)

    RenderState.PushBlendMode(BlendMode.Alpha)
    self.shader:setFloat4("color", 1.0, 0.0, 0.0, 0.5)
    Draw.Rect(
        OVERLAY.x,
        OVERLAY.y,
        OVERLAY.w,
        OVERLAY.h
    )
    RenderState.PopBlendMode()
    self.shader:stop()

    RenderTarget.Pop()
    Viewport.Pop()

    -- Present the renderer-owned composite through the normal surface path.
    RenderState.PopDepthWritable()
    RenderState.PopDepthTest()
    RenderState.PushDepthTest(true)
    RenderState.PushDepthWritable(true)
    Viewport.Push(0, 0, self.resX, self.resY, true)
    self.identity:start()
    self.identity:setTex2D("src", self.target)
    Draw.Rect(0, self.resY, self.resX, -self.resY)
    self.identity:stop()
    Viewport.Pop()
    RenderState.PopDepthWritable()
    RenderState.PopDepthTest()

    RenderState.PopBlendMode()
    RenderState.PopCullFace()
    RenderState.PopAll()

    local outside = self.target:sample(16, 64)
    local inside = self.target:sample(64, 64)
    local path = ProbePaths.file("ui-composite-" .. self.backend .. ".png")
    self.target:save(path)
    Log.Info(string.format(
        "[UiCompositeProbe] backend=%s target=%dx%d overlay=(%d,%d,%d,%d) outside=(%.3f,%.3f,%.3f) inside=(%.3f,%.3f,%.3f) artifact=%s",
        self.backend, TARGET_W, TARGET_H,
        OVERLAY.x, OVERLAY.y, OVERLAY.w, OVERLAY.h,
        outside.x, outside.y, outside.z,
        inside.x, inside.y, inside.z,
        path
    ))
    self.probed = true
end

function RenderingUiComposite:onExit()
    self.target = nil
    self.shader = nil
    self.identity = nil
    Log.Info("[UiCompositeProbe] resources released")
end

return RenderingUiComposite
