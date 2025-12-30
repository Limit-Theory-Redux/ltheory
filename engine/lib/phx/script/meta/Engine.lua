-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Engine
Engine = {}

---@return Window
function Engine:window() end

---@return Input
function Engine:input() end

---@return EventBus
function Engine:eventBus() end

---@return TaskQueue
function Engine:taskQueue() end

---@return HmGui
function Engine:hmGui() end

function Engine.Abort() end

---@return integer
function Engine.GetBits() end

-- Return time passed since engine start.
---@return number
function Engine:getTime() end

---@return string
function Engine.GetVersion() end

function Engine:exit() end

function Engine.Terminate() end

function Engine.Update() end

-- Start the multithreaded render system.
-- 
-- This transfers the GL context to a dedicated render thread.
-- All GL operations will then go through command mode.
-- Returns true if successfully started.
---@return boolean
function Engine:startRenderThread() end

-- Stop the multithreaded render system.
-- 
-- This shuts down the render thread and returns the GL context
-- to the main thread for direct rendering.
function Engine:stopRenderThread() end

-- Check if the render thread is currently active.
---@return boolean
function Engine:isRenderThreadActive() end

-- Get total commands processed by the render thread.
---@return integer
function Engine:getRenderThreadCommands() end

-- Get total draw calls executed by the render thread.
---@return integer
function Engine:getRenderThreadDrawCalls() end

-- Get total state changes on the render thread.
---@return integer
function Engine:getRenderThreadStateChanges() end

-- Get total frames rendered by the render thread.
---@return integer
function Engine:getRenderThreadFrameCount() end

-- Get the last frame render time in milliseconds.
---@return number
function Engine:getRenderThreadFrameTimeMs() end

-- Get commands processed in the last frame.
---@return integer
function Engine:getRenderThreadCommandsPerFrame() end

-- Get draw calls executed in the last frame.
---@return integer
function Engine:getRenderThreadDrawCallsPerFrame() end

-- Get total texture binds skipped due to caching (cumulative).
---@return integer
function Engine:getRenderThreadTextureBindsSkipped() end

-- Get main thread wait time in milliseconds (time spent waiting for render thread).
---@return number
function Engine:getMainThreadWaitTimeMs() end

-- Get current frames in flight (submitted but not yet rendered).
---@return integer
function Engine:getFramesInFlight() end

-- Get the number of CPU cores available for worker threads.
---@return integer
function Engine:getCpuCount() end

-- Get the number of worker threads that would be spawned.
-- This is CPU cores - 2 (reserve for main + render thread), minimum 1.
---@return integer
function Engine:getWorkerThreadCount() end

-- Check if the worker pool is active.
---@return boolean
function Engine:isWorkerPoolActive() end

-- Get the actual number of active workers in the pool.
---@return integer
function Engine:getActiveWorkerCount() end

-- Flush the render batch using the worker pool for parallel processing.
-- This submits accumulated entities to workers for frustum culling and
-- command generation. Returns the number of entities visible after culling.
---@return integer
function Engine:flushRenderBatch() end

-- Create the camera UBO on the render thread.
-- Call this once after starting the render thread.
function Engine:createCameraUBO() end

-- Update the camera UBO with new matrix and uniform data.
-- This should be called each frame before rendering when in command mode.
-- 
-- Parameters:
-- - m_view: View matrix (camera-relative, position at origin)
-- - m_view_inv: View inverse matrix (with actual world position for worldray.glsl)
-- - m_proj: Projection matrix
-- - eye_x, eye_y, eye_z: Camera eye position
-- - star_dir_x, star_dir_y, star_dir_z: Star/light direction
---@param mView Matrix
---@param mViewInv Matrix
---@param mProj Matrix
---@param eyeX number
---@param eyeY number
---@param eyeZ number
---@param starDirX number
---@param starDirY number
---@param starDirZ number
function Engine:updateCameraUBO(mView, mViewInv, mProj, eyeX, eyeY, eyeZ, starDirX, starDirY, starDirZ) end

-- Create the light UBO on the render thread.
-- This should be called once before using UpdateLightUBO.
function Engine:createLightUBO() end

-- Update the light UBO with new light data.
-- This should be called for each point light before rendering it.
-- 
-- Parameters:
-- - pos_x, pos_y, pos_z: Light position in world space
-- - radius: Light falloff radius
-- - r, g, b: Light color (0.0 to 1.0)
-- - intensity: Light intensity multiplier
---@param posX number
---@param posY number
---@param posZ number
---@param radius number
---@param r number
---@param g number
---@param b number
---@param intensity number
function Engine:updateLightUBO(posX, posY, posZ, radius, r, g, b, intensity) end

-- Reload a shader on the render thread.
-- 
-- This compiles a shader on the render thread (which owns the GL context)
-- and returns whether it succeeded. Use this for hot-reloading shaders
-- when the render thread is active.
-- 
-- Parameters:
-- - shader_key: The cache key for the shader (e.g., "wvpfragment/material/solidcolor")
-- - vs_name: Vertex shader resource name (e.g., "vertex/wvp")
-- - fs_name: Fragment shader resource name (e.g., "fragment/material/solidcolor")
-- 
-- Returns: true if shader compiled successfully, false otherwise
---@param shaderKey string
---@param vsName string
---@param fsName string
---@return boolean
function Engine:reloadShaderOnRenderThread(shaderKey, vsName, fsName) end

