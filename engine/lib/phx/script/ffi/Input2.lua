-- Input2 ----------------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('ffi.libphx').lib
local Input2

do -- C Definitions
    ffi.cdef [[
        typedef struct Input2 {} Input2;

        CursorState const*      Input2_Cursor      (Input2 const*);
        KeyboardState const*    Input2_Keyboard    (Input2 const*);
        MouseState const*       Input2_Mouse       (Input2 const*);
        TouchpadState const*    Input2_Touchpad    (Input2 const*);
        GamepadState const*     Input2_Gamepad     (Input2 const*);
        DragAndDropState const* Input2_DragAndDrop (Input2 const*);
    ]]
end

do -- Global Symbol Table
    Input2 = {
        Cursor      = libphx.Input2_Cursor,
        Keyboard    = libphx.Input2_Keyboard,
        Mouse       = libphx.Input2_Mouse,
        Touchpad    = libphx.Input2_Touchpad,
        Gamepad     = libphx.Input2_Gamepad,
        DragAndDrop = libphx.Input2_DragAndDrop,
    }

    if onDef_Input2 then onDef_Input2(Input2, mt) end
    Input2 = setmetatable(Input2, mt)
end

do -- Metatype for class instances
    local t  = ffi.typeof('Input2')
    local mt = {
        __index = {
            cursor      = libphx.Input2_Cursor,
            keyboard    = libphx.Input2_Keyboard,
            mouse       = libphx.Input2_Mouse,
            touchpad    = libphx.Input2_Touchpad,
            gamepad     = libphx.Input2_Gamepad,
            dragAndDrop = libphx.Input2_DragAndDrop,
        },
    }

    if onDef_Input2_t then onDef_Input2_t(t, mt) end
    Input2_t = ffi.metatype(t, mt)
end

return Input2
