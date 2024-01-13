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
        KeyboardState = {
            IsPressed    = libphx.KeyboardState_IsPressed,
            IsDown       = libphx.KeyboardState_IsDown,
            IsReleased   = libphx.KeyboardState_IsReleased,
            AltPressed   = libphx.KeyboardState_AltPressed,
            CtrlPressed  = libphx.KeyboardState_CtrlPressed,
            ShiftPressed = libphx.KeyboardState_ShiftPressed,
            Value        = libphx.KeyboardState_Value,
        }

        if onDef_KeyboardState then onDef_KeyboardState(KeyboardState, mt) end
        KeyboardState = setmetatable(KeyboardState, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('KeyboardState')
        local mt = {
            __index = {
                isPressed    = libphx.KeyboardState_IsPressed,
                isDown       = libphx.KeyboardState_IsDown,
                isReleased   = libphx.KeyboardState_IsReleased,
                altPressed   = libphx.KeyboardState_AltPressed,
                ctrlPressed  = libphx.KeyboardState_CtrlPressed,
                shiftPressed = libphx.KeyboardState_ShiftPressed,
                value        = libphx.KeyboardState_Value,
            },
        }

        if onDef_KeyboardState_t then onDef_KeyboardState_t(t, mt) end
        KeyboardState_t = ffi.metatype(t, mt)
    end

    return KeyboardState
end

return Loader
