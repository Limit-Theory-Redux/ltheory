-- AUTO GENERATED. DO NOT MODIFY!
-- CubeFace --------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    return 0, 'CubeFace'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local CubeFace

    do -- C Definitions
        ffi.cdef [[
            CubeFace CubeFace_Get (int index);
        ]]
    end

    do -- Global Symbol Table
        CubeFace = {
            Get = libphx.CubeFace_Get,
        }

        if onDef_CubeFace then onDef_CubeFace(CubeFace, mt) end
        CubeFace = setmetatable(CubeFace, mt)
    end

    return CubeFace
end

return Loader
