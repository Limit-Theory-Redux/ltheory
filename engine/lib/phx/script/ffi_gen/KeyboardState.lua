-- KeyboardState ---------------------------------------------------------------

---@class KeyboardState
---@field IsPressed fun(self, button: KeyboardButton): boolean
---@field IsDown fun(self, button: KeyboardButton): boolean
---@field IsReleased fun(self, button: KeyboardButton): boolean
---@field AltPressed fun(self): boolean
---@field CtrlPressed fun(self): boolean
---@field ShiftPressed fun(self): boolean
---@field Value fun(self, button: KeyboardButton): number

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
                ---@return boolean
                isPressed    = libphx.KeyboardState_IsPressed,
                ---@param button KeyboardButton
                ---@return boolean
                isDown       = libphx.KeyboardState_IsDown,
                ---@param button KeyboardButton
                ---@return boolean
                isReleased   = libphx.KeyboardState_IsReleased,
                ---@return boolean
                altPressed   = libphx.KeyboardState_AltPressed,
                ---@return boolean
                ctrlPressed  = libphx.KeyboardState_CtrlPressed,
                ---@return boolean
                shiftPressed = libphx.KeyboardState_ShiftPressed,
                ---@param button KeyboardButton
                ---@return number
                value        = libphx.KeyboardState_Value,
            },
        }

        if onDef_KeyboardState_t then onDef_KeyboardState_t(t, mt) end
        KeyboardState_t = ffi.metatype(t, mt)
    end

    return KeyboardState
end

return Loader
