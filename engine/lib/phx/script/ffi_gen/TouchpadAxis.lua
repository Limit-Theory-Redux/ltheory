-- TouchpadAxis ----------------------------------------------------------------


local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 TouchpadAxis;
    ]]

    return 2, 'TouchpadAxis'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local TouchpadAxis

    do -- C Definitions
        ffi.cdef [[
            TouchpadAxis TouchpadAxis_X;
            TouchpadAxis TouchpadAxis_Y;
            TouchpadAxis TouchpadAxis_MagnifyDelta;
            TouchpadAxis TouchpadAxis_RotateDelta;

            cstr         TouchpadAxis_ToString(TouchpadAxis);
        ]]
    end

    do -- Global Symbol Table
        TouchpadAxis = {
            X            = libphx.TouchpadAxis_X,
            Y            = libphx.TouchpadAxis_Y,
            MagnifyDelta = libphx.TouchpadAxis_MagnifyDelta,
            RotateDelta  = libphx.TouchpadAxis_RotateDelta,

            ToString     = libphx.TouchpadAxis_ToString,
        }

        if onDef_TouchpadAxis then onDef_TouchpadAxis(TouchpadAxis, mt) end
        TouchpadAxis = setmetatable(TouchpadAxis, mt)
    end

    return TouchpadAxis
end

return Loader
