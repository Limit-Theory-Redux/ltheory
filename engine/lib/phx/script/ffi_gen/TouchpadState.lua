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
        TouchpadState = {}

        if onDef_TouchpadState then onDef_TouchpadState(TouchpadState, mt) end
        TouchpadState = setmetatable(TouchpadState, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('TouchpadState')
        local mt = {
            __index = {
                ---@param axis TouchpadAxis
                ---@return number
                value        = libphx.TouchpadState_Value,
                ---@return Vec2
                position     = libphx.TouchpadState_Position,
                ---@return number
                magnifyDelta = libphx.TouchpadState_MagnifyDelta,
                ---@return number
                rotateDelta  = libphx.TouchpadState_RotateDelta,
            },
        }

        if onDef_TouchpadState_t then onDef_TouchpadState_t(t, mt) end
        TouchpadState_t = ffi.metatype(t, mt)
    end

    return TouchpadState
end

return Loader
