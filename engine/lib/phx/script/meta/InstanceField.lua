-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class InstanceField
InstanceField = {}

-- `pos` is `x, y, z` per asteroid, `chunk_offsets` the `chunks + 1`
-- prefix offsets into `chunk_indices` (0-based asteroid indices),
-- `chunk_centroids` is `x, y, z` per chunk. Copied.
---@param pos number[]
---@param pos_size integer
---@param scales number[]
---@param scales_size integer
---@param chunkOffsets integer[]
---@param chunkOffsets_size integer
---@param chunkIndices integer[]
---@param chunkIndices_size integer
---@param chunkCentroids number[]
---@param chunkCentroids_size integer
---@return InstanceField
function InstanceField.Create(pos, pos_size, scales, scales_size, chunkOffsets, chunkOffsets_size, chunkIndices, chunkIndices_size, chunkCentroids, chunkCentroids_size) end

-- Worker threads for the cull. 0 and 1 mean single-threaded.
---@param workers integer
function InstanceField:setWorkers(workers) end

-- Number of LOD levels that have a mesh; asteroids that select a
-- higher level are skipped.
---@param count integer
function InstanceField:setLodCount(count) end

-- 0-based indices of asteroids that are real entities and are not drawn
-- by the next `Cull`. Copied.
---@param spawned integer[]
---@param spawned_size integer
function InstanceField:setSpawned(spawned, spawned_size) end

-- No spawned asteroids for the next `Cull`.
function InstanceField:clearSpawned() end

-- Cull the field (see the module docs), skipping the asteroids given to
-- `SetSpawned`. Returns the number of instances to draw.
---@param eyeX number
---@param eyeY number
---@param eyeZ number
---@param fwdX number
---@param fwdY number
---@param fwdZ number
---@param originX number
---@param originY number
---@param originZ number
---@param pxPerUnitSq number
---@param renderDistSq number
---@param maxDrawn integer
---@return integer
function InstanceField:cull(eyeX, eyeY, eyeZ, fwdX, fwdY, fwdZ, originX, originY, originZ, pxPerUnitSq, renderDistSq, maxDrawn) end

-- Instances of LOD `lod` (0-based) after the last cull.
---@param lod integer
---@return integer
function InstanceField:getCount(lod) end

-- Number of LODs that had instances in the last cull.
---@return integer
function InstanceField:getLodOrderLen() end

-- The `k`th LOD (0-based) by first appearance in the last cull.
---@param k integer
---@return integer
function InstanceField:getLodOrder(k) end

-- Record the instanced draw of LOD `lod` with `mesh`. The indices are
-- copied into the vertex ring on the calling thread.
---@param pass RenderPass
---@param r Renderer
---@param lod integer
---@param mesh Mesh
function InstanceField:draw(pass, r, lod, mesh) end

