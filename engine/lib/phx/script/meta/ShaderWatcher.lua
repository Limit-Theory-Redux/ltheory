-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class ShaderWatcher
ShaderWatcher = {}

-- Initializes the global shader watcher.
-- Call this once at startup to enable shader hot-reloading.
---@return boolean
function ShaderWatcher.Init() end

-- Shuts down the global shader watcher.
function ShaderWatcher.Shutdown() end

-- Checks if shader watcher is active.
---@return boolean
function ShaderWatcher.IsActive() end

-- Registers a shader for hot-reload tracking.
-- 
-- # Arguments
-- * `shader_key` - The shader cache key (format: "vs_name:fs_name")
-- * `vs_path` - Resolved path to vertex shader file
-- * `fs_path` - Resolved path to fragment shader file
---@param shaderKey string
---@param vsPath string
---@param fsPath string
function ShaderWatcher.Register(shaderKey, vsPath, fsPath) end

-- Polls for changed shaders and returns count.
-- Use GetChanged() to get the actual shader keys.
---@return integer
function ShaderWatcher.Poll() end

-- Gets a changed shader key by index (0-based).
-- Call Poll() first to get the count.
---@param index integer
---@return string?
function ShaderWatcher.GetChanged(index) end

-- Clears the list of changed shaders after processing.
function ShaderWatcher.ClearChanged() end

