local Application = require("States.Application")
local ProbePaths = require("States.App.Rendering.ProbePaths")
local ProbeRead = require("States.App.Rendering.ProbeRead")

---@class RenderingIndexedDepth: Application
local RenderingIndexedDepth = Subclass("RenderingIndexedDepth", Application)

local TARGET_W = 128
local TARGET_H = 128

function RenderingIndexedDepth:onInit()
    self.color = Tex2D.Create(TARGET_W, TARGET_H, TexFormat.RGBA8)
    self.depth = Tex2D.Create(TARGET_W, TARGET_H, TexFormat.Depth32F)
    self.passDesc = RenderPassDesc.Create("IndexedDepth")
    self.passDesc:color(0, self.color:view(), LoadOp.Clear, 0.0, 0.0, 0.0, 1.0)
    self.passDesc:depth(self.depth:view(), LoadOp.Clear, 1.0)
    self.mesh = Mesh.Box(2)
    self.shader = Cache.Shader("indexed_depth", "indexed_depth")
    self.Params = self.shader:blockType("Params")
    local desc = PipelineDesc.Create(self.shader)
    desc:depth(true, true, CompareFn.LessEqual)
    self.pipeline = Pipeline.Get(desc)
    self.backend = os.getenv("LTHEORY_WGPU") and "wgpu" or "opengl"
    self.frames = 0
    self.probed = false
end

function RenderingIndexedDepth:eventLoop()
    Application.eventLoop(self)
    self.frames = self.frames + 1
    if self.frames >= 3 then
        self:quit()
    end
end

function RenderingIndexedDepth:onRender()
    if self.probed then return end

    local pass = Renderer:beginPass(self.passDesc)

    pass:setPipeline(self.pipeline)
    -- Near orange is written first; farther blue must be rejected by depth.
    local p = pass:alloc(self.Params)
    p.depthValue = 0.25
    p.color.x, p.color.y, p.color.z, p.color.w = 1.0, 0.125, 0.0, 1.0
    pass:drawMesh(self.mesh)
    p = pass:alloc(self.Params)
    p.depthValue = 0.75
    p.color.x, p.color.y, p.color.z, p.color.w = 0.0, 0.125, 1.0, 1.0
    pass:drawMesh(self.mesh)

    pass:finish()

    local tl = ProbeRead.sample(self.color, 0, 0)
    local center = ProbeRead.sample(self.color, math.floor(TARGET_W / 2), math.floor(TARGET_H / 2))
    local br = ProbeRead.sample(self.color, TARGET_W - 1, TARGET_H - 1)
    local path = ProbePaths.file("indexed-depth-" .. self.backend .. ".png")
    self.color:save(path)
    Log.Info(string.format(
        "[IndexedDepthProbe] backend=%s target=%dx%d samples tl=(%.3f,%.3f,%.3f) center=(%.3f,%.3f,%.3f) br=(%.3f,%.3f,%.3f) artifact=%s",
        self.backend, TARGET_W, TARGET_H,
        tl.x, tl.y, tl.z,
        center.x, center.y, center.z,
        br.x, br.y, br.z,
        path
    ))
    self.probed = true
end

function RenderingIndexedDepth:onExit()
    self.color = nil
    self.depth = nil
    self.mesh = nil
    self.shader = nil
    Log.Info("[IndexedDepthProbe] resources released")
end

return RenderingIndexedDepth
