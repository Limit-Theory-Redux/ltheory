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
                ---@return integer
                gamepadsCount  = libphx.GamepadState_GamepadsCount,
                ---@param index integer
                ---@return GamepadId
                gamepadId      = libphx.GamepadState_GamepadId,
                ---@param gamepad_id GamepadId
                ---@return string
                gamepadName    = libphx.GamepadState_GamepadName,
                ---@param axis GamepadAxis
                ---@return number
                value          = libphx.GamepadState_Value,
                ---@param button GamepadButton
                ---@return boolean
                isPressed      = libphx.GamepadState_IsPressed,
                ---@param button GamepadButton
                ---@return boolean
                isDown         = libphx.GamepadState_IsDown,
                ---@param button GamepadButton
                ---@return boolean
                isReleased     = libphx.GamepadState_IsReleased,
                ---@param gamepad_id GamepadId
                ---@param axis GamepadAxis
                ---@return number
                valueById      = libphx.GamepadState_ValueById,
                ---@param gamepad_id GamepadId
                ---@param button GamepadButton
                ---@return boolean
                isPressedById  = libphx.GamepadState_IsPressedById,
                ---@param gamepad_id GamepadId
                ---@param button GamepadButton
                ---@return boolean
                isDownById     = libphx.GamepadState_IsDownById,
                ---@param gamepad_id GamepadId
                ---@param button GamepadButton
                ---@return boolean
                isReleasedById = libphx.GamepadState_IsReleasedById,
            },
        }

        if onDef_GamepadState_t then onDef_GamepadState_t(t, mt) end
        GamepadState_t = ffi.metatype(t, mt)
    end

    return GamepadState
end

return Loader
