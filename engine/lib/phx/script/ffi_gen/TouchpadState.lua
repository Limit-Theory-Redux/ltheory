-- TouchpadState ---------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct TouchpadState {} TouchpadState;
    ]]

    return 1, 'TouchpadState'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local TouchpadState

    do -- C Definitions
        ffi.cdef [[
            void  TouchpadState_Free         (TouchpadState*);
            float TouchpadState_Value        (TouchpadState const*, TouchpadAxis axis);
            Vec2f TouchpadState_Position     (TouchpadState const*);
            float TouchpadState_MagnifyDelta (TouchpadState const*);
            float TouchpadState_RotateDelta  (TouchpadState const*);
        ]]
    end

    do -- Global Symbol Table
        TouchpadState = {
            Value        = libphx.TouchpadState_Value,
            Position     = libphx.TouchpadState_Position,
            MagnifyDelta = libphx.TouchpadState_MagnifyDelta,
            RotateDelta  = libphx.TouchpadState_RotateDelta,
        }

        if onDef_TouchpadState then onDef_TouchpadState(TouchpadState, mt) end
        TouchpadState = setmetatable(TouchpadState, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('TouchpadState')
        local mt = {
            __index = {
                value        = libphx.TouchpadState_Value,
                position     = libphx.TouchpadState_Position,
                magnifyDelta = libphx.TouchpadState_MagnifyDelta,
                rotateDelta  = libphx.TouchpadState_RotateDelta,
            },
        }

        if onDef_TouchpadState_t then onDef_TouchpadState_t(t, mt) end
        TouchpadState_t = ffi.metatype(t, mt)
    end

    return TouchpadState
end

return Loader
