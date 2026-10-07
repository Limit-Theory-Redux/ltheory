local Application = require("States.Application")
local ProbePaths = require("States.App.Rendering.ProbePaths")

---@class RenderingIndexedMaterial: Application
local RenderingIndexedMaterial = Subclass("RenderingIndexedMaterial", Application)

local TARGET_W = 128
local TARGET_H = 128

function RenderingIndexedMaterial:onInit()
    self.target = Tex2D.Create(TARGET_W, TARGET_H, TexFormat.RGBA8)
    self.passDesc = RenderPassDesc.Create("IndexedMaterial")
    self.passDesc:color(0, self.target:view(), LoadOp.Clear, 0.0, 0.0, 0.0, 1.0)
    self.mesh = Mesh.Box(2)
    self.shader = Cache.Shader("indexed_material", "indexed_material")
    self.Params = self.shader:blockType("Params")
    self.pipeline = Pipeline.Get(PipelineDesc.Create(self.shader))
    self.backend = os.getenv("LTHEORY_WGPU") and "wgpu" or "opengl"
    self.frames = 0
    self.probed = false
end

function RenderingIndexedMaterial:eventLoop()
    Application.eventLoop(self)
    self.frames = self.frames + 1
    if self.frames >= 3 then
        self:quit()
    end
end

function RenderingIndexedMaterial:onRender()
    if self.probed then return end

    local pass = Renderer:beginPass(self.passDesc)
    pass:setPipeline(self.pipeline)
    local p = pass:alloc(self.Params)
    p.color.x, p.color.y, p.color.z = 0.25, 0.75, 0.125
    pass:drawMesh(self.mesh)
    pass:finish()

    local tl = self.target:sample(0, 0)
    local center = self.target:sample(math.floor(TARGET_W / 2), math.floor(TARGET_H / 2))
    local br = self.target:sample(TARGET_W - 1, TARGET_H - 1)
    local path = ProbePaths.file("indexed-material-" .. self.backend .. ".png")
    self.target:save(path)
    Log.Info(string.format(
        "[IndexedMaterialProbe] backend=%s target=%dx%d samples tl=(%.3f,%.3f,%.3f) center=(%.3f,%.3f,%.3f) br=(%.3f,%.3f,%.3f) artifact=%s",
        self.backend, TARGET_W, TARGET_H,
        tl.x, tl.y, tl.z,
        center.x, center.y, center.z,
        br.x, br.y, br.z,
        path
    ))
    self.probed = true
end

function RenderingIndexedMaterial:onExit()
    self.target = nil
    self.mesh = nil
    self.shader = nil
    Log.Info("[IndexedMaterialProbe] resources released")
end

return RenderingIndexedMaterial
