-- EventType -------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint16 EventType;
    ]]

    return 2, 'EventType'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local EventType

    do -- C Definitions
        ffi.cdef [[
            EventType EventType_ResourceLoadingResult;
            EventType EventType_EngineEventTypesCount;

            cstr      EventType_ToString(EventType);
        ]]
    end

    do -- Global Symbol Table
        EventType = {
            ResourceLoadingResult = libphx.EventType_ResourceLoadingResult,
            EngineEventTypesCount = libphx.EventType_EngineEventTypesCount,

            ToString              = libphx.EventType_ToString,
        }

        if onDef_EventType then onDef_EventType(EventType, mt) end
        EventType = setmetatable(EventType, mt)
    end

    return EventType
end

return Loader
