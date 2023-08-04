-- GamepadButton ---------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('ffi.libphx').lib
local GamepadButton

do -- C Definitions
    ffi.cdef [[
        typedef uint8 GamepadButton;

        GamepadButton GamepadButton_South;
        GamepadButton GamepadButton_East;
        GamepadButton GamepadButton_North;
        GamepadButton GamepadButton_West;
        GamepadButton GamepadButton_C;
        GamepadButton GamepadButton_Z;
        GamepadButton GamepadButton_LeftTrigger;
        GamepadButton GamepadButton_LeftTrigger2;
        GamepadButton GamepadButton_RightTrigger;
        GamepadButton GamepadButton_RightTrigger2;
        GamepadButton GamepadButton_Select;
        GamepadButton GamepadButton_Start;
        GamepadButton GamepadButton_Mode;
        GamepadButton GamepadButton_LeftThumb;
        GamepadButton GamepadButton_RightThumb;
        GamepadButton GamepadButton_DPadUp;
        GamepadButton GamepadButton_DPadDown;
        GamepadButton GamepadButton_DPadLeft;
        GamepadButton GamepadButton_DPadRight;

        cstr          GamepadButton_ToString(GamepadButton);
    ]]
end

do -- Global Symbol Table
    GamepadButton = {
        South         = libphx.GamepadButton_South,
        East          = libphx.GamepadButton_East,
        North         = libphx.GamepadButton_North,
        West          = libphx.GamepadButton_West,
        C             = libphx.GamepadButton_C,
        Z             = libphx.GamepadButton_Z,
        LeftTrigger   = libphx.GamepadButton_LeftTrigger,
        LeftTrigger2  = libphx.GamepadButton_LeftTrigger2,
        RightTrigger  = libphx.GamepadButton_RightTrigger,
        RightTrigger2 = libphx.GamepadButton_RightTrigger2,
        Select        = libphx.GamepadButton_Select,
        Start         = libphx.GamepadButton_Start,
        Mode          = libphx.GamepadButton_Mode,
        LeftThumb     = libphx.GamepadButton_LeftThumb,
        RightThumb    = libphx.GamepadButton_RightThumb,
        DPadUp        = libphx.GamepadButton_DPadUp,
        DPadDown      = libphx.GamepadButton_DPadDown,
        DPadLeft      = libphx.GamepadButton_DPadLeft,
        DPadRight     = libphx.GamepadButton_DPadRight,

        ToString      = libphx.GamepadButton_ToString,
    }

    if onDef_GamepadButton then onDef_GamepadButton(GamepadButton, mt) end
    GamepadButton = setmetatable(GamepadButton, mt)
end

return GamepadButton
