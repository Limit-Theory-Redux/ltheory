-- CursorControl ---------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('ffi.libphx').lib
local CursorControl

do -- C Definitions
    ffi.cdef [[
        CursorControl CursorControl_X;
        CursorControl CursorControl_Y;
        CursorControl CursorControl_InWindow;
        cstr          CursorControl_ToString(CursorControl);
    ]]
end

do -- Global Symbol Table
    CursorControl = {
        X        = libphx.CursorControl_X,
        Y        = libphx.CursorControl_Y,
        InWindow = libphx.CursorControl_InWindow,
        ToString = libphx.CursorControl_ToString,
    }

    if onDef_CursorControl then onDef_CursorControl(CursorControl, mt) end
    CursorControl = setmetatable(CursorControl, mt)
end

return CursorControl
