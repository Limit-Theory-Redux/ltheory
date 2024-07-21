-- EventType -------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 EventType;
    ]]

    return 2, 'EventType'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local EventType

    do -- C Definitions
        ffi.cdef [[
            EventType EventType_SomeType;

            cstr      EventType_ToString(EventType);
        ]]
    end

    do -- Global Symbol Table
        EventType = {
            SomeType = libphx.EventType_SomeType,

            ToString = libphx.EventType_ToString,
        }

        if onDef_EventType then onDef_EventType(EventType, mt) end
        EventType = setmetatable(EventType, mt)
    end

    return EventType
end

return Loader
