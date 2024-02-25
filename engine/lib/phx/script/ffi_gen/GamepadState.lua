-- GamepadState ----------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct GamepadState {} GamepadState;
    ]]

    return 1, 'GamepadState'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local GamepadState

    do -- C Definitions
        ffi.cdef [[
            void             GamepadState_Free           (GamepadState*);
            uint64           GamepadState_GamepadsCount  (GamepadState const*);
            GamepadId const* GamepadState_GamepadId      (GamepadState const*, uint64 index);
            cstr             GamepadState_GamepadName    (GamepadState const*, GamepadId gamepadId);
            float            GamepadState_Value          (GamepadState const*, GamepadAxis axis);
            bool             GamepadState_IsPressed      (GamepadState const*, GamepadButton button);
            bool             GamepadState_IsDown         (GamepadState const*, GamepadButton button);
            bool             GamepadState_IsReleased     (GamepadState const*, GamepadButton button);
            float            GamepadState_ValueById      (GamepadState const*, GamepadId gamepadId, GamepadAxis axis);
            bool             GamepadState_IsPressedById  (GamepadState const*, GamepadId gamepadId, GamepadButton button);
            bool             GamepadState_IsDownById     (GamepadState const*, GamepadId gamepadId, GamepadButton button);
            bool             GamepadState_IsReleasedById (GamepadState const*, GamepadId gamepadId, GamepadButton button);
        ]]
    end

    do -- Global Symbol Table
        GamepadState = {
        }

        if onDef_GamepadState then onDef_GamepadState(GamepadState, mt) end
        GamepadState = setmetatable(GamepadState, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('GamepadState')
        local mt = {
            __index = {
                gamepadsCount  = libphx.GamepadState_GamepadsCount,
                gamepadId      = libphx.GamepadState_GamepadId,
                gamepadName    = libphx.GamepadState_GamepadName,
                value          = libphx.GamepadState_Value,
                isPressed      = libphx.GamepadState_IsPressed,
                isDown         = libphx.GamepadState_IsDown,
                isReleased     = libphx.GamepadState_IsReleased,
                valueById      = libphx.GamepadState_ValueById,
                isPressedById  = libphx.GamepadState_IsPressedById,
                isDownById     = libphx.GamepadState_IsDownById,
                isReleasedById = libphx.GamepadState_IsReleasedById,
            },
        }

        if onDef_GamepadState_t then onDef_GamepadState_t(t, mt) end
        GamepadState_t = ffi.metatype(t, mt)
    end

    return GamepadState
end

return Loader
