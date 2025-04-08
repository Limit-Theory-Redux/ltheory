-- AUTO GENERATED. DO NOT MODIFY!
-- CubeFace --------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint16 CubeFace;
    ]]

    return 2, 'CubeFace'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local CubeFace

    do -- C Definitions
        ffi.cdef [[
            cstr     CubeFace_ToString(CubeFace);

            CubeFace CubeFace_Get (int index);
        ]]
    end

    do -- Global Symbol Table
        CubeFace = {
            PX       = 34069,
            NX       = 34070,
            PY       = 34071,
            NY       = 34072,
            PZ       = 34073,
            NZ       = 34074,

            ToString = libphx.CubeFace_ToString,

            Get = libphx.CubeFace_Get,
        }

        if onDef_CubeFace then onDef_CubeFace(CubeFace, mt) end
        CubeFace = setmetatable(CubeFace, mt)
    end

    return CubeFace
end

return Loader
