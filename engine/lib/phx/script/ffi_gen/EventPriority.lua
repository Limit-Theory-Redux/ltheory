-- EventPriority ---------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint16 EventPriority;
    ]]

    return 2, 'EventPriority'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local EventPriority

    do -- C Definitions
        ffi.cdef [[
            EventPriority EventPriority_Low;
            EventPriority EventPriority_Medium;
            EventPriority EventPriority_High;
            EventPriority EventPriority_Max;

            cstr          EventPriority_ToString(EventPriority);
        ]]
    end

    do -- Global Symbol Table
        EventPriority = {
            Low      = libphx.EventPriority_Low,
            Medium   = libphx.EventPriority_Medium,
            High     = libphx.EventPriority_High,
            Max      = libphx.EventPriority_Max,

            ToString = libphx.EventPriority_ToString,
        }

        if onDef_EventPriority then onDef_EventPriority(EventPriority, mt) end
        EventPriority = setmetatable(EventPriority, mt)
    end

    return EventPriority
end

return Loader
