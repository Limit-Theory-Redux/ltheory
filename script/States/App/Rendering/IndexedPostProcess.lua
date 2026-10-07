local Application = require("States.Application")
local ProbePaths = require("States.App.Rendering.ProbePaths")
local ProbeRead = require("States.App.Rendering.ProbeRead")

---@class RenderingIndexedPostProcess: Application
local RenderingIndexedPostProcess = Subclass("RenderingIndexedPostProcess", Application)

local TARGET_W = 128
local TARGET_H = 128
local SOURCE_COLOR = {0.125, 0.25, 0.75}

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

function RenderingIndexedPostProcess:onInit()
    self.source = Tex2D.Create(TARGET_W, TARGET_H, TexFormat.RGBA8)
    self.post = Tex2D.Create(TARGET_W, TARGET_H, TexFormat.RGBA8)
    self.sourceDesc = RenderPassDesc.Create("IndexedPostProcess.source")
    self.sourceDesc:color(0, self.source:view(), LoadOp.Clear, 0.0, 0.0, 0.0, 1.0)
    self.postDesc = RenderPassDesc.Create("IndexedPostProcess.post")
    self.postDesc:color(0, self.post:view(), LoadOp.Clear, 0.0, 0.0, 0.0, 1.0)
    self.presentDesc = RenderPassDesc.Create("IndexedPostProcess.present")
    self.mesh = makeQuad()
    self.material = Cache.Shader("indexed_material", "indexed_material")
    self.postShader = Cache.Shader("indexed_postprocess", "indexed_postprocess")
    self.blit = Cache.Shader("fullscreen_flip", "blit")
    self.Params = self.material:blockType("Params")
    self.materialPipeline = Pipeline.Get(PipelineDesc.Create(self.material))
    self.postPipeline = Pipeline.Get(PipelineDesc.Create(self.postShader))
    local blitDesc = PipelineDesc.Create(self.blit)
    blitDesc:vertex(VertexLayout.Fullscreen)
    self.blitPipeline = Pipeline.Get(blitDesc)
    self.backend = os.getenv("LTHEORY_WGPU") and "wgpu" or "opengl"
    self.frames = 0
    self.probed = false
end

function RenderingIndexedPostProcess:eventLoop()
    Application.eventLoop(self)
    self.frames = self.frames + 1
    if self.frames >= 3 then
        self:quit()
    end
end

function RenderingIndexedPostProcess:onRender()
    if self.probed then return end

    local pass = Renderer:beginPass(self.sourceDesc)
    pass:setPipeline(self.materialPipeline)
    local p = pass:alloc(self.Params)
    p.color.x, p.color.y, p.color.z = SOURCE_COLOR[1], SOURCE_COLOR[2], SOURCE_COLOR[3]
    pass:drawMesh(self.mesh)
    pass:finish()

    pass = Renderer:beginPass(self.postDesc)
    pass:setPipeline(self.postPipeline)
    pass:setInputs(self.source:view(), Samplers.LinearClamp)
    pass:drawMesh(self.mesh)
    pass:finish()

    self.presentDesc:backbuffer(self.resX, self.resY, LoadOp.Load, 0.0, 0.0, 0.0, 1.0)
    pass = Renderer:beginPass(self.presentDesc)
    pass:setPipeline(self.blitPipeline)
    pass:setInputs(self.post:view(), Samplers.LinearClamp)
    pass:drawFullscreen()
    pass:finish()

    local source = ProbeRead.sample(self.source, math.floor(TARGET_W / 2), math.floor(TARGET_H / 2))
    local post = ProbeRead.sample(self.post, math.floor(TARGET_W / 2), math.floor(TARGET_H / 2))
    local sourcePath = ProbePaths.file("indexed-postprocess-source-" .. self.backend .. ".png")
    local postPath = ProbePaths.file("indexed-postprocess-" .. self.backend .. ".png")
    self.source:save(sourcePath)
    self.post:save(postPath)
    Log.Info(string.format(
        "[IndexedPostProcessProbe] backend=%s target=%dx%d source=(%.3f,%.3f,%.3f) post=(%.3f,%.3f,%.3f) artifacts=%s,%s",
        self.backend, TARGET_W, TARGET_H,
        source.x, source.y, source.z,
        post.x, post.y, post.z,
        sourcePath, postPath
    ))
    self.probed = true
end

function RenderingIndexedPostProcess:onExit()
    self.source = nil
    self.post = nil
    self.mesh = nil
    self.material = nil
    self.postShader = nil
    self.blit = nil
    Log.Info("[IndexedPostProcessProbe] resources released")
end

return RenderingIndexedPostProcess
