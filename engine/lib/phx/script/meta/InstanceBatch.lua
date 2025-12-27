-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class InstanceBatch
InstanceBatch = {}

-- Create a new instance batch for a mesh.
-- The mesh must have been created in command mode (has a ResourceId).
---@param mesh Mesh
---@param primitive integer
---@return InstanceBatch?
function InstanceBatch.Create(mesh, primitive) end

-- Add an instance with a 4x4 transform matrix and RGBA color.
-- Matrix is in column-major order (OpenGL convention).
---@param m00 number
---@param m01 number
---@param m02 number
---@param m03 number
---@param m10 number
---@param m11 number
---@param m12 number
---@param m13 number
---@param m20 number
---@param m21 number
---@param m22 number
---@param m23 number
---@param m30 number
---@param m31 number
---@param m32 number
---@param m33 number
---@param r number
---@param g number
---@param b number
---@param a number
function InstanceBatch:addInstance(m00, m01, m02, m03, m10, m11, m12, m13, m20, m21, m22, m23, m30, m31, m32, m33, r, g, b, a) end

-- Add an instance using a Matrix object and color values.
---@param matrix Matrix
---@param r number
---@param g number
---@param b number
---@param a number
function InstanceBatch:addInstanceMatrix(matrix, r, g, b, a) end

-- Get the current number of instances in the batch
---@return integer
function InstanceBatch:getInstanceCount() end

-- Clear all instances (reuse batch for next frame)
function InstanceBatch:clear() end

-- Submit the batch for drawing.
-- This sends a DrawInstancedWithData command to the render thread.
function InstanceBatch:draw() end

-- Submit and clear in one call (common pattern for per-frame batches)
function InstanceBatch:flush() end

