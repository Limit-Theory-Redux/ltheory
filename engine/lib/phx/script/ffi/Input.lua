-- Input -----------------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('ffi.libphx').lib
local Input

do -- C Definitions
    ffi.cdef [[
        CursorState const*      Input_Cursor               (Input const*);
        KeyboardState const*    Input_Keyboard             (Input const*);
        MouseState const*       Input_Mouse                (Input const*);
        TouchpadState const*    Input_Touchpad             (Input const*);
        GamepadState const*     Input_Gamepad              (Input const*);
        DragAndDropState const* Input_DragAndDrop          (Input const*);
        InputDevice const*      Input_ActiveDevice         (Input const*);
        InputDeviceType const*  Input_ActiveDeviceType     (Input const*);
        InputDeviceId const*    Input_ActiveDeviceId       (Input const*);
        void                    Input_SetCursorVisible     (Input*, bool visible);
        void                    Input_SetCursorVisibleAuto (Input*);
        void                    Input_SetCursorPosition    (Input*, float x, float y);
        bool                    Input_GetPressed           (Input const*, Button button);
        bool                    Input_GetDown              (Input const*, Button button);
        bool                    Input_GetReleased          (Input const*, Button button);
        float                   Input_GetValue             (Input const*, Button button);
    ]]
end

do -- Global Symbol Table
    Input = {
        Cursor               = libphx.Input_Cursor,
        Keyboard             = libphx.Input_Keyboard,
        Mouse                = libphx.Input_Mouse,
        Touchpad             = libphx.Input_Touchpad,
        Gamepad              = libphx.Input_Gamepad,
        DragAndDrop          = libphx.Input_DragAndDrop,
        ActiveDevice         = libphx.Input_ActiveDevice,
        ActiveDeviceType     = libphx.Input_ActiveDeviceType,
        ActiveDeviceId       = libphx.Input_ActiveDeviceId,
        SetCursorVisible     = libphx.Input_SetCursorVisible,
        SetCursorVisibleAuto = libphx.Input_SetCursorVisibleAuto,
        SetCursorPosition    = libphx.Input_SetCursorPosition,
        GetPressed           = libphx.Input_GetPressed,
        GetDown              = libphx.Input_GetDown,
        GetReleased          = libphx.Input_GetReleased,
        GetValue             = libphx.Input_GetValue,
    }

    if onDef_Input then onDef_Input(Input, mt) end
    Input = setmetatable(Input, mt)
end

do -- Metatype for class instances
    local t  = ffi.typeof('Input')
    local mt = {
        __index = {
            cursor               = libphx.Input_Cursor,
            keyboard             = libphx.Input_Keyboard,
            mouse                = libphx.Input_Mouse,
            touchpad             = libphx.Input_Touchpad,
            gamepad              = libphx.Input_Gamepad,
            dragAndDrop          = libphx.Input_DragAndDrop,
            activeDevice         = libphx.Input_ActiveDevice,
            activeDeviceType     = libphx.Input_ActiveDeviceType,
            activeDeviceId       = libphx.Input_ActiveDeviceId,
            setCursorVisible     = libphx.Input_SetCursorVisible,
            setCursorVisibleAuto = libphx.Input_SetCursorVisibleAuto,
            setCursorPosition    = libphx.Input_SetCursorPosition,
            getPressed           = libphx.Input_GetPressed,
            getDown              = libphx.Input_GetDown,
            getReleased          = libphx.Input_GetReleased,
            getValue             = libphx.Input_GetValue,
        },
    }

    if onDef_Input_t then onDef_Input_t(t, mt) end
    Input_t = ffi.metatype(t, mt)
end

return Input
