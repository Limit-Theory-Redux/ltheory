-- InputImpl -------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct InputImpl {} InputImpl;
    ]]

    return 1, 'InputImpl'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local InputImpl

    do -- C Definitions
        ffi.cdef [[
            void                    InputImpl_Free                   (InputImpl*);
            KeyboardState const*    InputImpl_Keyboard               (InputImpl const*);
            MouseState const*       InputImpl_Mouse                  (InputImpl const*);
            TouchpadState const*    InputImpl_Touchpad               (InputImpl const*);
            GamepadState const*     InputImpl_Gamepad                (InputImpl const*);
            DragAndDropState const* InputImpl_DragAndDrop            (InputImpl const*);
            InputDevice const*      InputImpl_ActiveDevice           (InputImpl const*);
            InputDeviceType const*  InputImpl_ActiveDeviceType       (InputImpl const*);
            InputDeviceId const*    InputImpl_ActiveDeviceId         (InputImpl const*);
            void                    InputImpl_SetCursorVisible       (InputImpl*, bool visible);
            void                    InputImpl_SetCursorVisibleAuto   (InputImpl*);
            void                    InputImpl_SetCursorPosition      (InputImpl*, float x, float y);
            bool                    InputImpl_IsPressed              (InputImpl const*, Button button);
            bool                    InputImpl_IsDown                 (InputImpl const*, Button button);
            bool                    InputImpl_IsReleased             (InputImpl const*, Button button);
            float                   InputImpl_GetValue               (InputImpl const*, Button button);
            bool                    InputImpl_IsKeyboardAltPressed   (InputImpl const*);
            bool                    InputImpl_IsKeyboardAltDown      (InputImpl const*);
            bool                    InputImpl_IsKeyboardCtrlPressed  (InputImpl const*);
            bool                    InputImpl_IsKeyboardCtrlDown     (InputImpl const*);
            bool                    InputImpl_IsKeyboardShiftPressed (InputImpl const*);
            bool                    InputImpl_IsKeyboardShiftDown    (InputImpl const*);
        ]]
    end

    do -- Global Symbol Table
        InputImpl = {}

        if onDef_InputImpl then onDef_InputImpl(InputImpl, mt) end
        InputImpl = setmetatable(InputImpl, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('InputImpl')
        local mt = {
            __index = {
                keyboard               = libphx.InputImpl_Keyboard,
                mouse                  = libphx.InputImpl_Mouse,
                touchpad               = libphx.InputImpl_Touchpad,
                gamepad                = libphx.InputImpl_Gamepad,
                dragAndDrop            = libphx.InputImpl_DragAndDrop,
                activeDevice           = libphx.InputImpl_ActiveDevice,
                activeDeviceType       = libphx.InputImpl_ActiveDeviceType,
                activeDeviceId         = libphx.InputImpl_ActiveDeviceId,
                setCursorVisible       = libphx.InputImpl_SetCursorVisible,
                setCursorVisibleAuto   = libphx.InputImpl_SetCursorVisibleAuto,
                setCursorPosition      = libphx.InputImpl_SetCursorPosition,
                isPressed              = libphx.InputImpl_IsPressed,
                isDown                 = libphx.InputImpl_IsDown,
                isReleased             = libphx.InputImpl_IsReleased,
                getValue               = libphx.InputImpl_GetValue,
                isKeyboardAltPressed   = libphx.InputImpl_IsKeyboardAltPressed,
                isKeyboardAltDown      = libphx.InputImpl_IsKeyboardAltDown,
                isKeyboardCtrlPressed  = libphx.InputImpl_IsKeyboardCtrlPressed,
                isKeyboardCtrlDown     = libphx.InputImpl_IsKeyboardCtrlDown,
                isKeyboardShiftPressed = libphx.InputImpl_IsKeyboardShiftPressed,
                isKeyboardShiftDown    = libphx.InputImpl_IsKeyboardShiftDown,
            },
        }

        if onDef_InputImpl_t then onDef_InputImpl_t(t, mt) end
        InputImpl_t = ffi.metatype(t, mt)
    end

    return InputImpl
end

return Loader
