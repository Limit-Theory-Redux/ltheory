-- GamepadAxis -----------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('libphx').lib
local GamepadAxis

do -- C Definitions
    ffi.cdef [[
        GamepadAxis GamepadAxis_LeftStickX;
        GamepadAxis GamepadAxis_LeftStickY;
        GamepadAxis GamepadAxis_LeftZ;
        GamepadAxis GamepadAxis_RightStickX;
        GamepadAxis GamepadAxis_RightStickY;
        GamepadAxis GamepadAxis_RightZ;

        cstr        GamepadAxis_ToString(GamepadAxis);
    ]]
end

do -- Global Symbol Table
    GamepadAxis = {
        LeftStickX  = libphx.GamepadAxis_LeftStickX,
        LeftStickY  = libphx.GamepadAxis_LeftStickY,
        LeftZ       = libphx.GamepadAxis_LeftZ,
        RightStickX = libphx.GamepadAxis_RightStickX,
        RightStickY = libphx.GamepadAxis_RightStickY,
        RightZ      = libphx.GamepadAxis_RightZ,

        ToString    = libphx.GamepadAxis_ToString,
    }

    if onDef_GamepadAxis then onDef_GamepadAxis(GamepadAxis, mt) end
    GamepadAxis = setmetatable(GamepadAxis, mt)
end

return GamepadAxis
