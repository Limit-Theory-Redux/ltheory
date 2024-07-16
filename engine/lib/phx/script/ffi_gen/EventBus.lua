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
            void EventBus_Free               (EventBus*);
            void EventBus_Register           (EventBus*, cstr eventName, int16 const* priority, UpdatePass* updatePass);
            void EventBus_Unregister         (EventBus*, cstr eventName);
            int  EventBus_Subscribe          (EventBus*, cstr eventName, int const* entityId);
            void EventBus_Unsubscribe        (EventBus*, int tunnelId);
            void EventBus_Send               (EventBus const*, cstr eventName, int entityId);
            void EventBus_PrintUpdatePassMap (EventBus const*);
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
                printUpdatePassMap = libphx.EventBus_PrintUpdatePassMap,
            },
        }

        if onDef_EventBus_t then onDef_EventBus_t(t, mt) end
        EventBus_t = ffi.metatype(t, mt)
    end

    return EventBus
end

return Loader
