local Application = require("States.Application")
local ProbePaths = require("States.App.Rendering.ProbePaths")

---@class RenderingIndexedBlend: Application
local RenderingIndexedBlend = Subclass("RenderingIndexedBlend", Application)

local TARGET_W = 128
local TARGET_H = 128

local function makeQuad()
    local mesh = Mesh.Create()
    mesh:addVertex(-1.0, -1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0)
    mesh:addVertex(1.0, -1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0)
    mesh:addVertex(1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0)
    mesh:addVertex(-1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0)
    mesh:addTri(0, 1, 2)
    mesh:addTri(0, 2, 3)
    return mesh
end

function RenderingIndexedBlend:onInit()
    self.target = Tex2D.Create(TARGET_W, TARGET_H, TexFormat.RGBA8)
    self.mesh = makeQuad()
    self.shader = Cache.Shader("indexed_blend", "indexed_blend")
    self.backend = os.getenv("LTHEORY_WGPU") and "wgpu" or "opengl"
    self.frames = 0
    self.probed = false
end

function RenderingIndexedBlend:eventLoop()
    Application.eventLoop(self)
    self.frames = self.frames + 1
    if self.frames >= 3 then
        self:quit()
    end
end

function RenderingIndexedBlend:onRender()
    if self.probed then return end

    Viewport.Push(0, 0, TARGET_W, TARGET_H, true)
    RenderState.PushAllDefaults()
    RenderState.PushBlendMode(BlendMode.Alpha)
    RenderState.PushCullFace(CullFace.None)
    RenderState.PushDepthTest(false)
    RenderState.PushDepthWritable(false)

    self.target:push()
    Draw.Clear(0.0, 0.0, 0.0, 1.0)
    self.shader:start()
    self.shader:setFloat4("color", 0.0, 0.0, 1.0, 1.0)
    self.mesh:draw()
    self.shader:setFloat4("color", 1.0, 0.0, 0.0, 0.5)
    self.mesh:draw()
    self.shader:stop()
    self.target:pop()

    RenderState.PopDepthWritable()
    RenderState.PopDepthTest()
    RenderState.PopCullFace()
    RenderState.PopBlendMode()
    RenderState.PopAll()
    Viewport.Pop()

    local center = self.target:sample(math.floor(TARGET_W / 2), math.floor(TARGET_H / 2))
    local path = ProbePaths.file("indexed-blend-" .. self.backend .. ".png")
    self.target:save(path)
    Log.Info(string.format(
        "[IndexedBlendProbe] backend=%s target=%dx%d center=(%.3f,%.3f,%.3f) artifact=%s",
        self.backend, TARGET_W, TARGET_H,
        center.x, center.y, center.z,
        path
    ))
    self.probed = true
end

function RenderingIndexedBlend:onExit()
    self.target = nil
    self.mesh = nil
    self.shader = nil
    Log.Info("[IndexedBlendProbe] resources released")
end

return RenderingIndexedBlend
