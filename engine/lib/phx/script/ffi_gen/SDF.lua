-- AUTO GENERATED. DO NOT MODIFY!
-- SDF -------------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct SDF {} SDF;
    ]]

    return 1, 'SDF'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local SDF

    do -- C Definitions
        ffi.cdef [[
            void  SDF_Free           (SDF*);
            SDF*  SDF_Create         (int sx, int sy, int sz);
            SDF*  SDF_FromTex3D      (Tex3D* tex);
            Mesh* SDF_ToMesh         (SDF const*);
            void  SDF_Clear          (SDF*, float value);
            void  SDF_ComputeNormals (SDF*);
            void  SDF_Set            (SDF*, int x, int y, int z, float value);
            void  SDF_SetNormal      (SDF*, int x, int y, int z, Vec3f const* normal);
        ]]
    end

    do -- Global Symbol Table
        SDF = {
            Create         = function(sx, sy, sz)
                local _instance = libphx.SDF_Create(sx, sy, sz)
                return Core.ManagedObject(_instance, libphx.SDF_Free)
            end,
            FromTex3D      = function(tex)
                local _instance = libphx.SDF_FromTex3D(tex)
                return Core.ManagedObject(_instance, libphx.SDF_Free)
            end,
        }

        if onDef_SDF then onDef_SDF(SDF, mt) end
        SDF = setmetatable(SDF, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('SDF')
        local mt = {
            __index = {
                toMesh         = function(self)
                    local _instance = libphx.SDF_ToMesh(self)
                    return Core.ManagedObject(_instance, libphx.Mesh_Free)
                end,
                clear          = libphx.SDF_Clear,
                computeNormals = libphx.SDF_ComputeNormals,
                set            = libphx.SDF_Set,
                setNormal      = libphx.SDF_SetNormal,
            },
        }

        if onDef_SDF_t then onDef_SDF_t(t, mt) end
        SDF_t = ffi.metatype(t, mt)
    end

    return SDF
end

return Loader
