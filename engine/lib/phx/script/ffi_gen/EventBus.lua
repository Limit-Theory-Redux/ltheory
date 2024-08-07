-- EventBus --------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct EventBus {} EventBus;
    ]]

    return 1, 'EventBus'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local EventBus

    do -- C Definitions
        ffi.cdef [[
            void             EventBus_Free                (EventBus*);
            double           EventBus_GetTimeScale        (EventBus const*);
            void             EventBus_SetTimeScale        (EventBus*, double scaleFactor);
            void             EventBus_Register            (EventBus*, uint16 eventId, cstr eventName, FrameStage frameStage);
            void             EventBus_Unregister          (EventBus*, uint16 eventId);
            uint32           EventBus_Subscribe           (EventBus*, uint16 eventId, uint64 const* entityId);
            void             EventBus_Unsubscribe         (EventBus*, uint32 tunnelId);
            void             EventBus_Send                (EventBus*, uint16 eventId, uint64 const* entityId, EventPayload const* payload);
            void             EventBus_StartEventIteration (EventBus*);
            EventData const* EventBus_NextEvent           (EventBus*);
            void             EventBus_PrintFrameStageMap  (EventBus const*);
        ]]
    end

    do -- Global Symbol Table
        EventBus = {}

        if onDef_EventBus then onDef_EventBus(EventBus, mt) end
        EventBus = setmetatable(EventBus, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('EventBus')
        local mt = {
            __index = {
                getTimeScale        = libphx.EventBus_GetTimeScale,
                setTimeScale        = libphx.EventBus_SetTimeScale,
                register            = libphx.EventBus_Register,
                unregister          = libphx.EventBus_Unregister,
                subscribe           = libphx.EventBus_Subscribe,
                unsubscribe         = libphx.EventBus_Unsubscribe,
                send                = libphx.EventBus_Send,
                startEventIteration = libphx.EventBus_StartEventIteration,
                nextEvent           = function(...)
                    local instance = libphx.EventBus_NextEvent(...)
                    return Core.ManagedObject(instance, libphx.EventData_Free)
                end,
                printFrameStageMap  = libphx.EventBus_PrintFrameStageMap,
            },
        }

        if onDef_EventBus_t then onDef_EventBus_t(t, mt) end
        EventBus_t = ffi.metatype(t, mt)
    end

    return EventBus
end

return Loader
