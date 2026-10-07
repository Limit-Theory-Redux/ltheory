-- AUTO GENERATED. DO NOT MODIFY!
-- Material --------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct Material {} Material;
    ]]

    return 1, 'Material'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Material

    do -- C Definitions
        ffi.cdef [[
            void      Material_Free          (Material*);
            Material* Material_Create        (Renderer* r, Shader const* shader, BlendMode blend, CullFace cull, bool depthTest, bool depthWrite);
            uint32    Material_GetParamsSize (Material const*);
            void      Material_SetTexture    (Material*, cstr name, TexView const* view, uint32 sampler);
            void      Material_Commit        (Material*, Renderer* r);
            BlendMode Material_GetBlend      (Material const*);
        ]]
    end

    do -- Global Symbol Table
        Material = {
            Create        = function(r, shader, blend, cull, depthTest, depthWrite)
                local _instance = libphx.Material_Create(r, shader, blend, cull, depthTest, depthWrite)
                return Core.ManagedObject(_instance, libphx.Material_Free)
            end,
        }

        if onDef_Material then onDef_Material(Material, mt) end
        Material = setmetatable(Material, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Material')
        local mt = {
            __index = {
                getParamsSize = libphx.Material_GetParamsSize,
                setTexture    = libphx.Material_SetTexture,
                commit        = libphx.Material_Commit,
                getBlend      = libphx.Material_GetBlend,
            },
        }

        if onDef_Material_t then onDef_Material_t(t, mt) end
        Material_t = ffi.metatype(t, mt)
    end

    return Material
end

return Loader
