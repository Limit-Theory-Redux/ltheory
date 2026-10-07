local ffi = require("ffi")
local Application = require("States.Application")
local ProbePaths = require("States.App.Rendering.ProbePaths")
local ProbeRead = require("States.App.Rendering.ProbeRead")

---@class RenderingIndexedGeometry: Application
local RenderingIndexedGeometry = Subclass("RenderingIndexedGeometry", Application)

local TARGET_W = 128
local TARGET_H = 128

function RenderingIndexedGeometry:onInit()
    self.target = Tex2D.Create(TARGET_W, TARGET_H, TexFormat.RGBA8)
    self.passDesc = RenderPassDesc.Create("IndexedGeometry")
    self.passDesc:color(0, self.target:view(), LoadOp.Clear, 0.0, 0.0, 0.0, 1.0)
    self.mesh = Mesh.Box(2)
    self.stage = os.getenv("INDEXED_GEOMETRY_STAGE") or "baseline"
    local shaderName = self.stage == "canonical" and "indexed_canonical" or "indexed_baseline"
    self.shader = Cache.Shader(shaderName, "indexed_baseline")
    self.pipeline = Pipeline.Get(PipelineDesc.Create(self.shader))
    if self.stage == "canonical" then
        self.Params = self.shader:blockType("Params")
    end
    self.mProj = Matrix.Scaling(0.75, 0.75, 0.25)
    self.mView = Matrix.Identity()
    self.mWorld = Matrix.Identity()
    self.backend = os.getenv("LTHEORY_WGPU") and "wgpu" or "opengl"
    self.frames = 0
    self.probed = false
end

function RenderingIndexedGeometry:eventLoop()
    Application.eventLoop(self)
    self.frames = self.frames + 1
    if self.frames >= 3 then
        self:quit()
    end
end

function RenderingIndexedGeometry:onRender()
    if self.probed then return end

    local pass = Renderer:beginPass(self.passDesc)
    pass:setPipeline(self.pipeline)
    if self.stage == "canonical" then
        local p = pass:alloc(self.Params)
        ffi.copy(p.mProj, self.mProj, ffi.sizeof(p.mProj))
        ffi.copy(p.mView, self.mView, ffi.sizeof(p.mView))
        ffi.copy(p.mWorld, self.mWorld, ffi.sizeof(p.mWorld))
    end
    pass:drawMesh(self.mesh)
    pass:finish()

    local tl = ProbeRead.sample(self.target, 0, 0)
    local center = ProbeRead.sample(self.target, math.floor(TARGET_W / 2), math.floor(TARGET_H / 2))
    local br = ProbeRead.sample(self.target, TARGET_W - 1, TARGET_H - 1)
    local path = ProbePaths.file("indexed-geometry-" .. self.backend .. ".png")
    self.target:save(path)
    Log.Info(string.format(
        "[IndexedGeometryProbe] backend=%s target=%dx%d samples tl=(%.3f,%.3f,%.3f) center=(%.3f,%.3f,%.3f) br=(%.3f,%.3f,%.3f) artifact=%s",
        self.backend, TARGET_W, TARGET_H,
        tl.x, tl.y, tl.z,
        center.x, center.y, center.z,
        br.x, br.y, br.z,
        path
    ))
    self.probed = true
end

function RenderingIndexedGeometry:onExit()
    self.target = nil
    self.mesh = nil
    self.shader = nil
    Log.Info("[IndexedGeometryProbe] resources released")
end

return RenderingIndexedGeometry
