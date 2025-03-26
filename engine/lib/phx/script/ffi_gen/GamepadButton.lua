-- AUTO GENERATED. DO NOT MODIFY!
-- GamepadButton ---------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 GamepadButton;
    ]]

    return 2, 'GamepadButton'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local GamepadButton

    do -- C Definitions
        ffi.cdef [[
            cstr          GamepadButton_ToString(GamepadButton);
        ]]
    end

    do -- Global Symbol Table
        GamepadButton = {
            South         = 0,
            East          = 1,
            North         = 2,
            West          = 3,
            C             = 4,
            Z             = 5,
            LeftTrigger   = 6,
            LeftTrigger2  = 7,
            RightTrigger  = 8,
            RightTrigger2 = 9,
            Select        = 10,
            Start         = 11,
            Mode          = 12,
            LeftThumb     = 13,
            RightThumb    = 14,
            DPadUp        = 15,
            DPadDown      = 16,
            DPadLeft      = 17,
            DPadRight     = 18,

            ToString      = libphx.GamepadButton_ToString,
        }

        if onDef_GamepadButton then onDef_GamepadButton(GamepadButton, mt) end
        GamepadButton = setmetatable(GamepadButton, mt)
    end

    return GamepadButton
end

return Loader
