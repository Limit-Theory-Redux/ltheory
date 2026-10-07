local Application = require("States.Application")
local ProbePaths = require("States.App.Rendering.ProbePaths")

---@class RenderingUiComposite: Application
local RenderingUiComposite = Subclass("RenderingUiComposite", Application)

local TARGET_W = 128
local TARGET_H = 128
local OVERLAY = {x = 32, y = 32, w = 64, h = 64}

function RenderingUiComposite:onInit()
    self.target = Tex2D.Create(TARGET_W, TARGET_H, TexFormat.RGBA8)
    self.passDesc = RenderPassDesc.Create("UiComposite")
    self.passDesc:color(0, self.target:view(), LoadOp.Clear, 0.0, 0.0, 0.0, 1.0)
    self.presentDesc = RenderPassDesc.Create("UiComposite.present")
    self.shader = Cache.Shader("fullscreen", "color_block")
    self.Params = self.shader:blockType("Params")
    local opaque = PipelineDesc.Create(self.shader)
    opaque:vertex(VertexLayout.Fullscreen)
    self.opaquePipeline = Pipeline.Get(opaque)
    local alpha = PipelineDesc.Create(self.shader)
    alpha:vertex(VertexLayout.Fullscreen)
    alpha:blend(BlendMode.Alpha)
    self.alphaPipeline = Pipeline.Get(alpha)
    self.blit = Cache.Shader("fullscreen_flip", "blit")
    local blit = PipelineDesc.Create(self.blit)
    blit:vertex(VertexLayout.Fullscreen)
    self.blitPipeline = Pipeline.Get(blit)
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

    local pass = Renderer:beginPass(self.passDesc)

    pass:setPipeline(self.opaquePipeline)
    local p = pass:alloc(self.Params)
    p.color.x, p.color.y, p.color.z, p.color.w = 0.0, 0.0, 1.0, 1.0
    pass:drawFullscreen()

    -- The overlay quad is a sub-viewport of the pass: the UI projection
    -- follows the viewport, so the same fullscreen draw covers the rect.
    pass:setViewport(OVERLAY.x, OVERLAY.y, OVERLAY.w, OVERLAY.h)
    pass:setPipeline(self.alphaPipeline)
    p = pass:alloc(self.Params)
    p.color.x, p.color.y, p.color.z, p.color.w = 1.0, 0.0, 0.0, 0.5
    pass:drawFullscreen()

    pass:finish()

    -- Present the renderer-owned composite through the normal surface path.
    self.presentDesc:backbuffer(self.resX, self.resY, LoadOp.Load, 0.0, 0.0, 0.0, 1.0)
    pass = Renderer:beginPass(self.presentDesc)
    pass:setPipeline(self.blitPipeline)
    pass:setInputs(self.target:view(), Samplers.LinearClamp)
    pass:drawFullscreen()
    pass:finish()

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
    self.blit = nil
    Log.Info("[UiCompositeProbe] resources released")
end

return RenderingUiComposite
