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
            void             EventBus_Free               (EventBus*);
            void             EventBus_Register           (EventBus*, cstr eventName, uint16 const* priority, UpdatePass updatePass, bool withUpdatePassMessage);
            void             EventBus_Unregister         (EventBus*, cstr eventName);
            uint32           EventBus_Subscribe          (EventBus*, cstr eventName, uint32 const* entityId);
            void             EventBus_Unsubscribe        (EventBus*, uint32 tunnelId);
            void             EventBus_Send               (EventBus*, cstr eventName, uint32 entityId);
            EventData const* EventBus_GetNextEvent       (EventBus*);
            void             EventBus_PrintUpdatePassMap (EventBus const*);
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
                register           = libphx.EventBus_Register,
                unregister         = libphx.EventBus_Unregister,
                subscribe          = libphx.EventBus_Subscribe,
                unsubscribe        = libphx.EventBus_Unsubscribe,
                send               = libphx.EventBus_Send,
                getNextEvent       = libphx.EventBus_GetNextEvent,
                printUpdatePassMap = libphx.EventBus_PrintUpdatePassMap,
            },
        }

        if onDef_EventBus_t then onDef_EventBus_t(t, mt) end
        EventBus_t = ffi.metatype(t, mt)
    end

    return EventBus
end

return Loader
