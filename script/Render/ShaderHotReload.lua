--[[
    ShaderHotReload - Automatic shader reloading during development

    Watches shader files for changes and automatically reloads affected materials.

    Usage:
        local ShaderHotReload = require('Render.ShaderHotReload')

        -- Initialize once at startup
        ShaderHotReload:init()

        -- Poll for changes each frame (e.g., in onUpdate)
        ShaderHotReload:update()

        -- Register materials for auto-reload
        ShaderHotReload:registerMaterial(material, "vs_name", "fs_name")

        -- Or manually handle changed shaders
        local changed = ShaderHotReload:pollChangedShaders()
        for _, key in ipairs(changed) do
            -- key is "vs_name" .. "fs_name"
        end
]]

local Cache = require('Render.Cache')

---@class ShaderHotReload
local ShaderHotReload = {}

-- Map from shader key to list of materials using that shader
local materialsByShader = {}

-- Is the watcher initialized?
local initialized = false

--- Initialize the shader hot reload system
---@return boolean success True if initialized successfully
function ShaderHotReload:init()
    if initialized then
        return true
    end

    if not ShaderWatcher then
        Log.Warn("ShaderWatcher not available - hot reload disabled")
        return false
    end

    if not ShaderWatcher.Init() then
        Log.Warn("Failed to initialize ShaderWatcher - hot reload disabled")
        return false
    end

    initialized = true
    Log.Info("ShaderHotReload: Initialized successfully")

    -- Re-register any shaders that were loaded before watcher was initialized
    self:registerExistingShaders()

    return true
end

--- Register all already-loaded shaders with the watcher
function ShaderHotReload:registerExistingShaders()
    if not self:isActive() then return end

    local count = 0
    for _, key in ipairs(Cache.GetShaderKeys()) do
        local info = Cache.GetShaderInfo(key)
        if info then
            local vsFile = Resource.GetPath(ResourceType.Shader, info.vsPath)
            local fsFile = Resource.GetPath(ResourceType.Shader, info.fsPath)
            Log.Info("ShaderHotReload: Registering existing shader '%s'", key)
            ShaderWatcher.Register(key, vsFile, fsFile)
            count = count + 1
        end
    end

    if count > 0 then
        Log.Info("ShaderHotReload: Registered %d existing shaders", count)
    end
end

--- Shutdown the shader hot reload system
function ShaderHotReload:shutdown()
    if initialized and ShaderWatcher then
        ShaderWatcher.Shutdown()
        initialized = false
        materialsByShader = {}
        Log.Info("ShaderHotReload: Shutdown")
    end
end

--- Check if hot reload is active
---@return boolean active
function ShaderHotReload:isActive()
    return initialized and ShaderWatcher and ShaderWatcher.IsActive()
end

--- Register a material for automatic shader reloading
---@param material Material The material to register
---@param vs string Vertex shader name (without 'vertex/' prefix)
---@param fs string Fragment shader name (without 'fragment/' prefix)
function ShaderHotReload:registerMaterial(material, vs, fs)
    if not self:isActive() then return end

    local key = vs .. fs
    if not materialsByShader[key] then
        materialsByShader[key] = {}
    end

    -- Check if already registered (avoid duplicates)
    for _, m in ipairs(materialsByShader[key]) do
        if m == material then
            return
        end
    end

    table.insert(materialsByShader[key], material)
end

--- Unregister a material from hot reload tracking
---@param material Material The material to unregister
function ShaderHotReload:unregisterMaterial(material)
    for key, materials in pairs(materialsByShader) do
        for i, m in ipairs(materials) do
            if m == material then
                table.remove(materials, i)
                break
            end
        end
    end
end

--- Poll for changed shaders and return their keys
---@return table changedKeys Array of shader keys that changed
function ShaderHotReload:pollChangedShaders()
    local changed = {}

    if not self:isActive() then
        return changed
    end

    local count = ShaderWatcher.Poll()
    if count > 0 then
        for i = 0, count - 1 do
            local key = ShaderWatcher.GetChanged(i)
            if key then
                table.insert(changed, ffi.string(key))
            end
        end
        ShaderWatcher.ClearChanged()
    end

    return changed
end

--- Update hot reload - poll for changes and reload affected materials
--- Call this once per frame in your update loop
---@return number reloadedCount Number of materials reloaded
---@return number failedCount Number of shaders that failed to compile
function ShaderHotReload:update()
    if not self:isActive() then
        return 0, 0
    end

    local changed = self:pollChangedShaders()
    local reloadedCount = 0
    local failedCount = 0

    for _, key in ipairs(changed) do
        Log.Info(">>> SHADER CHANGED: '%s' - attempting reload...", key)

        -- Try to reload using TryReloadShader (with fallback)
        local shader, success = Cache.TryReloadShader(key)

        if success then
            -- Shader compiled successfully - reload materials
            local materials = materialsByShader[key]
            if materials then
                for _, material in ipairs(materials) do
                    if material.reloadShader then
                        local ok = pcall(function()
                            material:reloadShader()
                        end)
                        if ok then
                            reloadedCount = reloadedCount + 1
                        else
                            Log.Warn("ShaderHotReload: Failed to reload material for shader '%s'", key)
                        end
                    end
                end
            end
        else
            -- Shader compilation failed - error already pushed to ShaderError queue
            failedCount = failedCount + 1
        end
    end

    return reloadedCount, failedCount
end

--- Manually trigger reload of a specific shader and its materials
---@param vs string Vertex shader name
---@param fs string Fragment shader name
---@return boolean success True if the shader compiled successfully
function ShaderHotReload:reloadShader(vs, fs)
    local key = vs .. fs

    Log.Info("ShaderHotReload: Manually reloading shader '%s'", key)

    -- Try to reload with fallback
    local shader, success = Cache.TryReloadShader(key)

    if success then
        -- Reload materials on success
        local materials = materialsByShader[key]
        if materials then
            for _, material in ipairs(materials) do
                if material.reloadShader then
                    pcall(function()
                        material:reloadShader()
                    end)
                end
            end
        end
    end

    return success
end

--- Check if there are shader compilation errors
---@return boolean hasErrors True if there are shader errors to display
function ShaderHotReload:hasErrors()
    return ShaderError and ShaderError.HasNewErrors()
end

--- Get the count of shader errors
---@return number count Number of shader errors in the queue
function ShaderHotReload:getErrorCount()
    return ShaderError and ShaderError.GetCount() or 0
end

--- Get the latest shader error message for display
---@return string|nil message The formatted error message, or nil if no errors
function ShaderHotReload:getLatestError()
    if not ShaderError then return nil end
    local msg = ShaderError.GetLatestMessage()
    return msg and ffi.string(msg) or nil
end

--- Acknowledge that errors have been seen (clears the "new" flag)
function ShaderHotReload:acknowledgeErrors()
    if ShaderError then
        ShaderError.AcknowledgeErrors()
    end
end

--- Clear all shader errors from the queue
function ShaderHotReload:clearErrors()
    if ShaderError then
        ShaderError.Clear()
    end
end

return ShaderHotReload
