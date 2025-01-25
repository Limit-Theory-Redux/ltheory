-- EventData -------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct EventData {} EventData;
    ]]

    return 1, 'EventData'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local EventData

    do -- C Definitions
        ffi.cdef [[
            void           EventData_Free       (EventData*);
            double         EventData_DeltaTime  (EventData const*);
            FrameStage     EventData_FrameStage (EventData const*);
            uint32         EventData_TunnelId   (EventData const*);
            Payload const* EventData_Payload    (EventData const*);
        ]]
    end

    do -- Global Symbol Table
        EventData = {}

        if onDef_EventData then onDef_EventData(EventData, mt) end
        EventData = setmetatable(EventData, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('EventData')
        local mt = {
            __index = {
                deltaTime  = libphx.EventData_DeltaTime,
                frameStage = libphx.EventData_FrameStage,
                tunnelId   = libphx.EventData_TunnelId,
                payload    = libphx.EventData_Payload,
            },
        }

        if onDef_EventData_t then onDef_EventData_t(t, mt) end
        EventData_t = ffi.metatype(t, mt)
    end

    return EventData
end

return Loader
