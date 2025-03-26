-- AUTO GENERATED. DO NOT MODIFY!
-- CullFace --------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 CullFace;
    ]]

    return 2, 'CullFace'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local CullFace

    do -- C Definitions
        ffi.cdef [[
            cstr     CullFace_ToString(CullFace);
        ]]
    end

    do -- Global Symbol Table
        CullFace = {
            None     = 0,
            Back     = 1,
            Front    = 2,

            ToString = libphx.CullFace_ToString,
        }

        if onDef_CullFace then onDef_CullFace(CullFace, mt) end
        CullFace = setmetatable(CullFace, mt)
    end

    return CullFace
end

return Loader
