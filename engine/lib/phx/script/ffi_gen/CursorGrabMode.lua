-- CursorGrabMode --------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('libphx').lib
local CursorGrabMode

function declareType()
    ffi.cdef [[
        typedef uint8 CursorGrabMode;
    ]]

    return 2, 'CursorGrabMode'
end

do -- C Definitions
    ffi.cdef [[
        CursorGrabMode CursorGrabMode_None;
        CursorGrabMode CursorGrabMode_Confined;
        CursorGrabMode CursorGrabMode_Locked;

        cstr           CursorGrabMode_ToString(CursorGrabMode);
    ]]
end

do -- Global Symbol Table
    CursorGrabMode = {
        None     = libphx.CursorGrabMode_None,
        Confined = libphx.CursorGrabMode_Confined,
        Locked   = libphx.CursorGrabMode_Locked,

        ToString = libphx.CursorGrabMode_ToString,
    }

    if onDef_CursorGrabMode then onDef_CursorGrabMode(CursorGrabMode, mt) end
    CursorGrabMode = setmetatable(CursorGrabMode, mt)
end

return CursorGrabMode
