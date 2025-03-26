-- AUTO GENERATED. DO NOT MODIFY!
-- GamepadAxis -----------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 GamepadAxis;
    ]]

    return 2, 'GamepadAxis'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local GamepadAxis

    do -- C Definitions
        ffi.cdef [[
            cstr        GamepadAxis_ToString(GamepadAxis);
        ]]
    end

    do -- Global Symbol Table
        GamepadAxis = {
            LeftStickX  = 0,
            LeftStickY  = 1,
            LeftZ       = 2,
            RightStickX = 3,
            RightStickY = 4,
            RightZ      = 5,

            ToString    = libphx.GamepadAxis_ToString,
        }

        if onDef_GamepadAxis then onDef_GamepadAxis(GamepadAxis, mt) end
        GamepadAxis = setmetatable(GamepadAxis, mt)
    end

    return GamepadAxis
end

return Loader
