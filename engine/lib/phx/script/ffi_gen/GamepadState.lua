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
        GamepadState = {}

        if onDef_GamepadState then onDef_GamepadState(GamepadState, mt) end
        GamepadState = setmetatable(GamepadState, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('GamepadState')
        local mt = {
            __index = {
                ---@return uint64
                gamepadsCount  = libphx.GamepadState_GamepadsCount,
                ---@param index uint64
                ---@return GamepadId const*
                gamepadId      = libphx.GamepadState_GamepadId,
                ---@param gamepad_id GamepadId
                ---@return cstr
                gamepadName    = libphx.GamepadState_GamepadName,
                ---@param axis GamepadAxis
                ---@return float
                value          = libphx.GamepadState_Value,
                ---@param button GamepadButton
                ---@return bool
                isPressed      = libphx.GamepadState_IsPressed,
                ---@param button GamepadButton
                ---@return bool
                isDown         = libphx.GamepadState_IsDown,
                ---@param button GamepadButton
                ---@return bool
                isReleased     = libphx.GamepadState_IsReleased,
                ---@param gamepad_id GamepadId
                ---@param axis GamepadAxis
                ---@return float
                valueById      = libphx.GamepadState_ValueById,
                ---@param gamepad_id GamepadId
                ---@param button GamepadButton
                ---@return bool
                isPressedById  = libphx.GamepadState_IsPressedById,
                ---@param gamepad_id GamepadId
                ---@param button GamepadButton
                ---@return bool
                isDownById     = libphx.GamepadState_IsDownById,
                ---@param gamepad_id GamepadId
                ---@param button GamepadButton
                ---@return bool
                isReleasedById = libphx.GamepadState_IsReleasedById,
            },
        }

        if onDef_GamepadState_t then onDef_GamepadState_t(t, mt) end
        GamepadState_t = ffi.metatype(t, mt)
    end

    return GamepadState
end

return Loader
