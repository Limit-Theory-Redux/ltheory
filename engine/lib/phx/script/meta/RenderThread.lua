-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class RenderThread
RenderThread = {}

-- Check if command mode is enabled (render thread handles GL calls)
---@return boolean
function RenderThread.IsCommandMode() end

-- Check if GL context is available for rendering
-- Returns false if GL context was lost (e.g., after render thread shutdown)
---@return boolean
function RenderThread.IsGLAvailable() end

-- Enable command mode
-- Call this after the render thread has been started and GL context transferred
function RenderThread.EnableCommandMode() end

-- Disable command mode
-- Call this before stopping the render thread
function RenderThread.DisableCommandMode() end

