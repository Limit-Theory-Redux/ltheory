-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class SceneList
SceneList = {}

---@return SceneList
function SceneList.Create() end

-- Forget last frame's transforms and items (capacity is kept).
function SceneList:reset() end

-- Add one mesh of the entity whose transform is `transform` (an index
-- from `addTransform`). Returns the item index. Commits the material if
-- it has changed since it was last committed, so this must not run
-- inside an open pass. The bucket is the material's blend mode.
---@param r Renderer
---@param transform integer
---@param mesh Mesh
---@param material Material
---@return integer
function SceneList:addItem(r, transform, mesh, material) end

-- Cull and sort the items of the bucket `blend`; returns how many
-- survive. Lua then fills the per-draw values of the survivors that
-- have a callback and calls `emit`.
---@param r Renderer
---@param blend BlendMode
---@param cull boolean
---@return integer
function SceneList:prepare(r, blend, cull) end

-- Emit the draws of the last `prepare` into the open pass.
---@param r Renderer
function SceneList:emit(r) end

---@return integer
function SceneList:getItemCount() end

---@return integer
function SceneList:getTransformCount() end

-- Items submitted since the last reset (all buckets).
---@return integer
function SceneList:getSubmitted() end

-- Items that survived culling since the last reset.
---@return integer
function SceneList:getVisible() end

-- Items culled since the last reset.
---@return integer
function SceneList:getCulled() end

