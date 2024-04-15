-- KeyboardState ---------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct KeyboardState {} KeyboardState;
    ]]

    return 1, 'KeyboardState'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local KeyboardState

    do -- C Definitions
        ffi.cdef [[
            void  KeyboardState_Free         (KeyboardState*);
            bool  KeyboardState_IsPressed    (KeyboardState const*, KeyboardButton button);
            bool  KeyboardState_IsDown       (KeyboardState const*, KeyboardButton button);
            bool  KeyboardState_IsReleased   (KeyboardState const*, KeyboardButton button);
            bool  KeyboardState_AltPressed   (KeyboardState const*);
            bool  KeyboardState_CtrlPressed  (KeyboardState const*);
            bool  KeyboardState_ShiftPressed (KeyboardState const*);
            float KeyboardState_Value        (KeyboardState const*, KeyboardButton button);
        ]]
    end

    do -- Global Symbol Table
        KeyboardState = {}

        if onDef_KeyboardState then onDef_KeyboardState(KeyboardState, mt) end
        KeyboardState = setmetatable(KeyboardState, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('KeyboardState')
        local mt = {
            __index = {
                ---@param button KeyboardButton
                ---@return bool
                isPressed    = libphx.KeyboardState_IsPressed,
                ---@param button KeyboardButton
                ---@return bool
                isDown       = libphx.KeyboardState_IsDown,
                ---@param button KeyboardButton
                ---@return bool
                isReleased   = libphx.KeyboardState_IsReleased,
                ---@return bool
                altPressed   = libphx.KeyboardState_AltPressed,
                ---@return bool
                ctrlPressed  = libphx.KeyboardState_CtrlPressed,
                ---@return bool
                shiftPressed = libphx.KeyboardState_ShiftPressed,
                ---@param button KeyboardButton
                ---@return float
                value        = libphx.KeyboardState_Value,
            },
        }

        if onDef_KeyboardState_t then onDef_KeyboardState_t(t, mt) end
        KeyboardState_t = ffi.metatype(t, mt)
    end

    return KeyboardState
end

return Loader
