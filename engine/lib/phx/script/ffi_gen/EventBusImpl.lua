-- EventBusImpl ----------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct EventBusImpl {} EventBusImpl;
    ]]

    return 1, 'EventBusImpl'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local EventBusImpl

    do -- C Definitions
        ffi.cdef [[
            void             EventBusImpl_Free               (EventBusImpl*);
            double           EventBusImpl_GetTimeScale       (EventBusImpl const*);
            void             EventBusImpl_SetTimeScale       (EventBusImpl*, double scaleFactor);
            void             EventBusImpl_Register           (EventBusImpl*, cstr eventName, int priority, FrameStage frameStage, bool withFrameStageMessage);
            void             EventBusImpl_Unregister         (EventBusImpl*, cstr eventName);
            uint32           EventBusImpl_Subscribe          (EventBusImpl*, cstr eventName, uint64 const* entityId);
            void             EventBusImpl_Unsubscribe        (EventBusImpl*, uint32 tunnelId);
            void             EventBusImpl_Send               (EventBusImpl*, cstr eventName, uint64 entityId);
            EventData const* EventBusImpl_GetNextEvent       (EventBusImpl*);
            void             EventBusImpl_PrintFrameStageMap (EventBusImpl const*);
        ]]
    end

    do -- Global Symbol Table
        EventBusImpl = {}

        if onDef_EventBusImpl then onDef_EventBusImpl(EventBusImpl, mt) end
        EventBusImpl = setmetatable(EventBusImpl, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('EventBusImpl')
        local mt = {
            __index = {
                getTimeScale       = libphx.EventBusImpl_GetTimeScale,
                setTimeScale       = libphx.EventBusImpl_SetTimeScale,
                register           = libphx.EventBusImpl_Register,
                unregister         = libphx.EventBusImpl_Unregister,
                subscribe          = libphx.EventBusImpl_Subscribe,
                unsubscribe        = libphx.EventBusImpl_Unsubscribe,
                send               = libphx.EventBusImpl_Send,
                getNextEvent       = libphx.EventBusImpl_GetNextEvent,
                printFrameStageMap = libphx.EventBusImpl_PrintFrameStageMap,
            },
        }

        if onDef_EventBusImpl_t then onDef_EventBusImpl_t(t, mt) end
        EventBusImpl_t = ffi.metatype(t, mt)
    end

    return EventBusImpl
end

return Loader
