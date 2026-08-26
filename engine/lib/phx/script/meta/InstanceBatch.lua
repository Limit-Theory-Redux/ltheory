-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class InstanceBatch
InstanceBatch = {}

-- Binds this batch to `mesh`'s GPU resource (lazily creating it, like
-- `Mesh::drawBind`) and its current index count. Changing which mesh a
-- batch draws requires creating a new `InstanceBatch`.
---@param mesh Mesh
---@param r Renderer
---@param primitive CmdPrimitiveType
---@return InstanceBatch
function InstanceBatch.Create(mesh, r, primitive) end

-- Queue one instance. Has no GL effect until `draw`/`flush`.
---@param transform Matrix
---@param r number
---@param g number
---@param b number
---@param a number
function InstanceBatch:addInstance(transform, r, g, b, a) end

-- Draw all queued instances in one instanced draw call. Does not clear
-- the queue - call `clear` (or use `flush`) to start the next batch.
---@param r Renderer
function InstanceBatch:draw(r) end

-- Drop all queued instances without drawing them.
function InstanceBatch:clear() end

-- `draw` followed by `clear` - the common per-frame pattern.
---@param r Renderer
function InstanceBatch:flush(r) end

---@return integer
function InstanceBatch:instanceCount() end

