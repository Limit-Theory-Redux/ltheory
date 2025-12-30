-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class RenderBatchApi
RenderBatchApi = {}

-- Begin a new batch
function RenderBatchApi.Begin() end

-- Set the camera for frustum culling
---@param view00 number
---@param view01 number
---@param view02 number
---@param view03 number
---@param view10 number
---@param view11 number
---@param view12 number
---@param view13 number
---@param view20 number
---@param view21 number
---@param view22 number
---@param view23 number
---@param view30 number
---@param view31 number
---@param view32 number
---@param view33 number
---@param proj00 number
---@param proj01 number
---@param proj02 number
---@param proj03 number
---@param proj10 number
---@param proj11 number
---@param proj12 number
---@param proj13 number
---@param proj20 number
---@param proj21 number
---@param proj22 number
---@param proj23 number
---@param proj30 number
---@param proj31 number
---@param proj32 number
---@param proj33 number
---@param eyeX number
---@param eyeY number
---@param eyeZ number
function RenderBatchApi.SetCamera(view00, view01, view02, view03, view10, view11, view12, view13, view20, view21, view22, view23, view30, view31, view32, view33, proj00, proj01, proj02, proj03, proj10, proj11, proj12, proj13, proj20, proj21, proj22, proj23, proj30, proj31, proj32, proj33, eyeX, eyeY, eyeZ) end

-- Add an entity to the batch
---@param t00 number
---@param t01 number
---@param t02 number
---@param t03 number
---@param t10 number
---@param t11 number
---@param t12 number
---@param t13 number
---@param t20 number
---@param t21 number
---@param t22 number
---@param t23 number
---@param t30 number
---@param t31 number
---@param t32 number
---@param t33 number
---@param boundsX number
---@param boundsY number
---@param boundsZ number
---@param boundsRadius number
---@param meshVao integer
---@param indexCount integer
---@param shaderHandle integer
---@param sortKey integer
function RenderBatchApi.AddEntity(t00, t01, t02, t03, t10, t11, t12, t13, t20, t21, t22, t23, t30, t31, t32, t33, boundsX, boundsY, boundsZ, boundsRadius, meshVao, indexCount, shaderHandle, sortKey) end

-- Flush the batch (serial processing, no workers)
function RenderBatchApi.Flush() end

-- Get number of entities in current batch
---@return integer
function RenderBatchApi.GetEntityCount() end

-- Get stats from last flush
---@return integer
function RenderBatchApi.GetEntitiesSubmitted() end

---@return integer
function RenderBatchApi.GetEntitiesVisible() end

---@return integer
function RenderBatchApi.GetEntitiesCulled() end

---@return integer
function RenderBatchApi.GetCommandsGenerated() end

