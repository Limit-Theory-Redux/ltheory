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
---@overload fun(self: RenderingPass, bufferOrder: BufferName[], settings: RenderStateSettings, onStartFn: function|nil)   class internal
---@overload fun(bufferOrder: BufferName[], settings: RenderStateSettings, onStartFn: function | nil)  class external
local RenderingPass = Class("RenderingPass", function(self, bufferOrder, settings, onStartFn)
    ---@diagnostic disable-next-line: invisible
    self:registerVars(bufferOrder, settings, onStartFn)
end)

---@param bufferOrder BufferName[]
---@param settings RenderStateSettings
---@param onStartFn function | nil
---@private
function RenderingPass:registerVars(bufferOrder, settings, onStartFn)
    self.bufferOrder = bufferOrder
    self.settings = settings
    self.onStartFn = onStartFn
end

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
