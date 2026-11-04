-- AUTO GENERATED. DO NOT MODIFY!
-- BindGroupDesc ---------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct BindGroupDesc {} BindGroupDesc;
    ]]

    return 1, 'BindGroupDesc'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local BindGroupDesc

    do -- C Definitions
        ffi.cdef [[
            void           BindGroupDesc_Free    (BindGroupDesc*);
            BindGroupDesc* BindGroupDesc_Create  (Shader const* shader, int group);
            void           BindGroupDesc_Texture (BindGroupDesc*, cstr name, TexView const* view, uint32 sampler);
        ]]
    end

    do -- Global Symbol Table
        BindGroupDesc = {
            Create  = function(shader, group)
                local _instance = libphx.BindGroupDesc_Create(shader, group)
                return Core.ManagedObject(_instance, libphx.BindGroupDesc_Free)
            end,
        }

        if onDef_BindGroupDesc then onDef_BindGroupDesc(BindGroupDesc, mt) end
        BindGroupDesc = setmetatable(BindGroupDesc, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('BindGroupDesc')
        local mt = {
            __index = {
                texture = libphx.BindGroupDesc_Texture,
            },
        }

        if onDef_BindGroupDesc_t then onDef_BindGroupDesc_t(t, mt) end
        BindGroupDesc_t = ffi.metatype(t, mt)
    end

    return BindGroupDesc
end

return Loader
