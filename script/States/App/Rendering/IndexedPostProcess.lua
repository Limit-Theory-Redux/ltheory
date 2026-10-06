local Application = require("States.Application")
local ProbePaths = require("States.App.Rendering.ProbePaths")

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
    self.source:setMinFilter(TexFilter.Linear)
    self.source:setMagFilter(TexFilter.Linear)
    self.source:setWrapMode(TexWrapMode.Clamp)
    self.post:setMinFilter(TexFilter.Linear)
    self.post:setMagFilter(TexFilter.Linear)
    self.post:setWrapMode(TexWrapMode.Clamp)
    self.mesh = makeQuad()
    self.material = Cache.Shader("indexed_material", "indexed_material")
    self.postShader = Cache.Shader("indexed_postprocess", "indexed_postprocess")
    self.identity = Cache.Shader("ui", "filter/identity")
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

    Viewport.Push(0, 0, TARGET_W, TARGET_H, true)
    RenderState.PushAllDefaults()
    RenderState.PushBlendMode(BlendMode.Disabled)
    RenderState.PushCullFace(CullFace.None)
    RenderState.PushDepthTest(false)
    RenderState.PushDepthWritable(false)

    RenderTarget.Push(TARGET_W, TARGET_H)
    RenderTarget.BindTex2D(self.source)
    Draw.Clear(0.0, 0.0, 0.0, 1.0)
    self.material:start()
    self.material:setFloat3("color", SOURCE_COLOR[1], SOURCE_COLOR[2], SOURCE_COLOR[3])
    self.mesh:draw()
    self.material:stop()
    RenderTarget.Pop()

    RenderTarget.Push(TARGET_W, TARGET_H)
    RenderTarget.BindTex2D(self.post)
    Draw.Clear(0.0, 0.0, 0.0, 1.0)
    self.postShader:start()
    self.postShader:setTex2D("src", self.source)
    self.mesh:draw()
    self.postShader:stop()
    RenderTarget.Pop()

    Viewport.Pop()
    -- Renderer-owned targets are color-only; the WGPU surface carries its
    -- own depth attachment. Match that surface only for the final composite.
    RenderState.PopDepthWritable()
    RenderState.PopDepthTest()
    RenderState.PushDepthTest(true)
    RenderState.PushDepthWritable(true)

    Viewport.Push(0, 0, self.resX, self.resY, true)
    self.identity:start()
    self.identity:setTex2D("src", self.post)
    Draw.Rect(0, self.resY, self.resX, -self.resY)
    self.identity:stop()
    Viewport.Pop()

    RenderState.PopDepthWritable()
    RenderState.PopDepthTest()
    RenderState.PopCullFace()
    RenderState.PopBlendMode()
    RenderState.PopAll()

    local source = self.source:sample(math.floor(TARGET_W / 2), math.floor(TARGET_H / 2))
    local post = self.post:sample(math.floor(TARGET_W / 2), math.floor(TARGET_H / 2))
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
    self.identity = nil
    Log.Info("[IndexedPostProcessProbe] resources released")
end

return RenderingIndexedPostProcess
