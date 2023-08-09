-- GamepadState ----------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('ffi.libphx').lib
local GamepadState

do -- C Definitions
    ffi.cdef [[
        typedef struct GamepadState {} GamepadState;

        uint64     GamepadState_GamepadsCount (GamepadState const*);
        GamepadId* GamepadState_GamepadId     (GamepadState const*, uint64 index);
        cstr       GamepadState_GamepadName   (GamepadState const*, GamepadId gamepadId);
        float      GamepadState_Value         (GamepadState const*, GamepadId gamepadId, GamepadAxis axis);
        bool       GamepadState_IsPressed     (GamepadState const*, GamepadId gamepadId, GamepadButton button);
        bool       GamepadState_IsDown        (GamepadState const*, GamepadId gamepadId, GamepadButton button);
        bool       GamepadState_IsReleased    (GamepadState const*, GamepadId gamepadId, GamepadButton button);
    ]]
end

do -- Global Symbol Table
    GamepadState = {
        GamepadsCount = libphx.GamepadState_GamepadsCount,
        GamepadId     = libphx.GamepadState_GamepadId,
        GamepadName   = libphx.GamepadState_GamepadName,
        Value         = libphx.GamepadState_Value,
        IsPressed     = libphx.GamepadState_IsPressed,
        IsDown        = libphx.GamepadState_IsDown,
        IsReleased    = libphx.GamepadState_IsReleased,
    }

    if onDef_GamepadState then onDef_GamepadState(GamepadState, mt) end
    GamepadState = setmetatable(GamepadState, mt)
end

do -- Metatype for class instances
    local t  = ffi.typeof('GamepadState')
    local mt = {
        __index = {
            gamepadsCount = libphx.GamepadState_GamepadsCount,
            gamepadId     = libphx.GamepadState_GamepadId,
            gamepadName   = libphx.GamepadState_GamepadName,
            value         = libphx.GamepadState_Value,
            isPressed     = libphx.GamepadState_IsPressed,
            isDown        = libphx.GamepadState_IsDown,
            isReleased    = libphx.GamepadState_IsReleased,
        },
    }

    if onDef_GamepadState_t then onDef_GamepadState_t(t, mt) end
    GamepadState_t = ffi.metatype(t, mt)
end

return GamepadState
