local Application = require("States.Application")
local ProbePaths = require("States.App.Rendering.ProbePaths")
local ProbeRead = require("States.App.Rendering.ProbeRead")

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
    local pass = Renderer:beginPass(self.passDesc)

    pass:setPipeline(self.pipelines[mode])
    local p = pass:alloc(self.Params)
    p.color.x, p.color.y, p.color.z, p.color.w = 0.0, 1.0, 0.0, 1.0
    pass:drawMesh(self.front)
    p = pass:alloc(self.Params)
    p.color.x, p.color.y, p.color.z, p.color.w = 1.0, 0.125, 0.0, 1.0
    pass:drawMesh(self.back)

    pass:finish()
    self.target:save(ProbePaths.file("indexed-cull-" .. self.backend .. "-" .. name .. ".png"))
    return ProbeRead.sample(self.target, math.floor(TARGET_W / 2), math.floor(TARGET_H / 2))
end

function RenderingIndexedCull:onInit()
    self.target = Tex2D.Create(TARGET_W, TARGET_H, TexFormat.RGBA8)
    self.passDesc = RenderPassDesc.Create("IndexedCull")
    self.passDesc:color(0, self.target:view(), LoadOp.Clear, 0.0, 0.0, 0.0, 1.0)
    self.front = triangle(false)
    self.back = triangle(true)
    self.shader = Cache.Shader("indexed_cull", "indexed_cull")
    self.Params = self.shader:blockType("Params")
    self.pipelines = {}
    for _, mode in ipairs({ CullFace.Back, CullFace.Front, CullFace.None }) do
        local desc = PipelineDesc.Create(self.shader)
        desc:cull(mode)
        self.pipelines[mode] = Pipeline.Get(desc)
    end
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

    local backCull = drawPair(self, CullFace.Back, "back")
    local frontCull = drawPair(self, CullFace.Front, "front")
    local noCull = drawPair(self, CullFace.None, "none")

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
