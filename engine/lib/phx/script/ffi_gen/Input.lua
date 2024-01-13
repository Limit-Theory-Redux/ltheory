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
        Input = {
            Keyboard               = libphx.Input_Keyboard,
            Mouse                  = libphx.Input_Mouse,
            Touchpad               = libphx.Input_Touchpad,
            Gamepad                = libphx.Input_Gamepad,
            DragAndDrop            = libphx.Input_DragAndDrop,
            ActiveDevice           = libphx.Input_ActiveDevice,
            ActiveDeviceType       = libphx.Input_ActiveDeviceType,
            ActiveDeviceId         = libphx.Input_ActiveDeviceId,
            SetCursorVisible       = libphx.Input_SetCursorVisible,
            SetCursorVisibleAuto   = libphx.Input_SetCursorVisibleAuto,
            SetCursorPosition      = libphx.Input_SetCursorPosition,
            IsPressed              = libphx.Input_IsPressed,
            IsDown                 = libphx.Input_IsDown,
            IsReleased             = libphx.Input_IsReleased,
            GetValue               = libphx.Input_GetValue,
            IsKeyboardAltPressed   = libphx.Input_IsKeyboardAltPressed,
            IsKeyboardCtrlPressed  = libphx.Input_IsKeyboardCtrlPressed,
            IsKeyboardShiftPressed = libphx.Input_IsKeyboardShiftPressed,
        }

        if onDef_Input then onDef_Input(Input, mt) end
        Input = setmetatable(Input, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Input')
        local mt = {
            __index = {
                keyboard               = libphx.Input_Keyboard,
                mouse                  = libphx.Input_Mouse,
                touchpad               = libphx.Input_Touchpad,
                gamepad                = libphx.Input_Gamepad,
                dragAndDrop            = libphx.Input_DragAndDrop,
                activeDevice           = libphx.Input_ActiveDevice,
                activeDeviceType       = libphx.Input_ActiveDeviceType,
                activeDeviceId         = libphx.Input_ActiveDeviceId,
                setCursorVisible       = libphx.Input_SetCursorVisible,
                setCursorVisibleAuto   = libphx.Input_SetCursorVisibleAuto,
                setCursorPosition      = libphx.Input_SetCursorPosition,
                isPressed              = libphx.Input_IsPressed,
                isDown                 = libphx.Input_IsDown,
                isReleased             = libphx.Input_IsReleased,
                getValue               = libphx.Input_GetValue,
                isKeyboardAltPressed   = libphx.Input_IsKeyboardAltPressed,
                isKeyboardCtrlPressed  = libphx.Input_IsKeyboardCtrlPressed,
                isKeyboardShiftPressed = libphx.Input_IsKeyboardShiftPressed,
            },
        }

        if onDef_Input_t then onDef_Input_t(t, mt) end
        Input_t = ffi.metatype(t, mt)
    end

    return Input
end

return Loader
