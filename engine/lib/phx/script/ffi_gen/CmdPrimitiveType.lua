-- AUTO GENERATED. DO NOT MODIFY!
-- CmdPrimitiveType ------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 CmdPrimitiveType;
    ]]

    return 2, 'CmdPrimitiveType'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local CmdPrimitiveType

    do -- C Definitions
        ffi.cdef [[
            cstr             CmdPrimitiveType_ToString(CmdPrimitiveType);
        ]]
    end

    do -- Global Symbol Table
        CmdPrimitiveType = {
            Points        = 0,
            Lines         = 1,
            LineStrip     = 2,
            Triangles     = 3,
            TriangleStrip = 4,
            TriangleFan   = 5,
            Quads         = 6,

            ToString      = libphx.CmdPrimitiveType_ToString,
        }

        if onDef_CmdPrimitiveType then onDef_CmdPrimitiveType(CmdPrimitiveType, mt) end
        CmdPrimitiveType = setmetatable(CmdPrimitiveType, mt)
    end

    return CmdPrimitiveType
end

return Loader
