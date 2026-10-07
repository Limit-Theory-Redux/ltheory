local ffi = require('ffi')
local libphx = require('libphx').lib

-- Now takes the current Renderer as an explicit argument (see
-- doc/engine/render-thread.md); inject the global `Renderer` set by
-- SetEngine so call sites don't change.
function onDef_Shader(t, mt)
    t.Create = function(vs, fs)
        local _instance = libphx.Shader_Create(Renderer, vs, fs)
        return Core.ManagedObject(_instance, libphx.Shader_Free)
    end

    t.Load = function(vsName, fsName)
        local _instance = libphx.Shader_Load(Renderer, vsName, fsName)
        return Core.ManagedObject(_instance, libphx.Shader_Free)
    end
end

-- LuaJIT struct types generated from reflected uniform blocks, by shader
-- resource (a reload gives the shader a new resource, so stale layouts are
-- never reused) and block name.
local blockTypes = {}

function onDef_Shader_t(t, mt)
    --- `shader:blockType('Params')`: a LuaJIT ctype with the exact byte layout
    --- of the shader's uniform block (std140 holes as `_padN` fields).
    mt.__index.blockType = function(self, name)
        local key = tostring(tonumber(self:resourceId())) .. ':' .. name
        local blockType = blockTypes[key]
        if blockType == nil then
            local decl = ffi.string(self:blockDecl(name))
            if decl == '' then
                error(string.format('Shader %s has no uniform block <%s>', ffi.string(self:name()), name), 2)
            end
            blockType = ffi.typeof(decl)
            blockTypes[key] = blockType
        end
        return blockType
    end

    mt.__index.reload = function(self)
        return libphx.Shader_Reload(self, Renderer)
    end
end
