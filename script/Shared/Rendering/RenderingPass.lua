---@class RenderingPassClear
---@field color number[]|nil  { r, g, b, a } clear value for every color attachment; nil keeps the contents (LoadOp.Load)
---@field depth number|nil    clear value for the depth attachment; nil keeps the contents (LoadOp.Load)

---@class RenderingPass
---@field name string
---@field bufferOrder BufferName[]
---@field onStartFn function | nil
---@field clear RenderingPassClear
---@field descs table[] small cache of { textures = Tex2D[], desc = RenderPassDesc }
---@field pass RenderPass|nil the open render pass between start() and stop()

-- Descriptors are cached per texture set. The buffers are swapped between
-- passes (buffer0/buffer1/buffer2), so a few distinct sets recur; anything
-- beyond this is a stale set from before a resize.
local MAX_CACHED_DESCS = 4

---@class RenderingPass
---@overload fun(self: RenderingPass, name: string, bufferOrder: BufferName[], clear: RenderingPassClear|nil, onStartFn: function|nil)   class internal
---@overload fun(name: string, bufferOrder: BufferName[], clear: RenderingPassClear|nil, onStartFn: function | nil)  class external
local RenderingPass = Class("RenderingPass", function(self, name, bufferOrder, clear, onStartFn)
    ---@diagnostic disable-next-line: invisible
    self:registerVars(name, bufferOrder, clear, onStartFn)
end)

---@param name string
---@param bufferOrder BufferName[]
---@param clear RenderingPassClear|nil
---@param onStartFn function | nil
---@private
function RenderingPass:registerVars(name, bufferOrder, clear, onStartFn)
    self.name = name
    self.bufferOrder = bufferOrder
    self.clear = clear or {}
    self.onStartFn = onStartFn
    self.descs = {}
    self.pass = nil
end

--- The `RenderPassDesc` for these buffers: color attachments in `bufferOrder`
--- order, the depth-format buffer as depth.
---@param buffers table<BufferName, Tex2D>
---@return RenderPassDesc
---@private
function RenderingPass:getDesc(buffers)
    local order = self.bufferOrder
    for _, entry in ipairs(self.descs) do
        local same = true
        for i = 1, #order do
            if entry.textures[i] ~= buffers[order[i]] then
                same = false
                break
            end
        end
        if same then return entry.desc end
    end

    local desc = RenderPassDesc.Create(self.name)
    local textures = {}
    local colorIndex = 0
    local c = self.clear.color
    local d = self.clear.depth
    for i = 1, #order do
        local tex = buffers[order[i]]
        textures[i] = tex
        if TexFormat.IsDepth(tex:getFormat()) then
            desc:depth(tex:view(), d and LoadOp.Clear or LoadOp.Load, d or 1.0)
        else
            if c then
                desc:color(colorIndex, tex:view(), LoadOp.Clear, c[1], c[2], c[3], c[4])
            else
                desc:color(colorIndex, tex:view(), LoadOp.Load, 0, 0, 0, 0)
            end
            colorIndex = colorIndex + 1
        end
    end

    if #self.descs >= MAX_CACHED_DESCS then
        table.remove(self.descs, 1)
    end
    self.descs[#self.descs + 1] = { textures = textures, desc = desc }
    return desc
end

--- Begin the pass. Fixed-function state is not set here: every draw inside a
--- scene pass brings its own pipeline (a material's, or `Pipelines.get`).
---@param buffers table<BufferName, Tex2D>
---@return RenderPass
function RenderingPass:start(buffers)
    self.pass = Renderer:beginPass(self:getDesc(buffers))

    if self.onStartFn then
        self.onStartFn()
    end
    return self.pass
end

function RenderingPass:stop()
    self.pass:finish()
    self.pass = nil
end

return RenderingPass
