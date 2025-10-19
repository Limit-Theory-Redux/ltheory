---@class RenderStateSettings
---@field blendMode BlendMode
---@field cullFace CullFace
---@field depthTest boolean
---@field depthWritable boolean

---@class RenderingPass
---@field bufferOrder BufferName[]
---@field settings RenderStateSettings
---@field onStartFn function | nil
---@field screenX number
---@field screenY number

---@class RenderingPass
---@overload fun(self: RenderingPass, bufferOrder: BufferName[], settings: RenderStateSettings, drawFunc: function|nil)   class internal
---@overload fun(bufferOrder: BufferName[], settings: RenderStateSettings, drawFunc: function | nil)  class external
local RenderingPass = Class("RenderingPass", function(self, bufferOrder, settings, drawFunc)
    self.bufferOrder = bufferOrder
    self.settings = settings
    self.drawFunc = drawFunc
end)

---@param buffers table<BufferName, Buffer>
---@param resX number
---@param resY number
function RenderingPass:start(buffers, resX, resY)
    RenderTarget.Push(resX, resY)
    for _, buffer in ipairs(self.bufferOrder) do
        RenderTarget.BindTex2D(buffers[buffer])
    end

    if self.onStartFn then
        self.onStartFn()
    end

    RenderState.PushBlendMode(self.settings.blendMode)
    RenderState.PushCullFace(self.settings.cullFace)
    RenderState.PushDepthTest(self.settings.depthTest)
    RenderState.PushDepthWritable(self.settings.depthWritable)
end

function RenderingPass:stop()
    RenderState.PopBlendMode()
    RenderState.PopCullFace()
    RenderState.PopDepthTest()
    RenderState.PopDepthWritable()
    RenderTarget.Pop()
end

return RenderingPass
