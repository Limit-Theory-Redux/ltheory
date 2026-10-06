local Application = require("States.Application")
local ProbePaths = require("States.App.Rendering.ProbePaths")

---@class RenderingIndexedClip: Application
local RenderingIndexedClip = Subclass("RenderingIndexedClip", Application)

local TARGET_W = 128
local TARGET_H = 128
local CLIP_X = 32
local CLIP_Y = 24
local CLIP_W = 64
local CLIP_H = 80

function RenderingIndexedClip:onInit()
    self.color = Tex2D.Create(TARGET_W, TARGET_H, TexFormat.RGBA8)
    self.mesh = Mesh.Box(2)
    self.shader = Cache.Shader("indexed_material", "indexed_material")
    self.backend = os.getenv("LTHEORY_WGPU") and "wgpu" or "opengl"
    self.frames = 0
    self.probed = false
end

function RenderingIndexedClip:eventLoop()
    Application.eventLoop(self)
    self.frames = self.frames + 1
    if self.frames >= 3 then
        self:quit()
    end
end

function RenderingIndexedClip:onRender()
    if self.probed then return end

    Viewport.Push(0, 0, TARGET_W, TARGET_H, true)
    RenderState.PushAllDefaults()
    RenderState.PushBlendMode(BlendMode.Disabled)
    RenderState.PushCullFace(CullFace.None)
    RenderState.PushDepthTest(false)
    RenderState.PushDepthWritable(false)

    RenderTarget.Push(TARGET_W, TARGET_H)
    RenderTarget.BindTex2D(self.color)
    Draw.Clear(0.0, 0.0, 0.0, 1.0)

    -- ClipRect converts the target-local rectangle to the backend scissor
    -- convention. The full primitive must be visible only inside this rect.
    ClipRect.Push(CLIP_X, CLIP_Y, CLIP_W, CLIP_H)
    self.shader:start()
    self.shader:setFloat3("color", 1.0, 0.125, 0.0)
    self.mesh:draw()
    self.shader:stop()
    ClipRect.Pop()

    RenderTarget.Pop()
    RenderState.PopDepthWritable()
    RenderState.PopDepthTest()
    RenderState.PopCullFace()
    RenderState.PopBlendMode()
    RenderState.PopAll()
    Viewport.Pop()

    local outside = self.color:sample(16, 64)
    local inside = self.color:sample(64, 64)
    local path = ProbePaths.file("indexed-clip-" .. self.backend .. ".png")
    self.color:save(path)
    Log.Info(string.format(
        "[IndexedClipProbe] backend=%s target=%dx%d clip=(%d,%d,%d,%d) outside=(%.3f,%.3f,%.3f) inside=(%.3f,%.3f,%.3f) artifact=%s",
        self.backend, TARGET_W, TARGET_H, CLIP_X, CLIP_Y, CLIP_W, CLIP_H,
        outside.x, outside.y, outside.z,
        inside.x, inside.y, inside.z,
        path
    ))
    self.probed = true
end

function RenderingIndexedClip:onExit()
    self.color = nil
    self.mesh = nil
    self.shader = nil
    Log.Info("[IndexedClipProbe] resources released")
end

return RenderingIndexedClip
