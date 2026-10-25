local ffi = require('ffi')
local libphx = require('libphx').lib

-- Hand-written export (the FFI generator cannot return raw pointers): the CPU
-- copy of the material's `MaterialParams` block.
ffi.cdef [[
    uint8_t* Material_Params(Material*);
]]

-- The Rust half of a material (doc/engine/render-api-v2.md, 3a); the Lua
-- `Material` class (`Shared/Rendering/Material.lua`) wraps it with the typed
-- parameter struct. Inject the global `Renderer` set by SetEngine so call
-- sites don't pass it.
function onDef_Material(t, mt)
    t.Create = function(shader, blend, cull, depthTest, depthWrite)
        local _instance = libphx.Material_Create(Renderer, shader, blend, cull, depthTest, depthWrite)
        return Core.ManagedObject(_instance, libphx.Material_Free)
    end
end

function onDef_Material_t(t, mt)
    local index = mt.__index

    index.commit = function(self)
        libphx.Material_Commit(self, Renderer)
    end

    --- The shader was hot reloaded: adopt its layout (see `Material:refresh`).
    --- Returns the report line; `params()` moved, and `commit()` must follow.
    index.refreshShader = function(self)
        return libphx.Material_RefreshShader(self, Renderer)
    end

    --- Raw pointer to the parameter block; cast it to the type of
    --- `shader:blockType('MaterialParams')`.
    index.paramsPointer = function(self)
        return libphx.Material_Params(self)
    end
end
