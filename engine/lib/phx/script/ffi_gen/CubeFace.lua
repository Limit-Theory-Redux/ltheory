-- AUTO GENERATED. DO NOT MODIFY!
-- CubeFace --------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint32 CubeFace;
    ]]

    return 2, 'CubeFace'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local CubeFace

    do -- C Definitions
        ffi.cdef [[
            CubeFace CubeFace_PX;
            CubeFace CubeFace_NX;
            CubeFace CubeFace_PY;
            CubeFace CubeFace_NY;
            CubeFace CubeFace_PZ;
            CubeFace CubeFace_NZ;

            cstr     CubeFace_ToString(CubeFace);

            CubeFace CubeFace_Get (int index);
        ]]
    end

    do -- Global Symbol Table
        CubeFace = {
            PX       = libphx.CubeFace_PX,
            NX       = libphx.CubeFace_NX,
            PY       = libphx.CubeFace_PY,
            NY       = libphx.CubeFace_NY,
            PZ       = libphx.CubeFace_PZ,
            NZ       = libphx.CubeFace_NZ,

            ToString = libphx.CubeFace_ToString,

            Get = libphx.CubeFace_Get,
        }

        if onDef_CubeFace then onDef_CubeFace(CubeFace, mt) end
        CubeFace = setmetatable(CubeFace, mt)
    end

    return CubeFace
end

return Loader
