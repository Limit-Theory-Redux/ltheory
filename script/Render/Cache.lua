local Cache    = {}

local files    = {}
local fonts    = {}
local shaders  = {}
local textures = {}

-- Track shader key -> {vs, fs} for hot reload
local shaderInfo = {}

-- Track last working shader for fallback on compile errors
-- Key: shader key, Value: last working Shader object
local lastWorkingShaders = {}

function Cache.Clear()
    -- Shaders are managed by Rust Drop, no need to free
    for k, v in pairs(textures) do v:free() end
    shaders = {}
    shaderInfo = {}
    lastWorkingShaders = {}
    textures = {}
end

function Cache.File(path)
    if not File.Exists(path) then return nil end
    if files[path] then return files[path] end
    local f = io.open(path, 'rb')
    if not f then Log.Error('Failed to open file <%s> for reading', path) end
    local self = f:read('*a')
    f:close()
    files[path] = self
    return self
end

-- TODO AB : Figure out proper way to do UI font caching
function Cache.Font(name, size)
    local key = name .. size
    local self = fonts[key]
    if self then return self end
    self = Font.Load(name, size)
    fonts[key] = self
    return self
end

function Cache.Shader(vs, fs)
    local key = vs .. ':' .. fs
    local self = shaders[key]
    if self then return self end

    local vsPath = 'vertex/' .. vs
    local fsPath = 'fragment/' .. fs
    self = Shader.Load(vsPath, fsPath)
    shaders[key] = self
    shaderInfo[key] = { vs = vs, fs = fs, vsPath = vsPath, fsPath = fsPath }

    -- Store as last working version for fallback
    lastWorkingShaders[key] = self

    -- Register with shader watcher for hot reload if active
    if ShaderWatcher and ShaderWatcher.IsActive() then
        local vsFile = Resource.GetPath(ResourceType.Shader, vsPath)
        local fsFile = Resource.GetPath(ResourceType.Shader, fsPath)
        Log.Debug("ShaderWatcher: Registering shader key='%s' vs='%s' fs='%s'", key, vsFile, fsFile)
        ShaderWatcher.Register(key, vsFile, fsFile)
    end

    return self
end

--- Try to reload a shader, falling back to last working version on failure.
--- Returns the new shader if successful, or the old shader if compilation failed.
---@param key string The shader key (vs..fs)
---@return Shader|nil shader The shader (new or fallback), or nil if no fallback exists
---@return boolean success True if the new shader compiled successfully
function Cache.TryReloadShader(key)
    local info = shaderInfo[key]
    if not info then
        Log.Warn("Cache.TryReloadShader: No info for shader '%s'", key)
        return nil, false
    end

    local vsPath = info.vsPath
    local fsPath = info.fsPath

    -- Check if render thread is active
    if Engine and Engine:isRenderThreadActive() then
        -- Compile shader on render thread (which has the GL context)
        Log.Debug("ShaderHotReload: Reloading shader '%s' on render thread", key)

        local ok, success = pcall(function()
            return Engine:reloadShaderOnRenderThread(key, vsPath, fsPath)
        end)

        if not ok then
            Log.Error("Cache.TryReloadShader: Exception during render thread reload: %s", tostring(success))
            return lastWorkingShaders[key], false
        end

        if success then
            Log.Info("ShaderHotReload: Successfully recompiled '%s' on render thread", key)

            -- Clear any previous errors for this shader
            if ShaderError then
                local errorKey = vsPath .. ":" .. fsPath
                ShaderError.ClearForShader(errorKey)
            end

            -- Invalidate the shader's resource_id to force fresh creation on next use.
            -- This ensures uniforms are properly re-bound to the hot-reloaded program.
            local shader = shaders[key]
            if shader then
                shader:invalidate()
            end

            return shaders[key], true
        else
            Log.Warn("ShaderHotReload: Compilation failed on render thread for '%s'", key)
            return lastWorkingShaders[key], false
        end
    end

    -- Direct GL mode: compile on main thread
    local ok, newShader = pcall(function()
        return Shader.TryLoad(vsPath, fsPath)
    end)

    if not ok then
        Log.Error("Cache.TryReloadShader: Exception during shader load: %s", tostring(newShader))
        return lastWorkingShaders[key], false
    end

    if newShader then
        -- Success! Update caches
        shaders[key] = newShader
        lastWorkingShaders[key] = newShader
        Log.Info("ShaderHotReload: Successfully recompiled '%s'", key)

        -- Clear any previous errors for this shader
        -- Note: Rust uses format "vsPath:fsPath" for shader keys in errors
        if ShaderError then
            local errorKey = vsPath .. ":" .. fsPath
            ShaderError.ClearForShader(errorKey)
        end

        -- Re-register with watcher
        if ShaderWatcher and ShaderWatcher.IsActive() then
            local vsFile = Resource.GetPath(ResourceType.Shader, vsPath)
            local fsFile = Resource.GetPath(ResourceType.Shader, fsPath)
            ShaderWatcher.Register(key, vsFile, fsFile)
        end

        return newShader, true
    else
        -- Compilation failed - fall back to last working version
        local fallback = lastWorkingShaders[key]
        if fallback then
            Log.Warn("ShaderHotReload: Compilation failed for '%s', using last working version", key)
            -- Keep the old shader in the cache
            shaders[key] = fallback
            return fallback, false
        else
            Log.Error("ShaderHotReload: Compilation failed for '%s' and no fallback available!", key)
            return nil, false
        end
    end
end

--- Get the last working shader for a key (for fallback purposes)
---@param key string The shader key
---@return Shader|nil shader The last working shader, or nil
function Cache.GetLastWorkingShader(key)
    return lastWorkingShaders[key]
end

--- Invalidate a cached shader, forcing reload on next Cache.Shader() call
---@param key string The shader key (vs..fs)
function Cache.InvalidateShader(key)
    if shaders[key] then
        -- Shader handles its own memory via Rust Drop, just remove from cache
        shaders[key] = nil
    end
    -- Keep shaderInfo so we know what to reload
end

--- Get info about a cached shader
---@param key string The shader key
---@return table|nil info Table with vs, fs, vsPath, fsPath or nil
function Cache.GetShaderInfo(key)
    return shaderInfo[key]
end

--- Get all loaded shader keys
---@return table keys Array of shader keys
function Cache.GetShaderKeys()
    local keys = {}
    for key, _ in pairs(shaders) do
        table.insert(keys, key)
    end
    return keys
end

function Cache.Texture(name, filtered)
    local self = textures[name]
    if self then return self end
    self = Tex2D.Load(name)
    textures[name] = self
    if filtered then
        self:setMagFilter(TexFilter.Linear)
        self:setMinFilter(TexFilter.LinearMipLinear)
        self:setWrapMode(TexWrapMode.Clamp)
        self:genMipmap()
    end
    return self
end

return Cache
