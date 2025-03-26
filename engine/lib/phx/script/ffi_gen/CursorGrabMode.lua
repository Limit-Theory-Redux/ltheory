-- AUTO GENERATED. DO NOT MODIFY!
-- CursorGrabMode --------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 CursorGrabMode;
    ]]

    return 2, 'CursorGrabMode'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local CursorGrabMode

    do -- C Definitions
        ffi.cdef [[
            cstr           CursorGrabMode_ToString(CursorGrabMode);
        ]]
    end

    do -- Global Symbol Table
        CursorGrabMode = {
            None     = 0,
            Confined = 1,
            Locked   = 2,

            ToString = libphx.CursorGrabMode_ToString,
        }

        if onDef_CursorGrabMode then onDef_CursorGrabMode(CursorGrabMode, mt) end
        CursorGrabMode = setmetatable(CursorGrabMode, mt)
    end

    return CursorGrabMode
end

return Loader
