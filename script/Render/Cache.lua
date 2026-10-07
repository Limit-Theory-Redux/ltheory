local Cache    = {}

local files    = {}
local fonts    = {}
local shaders  = {}
local textures = {}

-- Track shader key -> {vsPath, fsPath} so already-cached shaders (many are
-- loaded eagerly at `require`-time, e.g. via MaterialDefs, before
-- ShaderHotReload:init() runs) can be registered with the watcher later.
local shaderInfo = {}

function Cache.Clear()
    for k, v in pairs(shaders) do v:free() end
    for k, v in pairs(textures) do v:free() end
    shaders = {}
    shaderInfo = {}
    textures = {}
end

-- Functions called after a shader was reloaded from disk (see Cache.OnShaderReload).
local reloadHooks = {}

--- Register `fn(shader, changed)`, called after a cached shader was reloaded
--- successfully. `changed` is the set of uniform block names whose layout
--- (`shader:blockHash`) differs from before (empty when only the code changed);
--- the function returns the set of block names it took care of. Whatever
--- changed and nobody handled is reported: LuaJIT types made from such a block
--- before the reload are stale.
---@param fn fun(shader: Shader, changed: table<string, boolean>): table<string, boolean>|nil
function Cache.OnShaderReload(fn)
    table.insert(reloadHooks, fn)
end

---@param shader Shader
---@return table<string, integer> layout hash by uniform block name
local function blockHashes(shader)
    local hashes = {}
    for name in ffi.string(shader:blockNames()):gmatch('[^\n]+') do
        hashes[name] = shader:blockHash(name)
    end
    return hashes
end

--- Reload one cached shader (`'vs:fs'`) from disk and let the reload hooks
--- bring everything that was made from it up to date. On a compile error the
--- old shader stays in use and the error is queued for the overlay.
---@param key string
---@return boolean ok
function Cache.ReloadShader(key)
    local shader = shaders[key]
    if not shader then return false end

    local before = blockHashes(shader)
    if not shader:reload() then return false end
    local after = blockHashes(shader)

    local changed = {}
    for name, hash in pairs(after) do
        if before[name] ~= hash then changed[name] = true end
    end
    for name in pairs(before) do
        if after[name] == nil then changed[name] = true end
    end

    local handled = {}
    for _, hook in ipairs(reloadHooks) do
        local ok, result = pcall(hook, shader, changed)
        if not ok then
            Log.Warn("Shader '%s' reloaded, but a reload hook failed: %s", key, tostring(result))
        elseif result then
            for name in pairs(result) do handled[name] = true end
        end
    end
    for name in pairs(changed) do
        if not handled[name] then
            Log.Warn("Shader '%s': the layout of uniform block <%s> changed; types made from it before the reload are stale",
                key, name)
        end
    end
    return true
end

--- Hot-reload all cached shaders from disk.
--- Returns the number of shaders successfully reloaded.
function Cache.ReloadShaders()
    local count = 0
    local failed = 0
    local keys = Cache.GetShaderKeys()
    for _, key in ipairs(keys) do
        if Cache.ReloadShader(key) then
            count = count + 1
        else
            failed = failed + 1
        end
    end
    Log.Info("Shader hot-reload: %d reloaded, %d failed", count, failed)
    return count, failed
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
    shaderInfo[key] = { vsPath = vsPath, fsPath = fsPath }

    if ShaderWatcher and ShaderWatcher.IsActive() then
        local vsFile = Resource.GetPath(ResourceType.Shader, vsPath)
        local fsFile = Resource.GetPath(ResourceType.Shader, fsPath)
        ShaderWatcher.Register(key, vsFile, fsFile)
    end

    return self
end

--- Look up an already-cached shader by its canonical `vs:fs` key.
--- Used by ShaderHotReload to reload the exact Shader object in place.
function Cache.GetShader(key)
    return shaders[key]
end

--- All currently-cached shader keys ('vs:fs').
--- Used by ShaderHotReload to catch up shaders that were cached before the
--- watcher was initialized.
function Cache.GetShaderKeys()
    local keys = {}
    for key in pairs(shaders) do
        table.insert(keys, key)
    end
    return keys
end

--- {vsPath, fsPath} for a cached shader key, or nil.
function Cache.GetShaderInfo(key)
    return shaderInfo[key]
end

function Cache.Texture(name, filtered)
    local self = textures[name]
    if self then return self end
    self = Tex2D.Load(name)
    textures[name] = self
    if filtered then
        -- Mip chain only: the filter and wrap mode belong to the sampler the user binds.
        self:genMipmap()
    end
    return self
end

return Cache
