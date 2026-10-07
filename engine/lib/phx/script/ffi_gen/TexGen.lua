-- AUTO GENERATED. DO NOT MODIFY!
-- TexGen ----------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    return 0, 'TexGen'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local TexGen

    do -- C Definitions
        ffi.cdef [[
            TexCube* TexGen_Cube     (Renderer* r, GenDesc const* desc, int size, TexFormat format);
            void     TexGen_CubeInto (Renderer* r, GenDesc const* desc, TexCube const* cube);
            Tex3D*   TexGen_Volume   (Renderer* r, GenDesc const* desc, int size, TexFormat format);
        ]]
    end

    do -- Global Symbol Table
        TexGen = {
            Cube     = function(r, desc, size, format)
                local _instance = libphx.TexGen_Cube(r, desc, size, format)
                return Core.ManagedObject(_instance, libphx.TexCube_Free)
            end,
            CubeInto = libphx.TexGen_CubeInto,
            Volume   = function(r, desc, size, format)
                local _instance = libphx.TexGen_Volume(r, desc, size, format)
                return Core.ManagedObject(_instance, libphx.Tex3D_Free)
            end,
        }

        if onDef_TexGen then onDef_TexGen(TexGen, mt) end
        TexGen = setmetatable(TexGen, mt)
    end

    return TexGen
end

return Loader
