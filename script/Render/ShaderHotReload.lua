--[[
    ShaderHotReload - Automatic shader reloading during development

    Watches shader files for changes (via the Rust ShaderWatcher) and
    automatically reloads affected shaders + the materials that use them.

    Usage:
        local ShaderHotReload = require('Render.ShaderHotReload')

        -- Once at startup, before any Cache.Shader() call:
        ShaderHotReload:init()

        -- Once per frame:
        ShaderHotReload:update()

        -- Wherever a material is created:
        ShaderHotReload:registerMaterial(material, vsName, fsName)
]]

local Cache = require('Render.Cache')

---@class ShaderHotReload
local ShaderHotReload = {}

-- Map from canonical shader key ('vs:fs') to list of materials using it.
local materialsByShader = {}

local initialized = false

--- Initialize the shader hot reload system. Safe to call even after some
--- shaders were already cached (e.g. materials loaded eagerly at
--- `require`-time, before this runs) - already-cached shaders are
--- registered with the watcher as a catch-up pass.
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

    self:registerExistingShaders()

    return true
end

--- Register every shader already in Cache with the watcher. Called once
--- from init() to catch shaders cached before the watcher was active.
function ShaderHotReload:registerExistingShaders()
    local count = 0
    for _, key in ipairs(Cache.GetShaderKeys()) do
        local info = Cache.GetShaderInfo(key)
        if info then
            local vsFile = Resource.GetPath(ResourceType.Shader, info.vsPath)
            local fsFile = Resource.GetPath(ResourceType.Shader, info.fsPath)
            ShaderWatcher.Register(key, vsFile, fsFile)
            count = count + 1
        end
    end

    if count > 0 then
        Log.Info("ShaderHotReload: Registered %d pre-loaded shader(s)", count)
    end
end

--- Shutdown the shader hot reload system.
function ShaderHotReload:shutdown()
    if initialized and ShaderWatcher then
        ShaderWatcher.Shutdown()
        initialized = false
        materialsByShader = {}
        Log.Info("ShaderHotReload: Shutdown")
    end
end

--- Check if hot reload is active.
---@return boolean active
function ShaderHotReload:isActive()
    return initialized and ShaderWatcher and ShaderWatcher.IsActive()
end

--- Register a material for automatic reload when its shader changes.
---@param material Material The material to register
---@param vs string Vertex shader name (without 'vertex/' prefix)
---@param fs string Fragment shader name (without 'fragment/' prefix)
function ShaderHotReload:registerMaterial(material, vs, fs)
    if not self:isActive() then return end

    local key = vs .. ':' .. fs
    local materials = materialsByShader[key]
    if not materials then
        materials = {}
        materialsByShader[key] = materials
    end

    for _, m in ipairs(materials) do
        if m == material then return end
    end

    table.insert(materials, material)
end

--- Unregister a material from hot reload tracking.
---@param material Material The material to unregister
function ShaderHotReload:unregisterMaterial(material)
    for _, materials in pairs(materialsByShader) do
        for i, m in ipairs(materials) do
            if m == material then
                table.remove(materials, i)
                break
            end
        end
    end
end

--- Poll for changed shaders and reload them (and their registered
--- materials). Call this once per frame.
---@return number reloadedCount Number of shaders successfully reloaded
---@return number failedCount Number of shaders that failed to compile
function ShaderHotReload:update()
    if not self:isActive() then
        return 0, 0
    end

    if ShaderError then
        ShaderError.Update()
    end

    local count = ShaderWatcher.Poll()
    if count == 0 then
        return 0, 0
    end

    local changed = {}
    for i = 0, count - 1 do
        local key = ShaderWatcher.GetChanged(i)
        if key then
            table.insert(changed, ffi.string(key))
        end
    end
    ShaderWatcher.ClearChanged()

    local reloadedCount = 0
    local failedCount = 0

    for _, key in ipairs(changed) do
        local shader = Cache.GetShader(key)
        if not shader then
            Log.Warn("ShaderHotReload: No cached shader for changed key '%s'", key)
        else
            Log.Info("ShaderHotReload: Reloading '%s'", key)
            local ok = shader:reload()
            if ok then
                reloadedCount = reloadedCount + 1
                if ShaderError then
                    ShaderError.ClearForShader(key)
                end

                local materials = materialsByShader[key]
                if materials then
                    for _, material in ipairs(materials) do
                        if material.reloadShader then
                            local reloadOk = pcall(function() material:reloadShader() end)
                            if not reloadOk then
                                Log.Warn("ShaderHotReload: Failed to reload material for shader '%s'", key)
                            end
                        end
                    end
                end
            else
                -- Compilation failed - error is already in the ShaderError queue.
                failedCount = failedCount + 1
            end
        end
    end

    return reloadedCount, failedCount
end

--- Whether there are shader compilation errors to display.
---@return boolean hasErrors
function ShaderHotReload:hasErrors()
    return ShaderError and ShaderError.HasNewErrors()
end

--- Count of shader errors currently in the queue.
---@return number count
function ShaderHotReload:getErrorCount()
    return ShaderError and ShaderError.GetCount() or 0
end

--- Latest shader error message, formatted for display.
---@return string|nil message
function ShaderHotReload:getLatestError()
    if not ShaderError then return nil end
    local msg = ShaderError.GetLatestMessage()
    return msg and ffi.string(msg) or nil
end

--- Acknowledge that errors have been seen (clears the "new" flag).
function ShaderHotReload:acknowledgeErrors()
    if ShaderError then
        ShaderError.AcknowledgeErrors()
    end
end

--- Clear all shader errors from the queue.
function ShaderHotReload:clearErrors()
    if ShaderError then
        ShaderError.Clear()
    end
end

return ShaderHotReload
