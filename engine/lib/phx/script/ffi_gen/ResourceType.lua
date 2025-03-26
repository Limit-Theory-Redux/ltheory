-- AUTO GENERATED. DO NOT MODIFY!
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
            cstr         ResourceType_ToString(ResourceType);
        ]]
    end

    do -- Global Symbol Table
        ResourceType = {
            Font     = 0,
            Mesh     = 1,
            Script   = 2,
            Shader   = 3,
            Sound    = 4,
            Tex1D    = 5,
            Tex2D    = 6,
            Tex3D    = 7,
            TexCube  = 8,
            Theme    = 9,
            Other    = 10,

            ToString = libphx.ResourceType_ToString,
        }

        if onDef_ResourceType then onDef_ResourceType(ResourceType, mt) end
        ResourceType = setmetatable(ResourceType, mt)
    end

    return ResourceType
end

return Loader
