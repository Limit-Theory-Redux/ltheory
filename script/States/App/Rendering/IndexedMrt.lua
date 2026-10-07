local Application = require("States.Application")
local ProbePaths = require("States.App.Rendering.ProbePaths")
local ProbeRead = require("States.App.Rendering.ProbeRead")

---@class RenderingIndexedMrt: Application
local RenderingIndexedMrt = Subclass("RenderingIndexedMrt", Application)

local TARGET_W = 128
local TARGET_H = 128

function RenderingIndexedMrt:onInit()
    self.buffer0 = Tex2D.Create(TARGET_W, TARGET_H, TexFormat.RGBA16F)
    self.buffer1 = Tex2D.Create(TARGET_W, TARGET_H, TexFormat.RGBA16F)
    self.zBufferL = Tex2D.Create(TARGET_W, TARGET_H, TexFormat.R32F)
    self.zBuffer = Tex2D.Create(TARGET_W, TARGET_H, TexFormat.Depth32F)
    self.passDesc = RenderPassDesc.Create("IndexedMrt")
    self.passDesc:color(0, self.buffer0:view(), LoadOp.Clear, 0.0, 0.0, 0.0, 0.0)
    self.passDesc:color(1, self.buffer1:view(), LoadOp.Clear, 0.0, 0.0, 0.0, 0.0)
    self.passDesc:color(2, self.zBufferL:view(), LoadOp.Clear, 0.0, 0.0, 0.0, 0.0)
    self.passDesc:depth(self.zBuffer:view(), LoadOp.Clear, 1.0)
    self.mesh = Mesh.Box(2)
    self.shader = Cache.Shader("indexed_mrt", "indexed_mrt")
    local desc = PipelineDesc.Create(self.shader)
    desc:depth(true, true, CompareFn.LessEqual)
    desc:colorFormat(0, TexFormat.RGBA16F)
    desc:colorFormat(1, TexFormat.RGBA16F)
    desc:colorFormat(2, TexFormat.R32F)
    desc:depthFormat(TexFormat.Depth32F)
    self.pipeline = Pipeline.Get(desc)
    self.backend = os.getenv("LTHEORY_WGPU") and "wgpu" or "opengl"
    self.frames = 0
    self.probed = false
end

function RenderingIndexedMrt:eventLoop()
    Application.eventLoop(self)
    self.frames = self.frames + 1
    if self.frames >= 3 then
        self:quit()
    end
end

function RenderingIndexedMrt:onRender()
    if self.probed then return end

    local pass = Renderer:beginPass(self.passDesc)
    pass:setPipeline(self.pipeline)
    pass:drawMesh(self.mesh)
    pass:finish()

    local p0 = ProbeRead.sample(self.buffer0, math.floor(TARGET_W / 2), math.floor(TARGET_H / 2))
    local p1 = ProbeRead.sample(self.buffer1, math.floor(TARGET_W / 2), math.floor(TARGET_H / 2))
    local pz = ProbeRead.sample(self.zBufferL, math.floor(TARGET_W / 2), math.floor(TARGET_H / 2))
    local stem = "indexed-mrt-" .. self.backend
    local path0 = ProbePaths.file(stem .. "-buffer0.png")
    local path1 = ProbePaths.file(stem .. "-buffer1.png")
    local pathz = ProbePaths.file(stem .. "-zbufferl.png")
    self.buffer0:save(path0)
    self.buffer1:save(path1)
    self.zBufferL:save(pathz)
    Log.Info(string.format(
        "[IndexedMrtProbe] backend=%s target=%dx%d buffer0=(%.3f,%.3f,%.3f) buffer1=(%.3f,%.3f,%.3f) zBufferL=(%.3f,%.3f,%.3f) artifacts=%s,%s,%s",
        self.backend, TARGET_W, TARGET_H,
        p0.x, p0.y, p0.z,
        p1.x, p1.y, p1.z,
        pz.x, pz.y, pz.z,
        path0, path1, pathz
    ))
    self.probed = true
end

function RenderingIndexedMrt:onExit()
    self.buffer0 = nil
    self.buffer1 = nil
    self.zBufferL = nil
    self.zBuffer = nil
    self.mesh = nil
    self.shader = nil
    Log.Info("[IndexedMrtProbe] resources released")
end

return RenderingIndexedMrt
