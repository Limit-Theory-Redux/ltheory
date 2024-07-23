-- EventPriority ---------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 EventPriority;
    ]]

    return 2, 'EventPriority'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local EventPriority

    do -- C Definitions
        ffi.cdef [[
            EventPriority EventPriority_Lowest;
            EventPriority EventPriority_VeryLow;
            EventPriority EventPriority_Low;
            EventPriority EventPriority_BelowDefault;
            EventPriority EventPriority_Default;
            EventPriority EventPriority_AboveDefault;
            EventPriority EventPriority_High;
            EventPriority EventPriority_VeryHigh;
            EventPriority EventPriority_Highest;

            cstr          EventPriority_ToString(EventPriority);
        ]]
    end

    do -- Global Symbol Table
        EventPriority = {
            Lowest       = libphx.EventPriority_Lowest,
            VeryLow      = libphx.EventPriority_VeryLow,
            Low          = libphx.EventPriority_Low,
            BelowDefault = libphx.EventPriority_BelowDefault,
            Default      = libphx.EventPriority_Default,
            AboveDefault = libphx.EventPriority_AboveDefault,
            High         = libphx.EventPriority_High,
            VeryHigh     = libphx.EventPriority_VeryHigh,
            Highest      = libphx.EventPriority_Highest,

            ToString     = libphx.EventPriority_ToString,
        }

        if onDef_EventPriority then onDef_EventPriority(EventPriority, mt) end
        EventPriority = setmetatable(EventPriority, mt)
    end

    return EventPriority
end

return Loader
