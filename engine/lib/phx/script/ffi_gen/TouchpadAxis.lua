-- AUTO GENERATED. DO NOT MODIFY!
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
            cstr         TouchpadAxis_ToString(TouchpadAxis);
        ]]
    end

    do -- Global Symbol Table
        TouchpadAxis = {
            X            = 0,
            Y            = 1,
            MagnifyDelta = 2,
            RotateDelta  = 3,

            ToString     = libphx.TouchpadAxis_ToString,
        }

        if onDef_TouchpadAxis then onDef_TouchpadAxis(TouchpadAxis, mt) end
        TouchpadAxis = setmetatable(TouchpadAxis, mt)
    end

    return TouchpadAxis
end

return Loader
