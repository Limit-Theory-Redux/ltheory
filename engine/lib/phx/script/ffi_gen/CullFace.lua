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
            CullFace CullFace_None;
            CullFace CullFace_Back;
            CullFace CullFace_Front;

            cstr     CullFace_ToString(CullFace);
        ]]
    end

    do -- Global Symbol Table
        CullFace = {
            None     = libphx.CullFace_None,
            Back     = libphx.CullFace_Back,
            Front    = libphx.CullFace_Front,

            ToString = libphx.CullFace_ToString,
        }

        if onDef_CullFace then onDef_CullFace(CullFace, mt) end
        CullFace = setmetatable(CullFace, mt)
    end

    return CullFace
end

return Loader
