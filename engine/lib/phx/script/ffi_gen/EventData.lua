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
            void       EventData_Free          (EventData*);
            double     EventData_GetDeltaTime  (EventData const*);
            FrameStage EventData_GetFrameStage (EventData const*);
            uint32     EventData_GetTunnelId   (EventData const*);
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
                getDeltaTime  = libphx.EventData_GetDeltaTime,
                getFrameStage = libphx.EventData_GetFrameStage,
                getTunnelId   = libphx.EventData_GetTunnelId,
            },
        }

        if onDef_EventData_t then onDef_EventData_t(t, mt) end
        EventData_t = ffi.metatype(t, mt)
    end

    return EventData
end

return Loader
