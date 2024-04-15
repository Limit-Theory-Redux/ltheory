-- Input -----------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct Input {} Input;
    ]]

    return 1, 'Input'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Input

    do -- C Definitions
        ffi.cdef [[
            void                    Input_Free                   (Input*);
            KeyboardState const*    Input_Keyboard               (Input const*);
            MouseState const*       Input_Mouse                  (Input const*);
            TouchpadState const*    Input_Touchpad               (Input const*);
            GamepadState const*     Input_Gamepad                (Input const*);
            DragAndDropState const* Input_DragAndDrop            (Input const*);
            InputDevice const*      Input_ActiveDevice           (Input const*);
            InputDeviceType const*  Input_ActiveDeviceType       (Input const*);
            InputDeviceId const*    Input_ActiveDeviceId         (Input const*);
            void                    Input_SetCursorVisible       (Input*, bool visible);
            void                    Input_SetCursorVisibleAuto   (Input*);
            void                    Input_SetCursorPosition      (Input*, float x, float y);
            bool                    Input_IsPressed              (Input const*, Button button);
            bool                    Input_IsDown                 (Input const*, Button button);
            bool                    Input_IsReleased             (Input const*, Button button);
            float                   Input_GetValue               (Input const*, Button button);
            bool                    Input_IsKeyboardAltPressed   (Input const*);
            bool                    Input_IsKeyboardCtrlPressed  (Input const*);
            bool                    Input_IsKeyboardShiftPressed (Input const*);
        ]]
    end

    do -- Global Symbol Table
        Input = {}

        if onDef_Input then onDef_Input(Input, mt) end
        Input = setmetatable(Input, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Input')
        local mt = {
            __index = {
                ---@return KeyboardState
                keyboard               = libphx.Input_Keyboard,
                ---@return MouseState
                mouse                  = libphx.Input_Mouse,
                ---@return TouchpadState
                touchpad               = libphx.Input_Touchpad,
                ---@return GamepadState
                gamepad                = libphx.Input_Gamepad,
                ---@return DragAndDropState
                dragAndDrop            = libphx.Input_DragAndDrop,
                ---@return InputDevice
                activeDevice           = libphx.Input_ActiveDevice,
                ---@return InputDeviceType
                activeDeviceType       = libphx.Input_ActiveDeviceType,
                ---@return InputDeviceId
                activeDeviceId         = libphx.Input_ActiveDeviceId,
                ---@param visible boolean
                setCursorVisible       = libphx.Input_SetCursorVisible,
                setCursorVisibleAuto   = libphx.Input_SetCursorVisibleAuto,
                ---@param x number
                ---@param y number
                setCursorPosition      = libphx.Input_SetCursorPosition,
                ---@param button Button
                ---@return boolean
                isPressed              = libphx.Input_IsPressed,
                ---@param button Button
                ---@return boolean
                isDown                 = libphx.Input_IsDown,
                ---@param button Button
                ---@return boolean
                isReleased             = libphx.Input_IsReleased,
                ---@param button Button
                ---@return number
                getValue               = libphx.Input_GetValue,
                ---@return boolean
                isKeyboardAltPressed   = libphx.Input_IsKeyboardAltPressed,
                ---@return boolean
                isKeyboardCtrlPressed  = libphx.Input_IsKeyboardCtrlPressed,
                ---@return boolean
                isKeyboardShiftPressed = libphx.Input_IsKeyboardShiftPressed,
            },
        }

        if onDef_Input_t then onDef_Input_t(t, mt) end
        Input_t = ffi.metatype(t, mt)
    end

    return Input
end

return Loader
