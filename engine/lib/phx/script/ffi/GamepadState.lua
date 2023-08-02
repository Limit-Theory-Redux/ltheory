-- GamepadState ----------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('ffi.libphx').lib
local GamepadState

do -- C Definitions
    ffi.cdef [[
        typedef struct GamepadState {} GamepadState;

        uint64     GamepadState_GetGamepadsCount (GamepadState const*);
        GamepadId* GamepadState_GetGamepadId     (GamepadState const*, uint64 index);
        float      GamepadState_GetButtonValue   (GamepadState const*, GamepadId gamepadId, GamepadButton button);
        float      GamepadState_GetAxisValue     (GamepadState const*, GamepadId gamepadId, GamepadAxis axis);
    ]]
end

do -- Global Symbol Table
    GamepadState = {
        GetGamepadsCount = libphx.GamepadState_GetGamepadsCount,
        GetGamepadId     = libphx.GamepadState_GetGamepadId,
        GetButtonValue   = libphx.GamepadState_GetButtonValue,
        GetAxisValue     = libphx.GamepadState_GetAxisValue,
    }

    if onDef_GamepadState then onDef_GamepadState(GamepadState, mt) end
    GamepadState = setmetatable(GamepadState, mt)
end

do -- Metatype for class instances
    local t  = ffi.typeof('GamepadState')
    local mt = {
        __index = {
            getGamepadsCount = libphx.GamepadState_GetGamepadsCount,
            getGamepadId     = libphx.GamepadState_GetGamepadId,
            getButtonValue   = libphx.GamepadState_GetButtonValue,
            getAxisValue     = libphx.GamepadState_GetAxisValue,
        },
    }

    if onDef_GamepadState_t then onDef_GamepadState_t(t, mt) end
    GamepadState_t = ffi.metatype(t, mt)
end

return GamepadState
