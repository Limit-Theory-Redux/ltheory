-- TexCube ---------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct TexCube {} TexCube;
    ]]

    return 1, 'TexCube'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local TexCube

    do -- C Definitions
        ffi.cdef [[
            void      TexCube_Free         (TexCube*);
            TexCube*  TexCube_Create       (int size, TexFormat format);
            TexCube*  TexCube_Load         (cstr path);
            void      TexCube_Clear        (TexCube*, float r, float g, float b, float a);
            void      TexCube_Save         (TexCube*, cstr path);
            void      TexCube_SaveLevel    (TexCube*, cstr path, int level);
            Bytes*    TexCube_GetDataBytes (TexCube*, CubeFace* face, int level, PixelFormat pf, DataFormat df);
            TexFormat TexCube_GetFormat    (TexCube const*);
            uint32    TexCube_GetHandle    (TexCube const*);
            int       TexCube_GetSize      (TexCube const*);
            void      TexCube_Generate     (TexCube*, ShaderState* state);
            void      TexCube_GenMipmap    (TexCube*);
            void      TexCube_SetDataBytes (TexCube*, Bytes const* data, CubeFace* face, int level, PixelFormat pf, DataFormat df);
            void      TexCube_SetMagFilter (TexCube*, TexFilter filter);
            void      TexCube_SetMinFilter (TexCube*, TexFilter filter);
            TexCube*  TexCube_GenIRMap     (TexCube*, int sampleCount);
        ]]
    end

    do -- Global Symbol Table
        TexCube = {
            Create       = function(...)
                local instance = libphx.TexCube_Create(...)
                return Core.ManagedObject(instance, libphx.TexCube_Free)
            end,
            Load         = function(...)
                local instance = libphx.TexCube_Load(...)
                return Core.ManagedObject(instance, libphx.TexCube_Free)
            end,
        }

        if onDef_TexCube then onDef_TexCube(TexCube, mt) end
        TexCube = setmetatable(TexCube, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('TexCube')
        local mt = {
            __index = {
                clear        = libphx.TexCube_Clear,
                save         = libphx.TexCube_Save,
                saveLevel    = libphx.TexCube_SaveLevel,
                getDataBytes = function(...)
                    local instance = libphx.TexCube_GetDataBytes(...)
                    return Core.ManagedObject(instance, libphx.Bytes_Free)
                end,
                getFormat    = libphx.TexCube_GetFormat,
                getHandle    = libphx.TexCube_GetHandle,
                getSize      = libphx.TexCube_GetSize,
                generate     = libphx.TexCube_Generate,
                genMipmap    = libphx.TexCube_GenMipmap,
                setDataBytes = libphx.TexCube_SetDataBytes,
                setMagFilter = libphx.TexCube_SetMagFilter,
                setMinFilter = libphx.TexCube_SetMinFilter,
                genIRMap     = function(...)
                    local instance = libphx.TexCube_GenIRMap(...)
                    return Core.ManagedObject(instance, libphx.TexCube_Free)
                end,
            },
        }

        if onDef_TexCube_t then onDef_TexCube_t(t, mt) end
        TexCube_t = ffi.metatype(t, mt)
    end

    return TexCube
end

return Loader
