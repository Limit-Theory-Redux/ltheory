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
            bool                    Input_IsKeyboardAltDown      (Input const*);
            bool                    Input_IsKeyboardCtrlPressed  (Input const*);
            bool                    Input_IsKeyboardCtrlDown     (Input const*);
            bool                    Input_IsKeyboardShiftPressed (Input const*);
            bool                    Input_IsKeyboardShiftDown    (Input const*);
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
                isKeyboardAltDown      = libphx.Input_IsKeyboardAltDown,
                isKeyboardCtrlPressed  = libphx.Input_IsKeyboardCtrlPressed,
                isKeyboardCtrlDown     = libphx.Input_IsKeyboardCtrlDown,
                isKeyboardShiftPressed = libphx.Input_IsKeyboardShiftPressed,
                isKeyboardShiftDown    = libphx.Input_IsKeyboardShiftDown,
            },
        }

        if onDef_Input_t then onDef_Input_t(t, mt) end
        Input_t = ffi.metatype(t, mt)
    end

    return Input
end

return Loader
