-- AUTO GENERATED. DO NOT MODIFY!
-- NavAxis ---------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint32 NavAxis;
    ]]

    return 2, 'NavAxis'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local NavAxis

    do -- C Definitions
        ffi.cdef [[
            NavAxis NavAxis_Pitch;
            NavAxis NavAxis_Yaw;
            NavAxis NavAxis_Roll;
            NavAxis NavAxis_ThrustForward;
            NavAxis NavAxis_ThrustRight;
            NavAxis NavAxis_ThrustUp;

            cstr    NavAxis_ToString(NavAxis);
        ]]
    end

    do -- Global Symbol Table
        NavAxis = {
            Pitch         = libphx.NavAxis_Pitch,
            Yaw           = libphx.NavAxis_Yaw,
            Roll          = libphx.NavAxis_Roll,
            ThrustForward = libphx.NavAxis_ThrustForward,
            ThrustRight   = libphx.NavAxis_ThrustRight,
            ThrustUp      = libphx.NavAxis_ThrustUp,

            ToString      = libphx.NavAxis_ToString,
        }

        if onDef_NavAxis then onDef_NavAxis(NavAxis, mt) end
        NavAxis = setmetatable(NavAxis, mt)
    end

    return NavAxis
end

return Loader
