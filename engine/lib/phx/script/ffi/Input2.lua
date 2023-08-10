-- Input2 ----------------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('ffi.libphx').lib
local Input2

do -- C Definitions
    ffi.cdef [[
        typedef struct Input2 {} Input2;

        CursorState const*      Input2_Cursor               (Input2 const*);
        KeyboardState const*    Input2_Keyboard             (Input2 const*);
        MouseState const*       Input2_Mouse                (Input2 const*);
        TouchpadState const*    Input2_Touchpad             (Input2 const*);
        GamepadState const*     Input2_Gamepad              (Input2 const*);
        DragAndDropState const* Input2_DragAndDrop          (Input2 const*);
        InputDevice const*      Input2_ActiveDevice         (Input2 const*);
        InputDeviceType const*  Input2_ActiveDeviceType     (Input2 const*);
        InputDeviceId const*    Input2_ActiveDeviceId       (Input2 const*);
        void                    Input2_SetCursorVisible     (Input2*, bool visible);
        void                    Input2_SetCursorVisibleAuto (Input2*);
        void                    Input2_SetCursorPosition    (Input2*, float x, float y);
        bool                    Input2_GetPressed           (Input2 const*, Button2 button);
        bool                    Input2_GetDown              (Input2 const*, Button2 button);
        bool                    Input2_GetReleased          (Input2 const*, Button2 button);
        float                   Input2_GetValue             (Input2 const*, Button2 button);
    ]]
end

do -- Global Symbol Table
    Input2 = {
        Cursor               = libphx.Input2_Cursor,
        Keyboard             = libphx.Input2_Keyboard,
        Mouse                = libphx.Input2_Mouse,
        Touchpad             = libphx.Input2_Touchpad,
        Gamepad              = libphx.Input2_Gamepad,
        DragAndDrop          = libphx.Input2_DragAndDrop,
        ActiveDevice         = libphx.Input2_ActiveDevice,
        ActiveDeviceType     = libphx.Input2_ActiveDeviceType,
        ActiveDeviceId       = libphx.Input2_ActiveDeviceId,
        SetCursorVisible     = libphx.Input2_SetCursorVisible,
        SetCursorVisibleAuto = libphx.Input2_SetCursorVisibleAuto,
        SetCursorPosition    = libphx.Input2_SetCursorPosition,
        GetPressed           = libphx.Input2_GetPressed,
        GetDown              = libphx.Input2_GetDown,
        GetReleased          = libphx.Input2_GetReleased,
        GetValue             = libphx.Input2_GetValue,
    }

    if onDef_Input2 then onDef_Input2(Input2, mt) end
    Input2 = setmetatable(Input2, mt)
end

do -- Metatype for class instances
    local t  = ffi.typeof('Input2')
    local mt = {
        __index = {
            cursor               = libphx.Input2_Cursor,
            keyboard             = libphx.Input2_Keyboard,
            mouse                = libphx.Input2_Mouse,
            touchpad             = libphx.Input2_Touchpad,
            gamepad              = libphx.Input2_Gamepad,
            dragAndDrop          = libphx.Input2_DragAndDrop,
            activeDevice         = libphx.Input2_ActiveDevice,
            activeDeviceType     = libphx.Input2_ActiveDeviceType,
            activeDeviceId       = libphx.Input2_ActiveDeviceId,
            setCursorVisible     = libphx.Input2_SetCursorVisible,
            setCursorVisibleAuto = libphx.Input2_SetCursorVisibleAuto,
            setCursorPosition    = libphx.Input2_SetCursorPosition,
            getPressed           = libphx.Input2_GetPressed,
            getDown              = libphx.Input2_GetDown,
            getReleased          = libphx.Input2_GetReleased,
            getValue             = libphx.Input2_GetValue,
        },
    }

    if onDef_Input2_t then onDef_Input2_t(t, mt) end
    Input2_t = ffi.metatype(t, mt)
end

return Input2
