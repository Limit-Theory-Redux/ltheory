local Application = require("States.Application")
local ProbePaths = require("States.App.Rendering.ProbePaths")

---@class RenderingIndexedTexture: Application
local RenderingIndexedTexture = Subclass("RenderingIndexedTexture", Application)

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

function RenderingIndexedTexture:onInit()
    self.target = Tex2D.Create(TARGET_W, TARGET_H, TexFormat.RGBA8)
    self.passDesc = RenderPassDesc.Create("IndexedTexture")
    self.passDesc:color(0, self.target:view(), LoadOp.Clear, 0.0, 0.0, 0.0, 1.0)
    self.texture = Tex2D.Create(2, 2, TexFormat.RGBA8)

    -- A uniform 2x2 payload isolates upload and sampling. Filtering is tested
    -- separately by the Upscale/downsample rung, not by this state.
    local bytes = Bytes.Create(2 * 2 * 4)
    for _ = 1, 4 do
        bytes:writeU8(64); bytes:writeU8(128); bytes:writeU8(192); bytes:writeU8(255)
    end
    self.texture:setDataBytes(bytes, PixelFormat.RGBA, DataFormat.U8)

    self.mesh = makeQuad()
    self.shader = Cache.Shader("indexed_texture", "indexed_texture")
    self.pipeline = Pipeline.Get(PipelineDesc.Create(self.shader))
    -- The texture is a material-style (group 1) binding: a bind group.
    local group = BindGroupDesc.Create(self.shader, 1)
    group:texture("tex", self.texture:view(), Samplers.LinearClamp)
    self.bindGroup = Renderer:createBindGroup(group)
    self.backend = os.getenv("LTHEORY_WGPU") and "wgpu" or "opengl"
    self.frames = 0
    self.probed = false
end

function RenderingIndexedTexture:eventLoop()
    Application.eventLoop(self)
    self.frames = self.frames + 1
    if self.frames >= 3 then
        self:quit()
    end
end

function RenderingIndexedTexture:onRender()
    if self.probed then return end

    local pass = Renderer:beginPass(self.passDesc)
    pass:setPipeline(self.pipeline)
    pass:setBindGroup(1, self.bindGroup)
    pass:drawMesh(self.mesh)
    pass:finish()

    local tl = self.target:sample(0, 0)
    local center = self.target:sample(math.floor(TARGET_W / 2), math.floor(TARGET_H / 2))
    local br = self.target:sample(TARGET_W - 1, TARGET_H - 1)
    local path = ProbePaths.file("indexed-texture-" .. self.backend .. ".png")
    self.target:save(path)
    Log.Info(string.format(
        "[IndexedTextureProbe] backend=%s target=%dx%d samples tl=(%.3f,%.3f,%.3f) center=(%.3f,%.3f,%.3f) br=(%.3f,%.3f,%.3f) artifact=%s",
        self.backend, TARGET_W, TARGET_H,
        tl.x, tl.y, tl.z,
        center.x, center.y, center.z,
        br.x, br.y, br.z,
        path
    ))
    self.probed = true
end

function RenderingIndexedTexture:onExit()
    self.target = nil
    self.texture = nil
    self.mesh = nil
    self.shader = nil
    Log.Info("[IndexedTextureProbe] resources released")
end

return RenderingIndexedTexture
