-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class ShaderWatcher
ShaderWatcher = {}

-- Initializes the shader watcher. Call this once at startup to enable
-- shader hot-reloading.
---@param r Renderer
---@return boolean
function ShaderWatcher.Init(r) end

-- Shuts down the shader watcher.
---@param r Renderer
function ShaderWatcher.Shutdown(r) end

-- Checks whether the shader watcher is active.
---@param r Renderer
---@return boolean
function ShaderWatcher.IsActive(r) end

-- Registers a shader for hot-reload tracking.
-- 
-- * `shader_key` - The shader cache key (format: "vs_name:fs_name")
-- * `vs_path` - Resolved path to the vertex shader file
-- * `fs_path` - Resolved path to the fragment shader file
---@param r Renderer
---@param shaderKey string
---@param vsPath string
---@param fsPath string
function ShaderWatcher.Register(r, shaderKey, vsPath, fsPath) end

-- Polls for changed shaders and returns the count.
-- Use `GetChanged` to get the actual shader keys.
---@param r Renderer
---@return integer
function ShaderWatcher.Poll(r) end

-- Gets a changed shader key by index (0-based). Call `Poll` first.
---@param r Renderer
---@param index integer
---@return string?
function ShaderWatcher.GetChanged(r, index) end

-- Clears the list of changed shaders after processing.
---@param r Renderer
function ShaderWatcher.ClearChanged(r) end

