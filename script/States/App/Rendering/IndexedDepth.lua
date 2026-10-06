local Application = require("States.Application")
local ProbePaths = require("States.App.Rendering.ProbePaths")

---@class RenderingIndexedDepth: Application
local RenderingIndexedDepth = Subclass("RenderingIndexedDepth", Application)

local TARGET_W = 128
local TARGET_H = 128

function RenderingIndexedDepth:onInit()
    self.color = Tex2D.Create(TARGET_W, TARGET_H, TexFormat.RGBA8)
    self.depth = Tex2D.Create(TARGET_W, TARGET_H, TexFormat.Depth32F)
    self.mesh = Mesh.Box(2)
    self.shader = Cache.Shader("indexed_depth", "indexed_depth")
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

    Viewport.Push(0, 0, TARGET_W, TARGET_H, true)
    RenderState.PushAllDefaults()
    RenderState.PushBlendMode(BlendMode.Disabled)
    RenderState.PushCullFace(CullFace.None)
    RenderState.PushDepthTest(true)
    RenderState.PushDepthWritable(true)

    RenderTarget.Push(TARGET_W, TARGET_H)
    RenderTarget.BindTex2D(self.color)
    RenderTarget.BindTex2D(self.depth)
    Draw.Clear(0.0, 0.0, 0.0, 1.0)
    Draw.ClearDepth(1.0)

    self.shader:start()
    -- Near orange is written first; farther blue must be rejected by depth.
    self.shader:setFloat("depthValue", 0.25)
    self.shader:setFloat4("color", 1.0, 0.125, 0.0, 1.0)
    self.mesh:draw()
    self.shader:setFloat("depthValue", 0.75)
    self.shader:setFloat4("color", 0.0, 0.125, 1.0, 1.0)
    self.mesh:draw()
    self.shader:stop()

    RenderTarget.Pop()
    RenderState.PopDepthWritable()
    RenderState.PopDepthTest()
    RenderState.PopCullFace()
    RenderState.PopBlendMode()
    RenderState.PopAll()
    Viewport.Pop()

    local tl = self.color:sample(0, 0)
    local center = self.color:sample(math.floor(TARGET_W / 2), math.floor(TARGET_H / 2))
    local br = self.color:sample(TARGET_W - 1, TARGET_H - 1)
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
