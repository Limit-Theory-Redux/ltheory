-- ResourceType ----------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 ResourceType;
    ]]

    return 2, 'ResourceType'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local ResourceType

    do -- C Definitions
        ffi.cdef [[
            ResourceType ResourceType_Font;
            ResourceType ResourceType_Mesh;
            ResourceType ResourceType_Script;
            ResourceType ResourceType_Shader;
            ResourceType ResourceType_Sound;
            ResourceType ResourceType_Tex1D;
            ResourceType ResourceType_Tex2D;
            ResourceType ResourceType_Tex3D;
            ResourceType ResourceType_TexCube;
            ResourceType ResourceType_Other;

            cstr         ResourceType_ToString(ResourceType);
        ]]
    end

    do -- Global Symbol Table
        ResourceType = {
            Font     = libphx.ResourceType_Font,
            Mesh     = libphx.ResourceType_Mesh,
            Script   = libphx.ResourceType_Script,
            Shader   = libphx.ResourceType_Shader,
            Sound    = libphx.ResourceType_Sound,
            Tex1D    = libphx.ResourceType_Tex1D,
            Tex2D    = libphx.ResourceType_Tex2D,
            Tex3D    = libphx.ResourceType_Tex3D,
            TexCube  = libphx.ResourceType_TexCube,
            Other    = libphx.ResourceType_Other,

            ToString = libphx.ResourceType_ToString,
        }

        if onDef_ResourceType then onDef_ResourceType(ResourceType, mt) end
        ResourceType = setmetatable(ResourceType, mt)
    end

    return ResourceType
end

return Loader
