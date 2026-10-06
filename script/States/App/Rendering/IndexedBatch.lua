local ffi = require("ffi")
local Application = require("States.Application")
local ProbePaths = require("States.App.Rendering.ProbePaths")

---@class RenderingIndexedBatch: Application
local RenderingIndexedBatch = Subclass("RenderingIndexedBatch", Application)

local TARGET_W = 192
local TARGET_H = 128

function RenderingIndexedBatch:onInit()
    self.target = Tex2D.Create(TARGET_W, TARGET_H, TexFormat.RGBA8)
    self.mesh = Mesh.Box(2)
    self.indices = ffi.new("uint32_t[3]", {0, 1, 2})
    self.shader = Cache.Shader("indexed_batch", "indexed_batch")
    self.backend = os.getenv("LTHEORY_WGPU") and "wgpu" or "opengl"
    self.frames = 0
    self.probed = false
end

function RenderingIndexedBatch:eventLoop()
    Application.eventLoop(self)
    self.frames = self.frames + 1
    if self.frames >= 3 then
        self:quit()
    end
end

function RenderingIndexedBatch:onRender()
    if self.probed then return end

    Viewport.Push(0, 0, TARGET_W, TARGET_H, true)
    RenderState.PushAllDefaults()
    RenderState.PushBlendMode(BlendMode.Disabled)
    RenderState.PushCullFace(CullFace.None)
    RenderState.PushDepthTest(false)
    RenderState.PushDepthWritable(false)

    self.target:push()
    Draw.Clear(0.0, 0.0, 0.0, 1.0)
    self.shader:start()
    self.mesh:drawInstancedIndices(self.indices, 3)
    self.shader:stop()
    self.target:pop()

    RenderState.PopDepthWritable()
    RenderState.PopDepthTest()
    RenderState.PopCullFace()
    RenderState.PopBlendMode()
    RenderState.PopAll()
    Viewport.Pop()

    local left = self.target:sample(32, math.floor(TARGET_H / 2))
    local center = self.target:sample(math.floor(TARGET_W / 2), math.floor(TARGET_H / 2))
    local right = self.target:sample(160, math.floor(TARGET_H / 2))
    local path = ProbePaths.file("indexed-batch-" .. self.backend .. ".png")
    self.target:save(path)
    Log.Info(string.format(
        "[IndexedBatchProbe] backend=%s target=%dx%d samples left=(%.3f,%.3f,%.3f) center=(%.3f,%.3f,%.3f) right=(%.3f,%.3f,%.3f) artifact=%s",
        self.backend, TARGET_W, TARGET_H,
        left.x, left.y, left.z,
        center.x, center.y, center.z,
        right.x, right.y, right.z,
        path
    ))
    self.probed = true
end

function RenderingIndexedBatch:onExit()
    self.target = nil
    self.mesh = nil
    self.indices = nil
    self.shader = nil
    Log.Info("[IndexedBatchProbe] resources released")
end

return RenderingIndexedBatch
