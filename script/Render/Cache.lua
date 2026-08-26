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

--- Hot-reload all cached shaders from disk.
--- Returns the number of shaders successfully reloaded.
function Cache.ReloadShaders()
    local count = 0
    local failed = 0
    for key, shader in pairs(shaders) do
        if shader:reload() then
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
        self:setMagFilter(TexFilter.Linear)
        self:setMinFilter(TexFilter.LinearMipLinear)
        self:setWrapMode(TexWrapMode.Clamp)
        self:genMipmap()
    end
    return self
end

return Cache
