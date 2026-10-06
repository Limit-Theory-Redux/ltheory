local Application = require("States.Application")
local ProbePaths = require("States.App.Rendering.ProbePaths")

---@class RenderingIndexedCull: Application
local RenderingIndexedCull = Subclass("RenderingIndexedCull", Application)

local TARGET_W = 128
local TARGET_H = 128

local function triangle(reverse)
    local mesh = Mesh.Create()
    mesh:addVertex(-1.0, -1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0)
    mesh:addVertex(1.0, -1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0)
    mesh:addVertex(0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.5, 1.0)
    if reverse then
        mesh:addTri(0, 2, 1)
    else
        mesh:addTri(0, 1, 2)
    end
    return mesh
end

---@param self RenderingIndexedCull
---@param mode CullFace
---@param name string
local function drawPair(self, mode, name)
    RenderState.PushCullFace(mode)
    RenderTarget.Push(TARGET_W, TARGET_H)
    RenderTarget.BindTex2D(self.target)
    Draw.Clear(0.0, 0.0, 0.0, 1.0)

    self.shader:start()
    self.shader:setFloat4("color", 0.0, 1.0, 0.0, 1.0)
    self.front:draw()
    self.shader:setFloat4("color", 1.0, 0.125, 0.0, 1.0)
    self.back:draw()
    self.shader:stop()

    RenderTarget.Pop()
    RenderState.PopCullFace()
    self.target:save(ProbePaths.file("indexed-cull-" .. self.backend .. "-" .. name .. ".png"))
    return self.target:sample(math.floor(TARGET_W / 2), math.floor(TARGET_H / 2))
end

function RenderingIndexedCull:onInit()
    self.target = Tex2D.Create(TARGET_W, TARGET_H, TexFormat.RGBA8)
    self.front = triangle(false)
    self.back = triangle(true)
    self.shader = Cache.Shader("indexed_cull", "indexed_cull")
    self.backend = os.getenv("LTHEORY_WGPU") and "wgpu" or "opengl"
    self.frames = 0
    self.probed = false
end

function RenderingIndexedCull:eventLoop()
    Application.eventLoop(self)
    self.frames = self.frames + 1
    if self.frames >= 3 then
        self:quit()
    end
end

function RenderingIndexedCull:onRender()
    if self.probed then return end

    Viewport.Push(0, 0, TARGET_W, TARGET_H, true)
    RenderState.PushAllDefaults()
    RenderState.PushBlendMode(BlendMode.Disabled)
    RenderState.PushDepthTest(false)
    RenderState.PushDepthWritable(false)

    local backCull = drawPair(self, CullFace.Back, "back")
    local frontCull = drawPair(self, CullFace.Front, "front")
    local noCull = drawPair(self, CullFace.None, "none")

    RenderState.PopDepthWritable()
    RenderState.PopDepthTest()
    RenderState.PopBlendMode()
    RenderState.PopAll()
    Viewport.Pop()

    Log.Info(string.format(
        "[IndexedCullProbe] backend=%s target=%dx%d samples back=(%.3f,%.3f,%.3f) front=(%.3f,%.3f,%.3f) none=(%.3f,%.3f,%.3f)",
        self.backend, TARGET_W, TARGET_H,
        backCull.x, backCull.y, backCull.z,
        frontCull.x, frontCull.y, frontCull.z,
        noCull.x, noCull.y, noCull.z
    ))
    self.probed = true
end

function RenderingIndexedCull:onExit()
    self.target = nil
    self.front = nil
    self.back = nil
    self.shader = nil
    Log.Info("[IndexedCullProbe] resources released")
end

return RenderingIndexedCull
