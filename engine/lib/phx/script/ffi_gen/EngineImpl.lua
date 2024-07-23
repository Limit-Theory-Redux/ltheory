-- EngineImpl ------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct EngineImpl {} EngineImpl;
    ]]

    return 1, 'EngineImpl'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local EngineImpl

    do -- C Definitions
        ffi.cdef [[
            void          EngineImpl_Free       (EngineImpl*);
            Window*       EngineImpl_Window     (EngineImpl*);
            Input*        EngineImpl_Input      (EngineImpl*);
            EventBusImpl* EngineImpl_EventBus   (EngineImpl*);
            HmGui*        EngineImpl_HmGui      (EngineImpl*);
            void          EngineImpl_Abort      ();
            int           EngineImpl_GetBits    ();
            double        EngineImpl_GetTime    (EngineImpl const*);
            cstr          EngineImpl_GetVersion ();
            void          EngineImpl_Exit       (EngineImpl*);
            void          EngineImpl_Terminate  ();
            void          EngineImpl_Update     ();
        ]]
    end

    do -- Global Symbol Table
        EngineImpl = {
            Abort      = libphx.EngineImpl_Abort,
            GetBits    = libphx.EngineImpl_GetBits,
            GetVersion = libphx.EngineImpl_GetVersion,
            Terminate  = libphx.EngineImpl_Terminate,
            Update     = libphx.EngineImpl_Update,
        }

        if onDef_EngineImpl then onDef_EngineImpl(EngineImpl, mt) end
        EngineImpl = setmetatable(EngineImpl, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('EngineImpl')
        local mt = {
            __index = {
                window   = libphx.EngineImpl_Window,
                input    = libphx.EngineImpl_Input,
                eventBus = libphx.EngineImpl_EventBus,
                hmGui    = libphx.EngineImpl_HmGui,
                getTime  = libphx.EngineImpl_GetTime,
                exit     = libphx.EngineImpl_Exit,
            },
        }

        if onDef_EngineImpl_t then onDef_EngineImpl_t(t, mt) end
        EngineImpl_t = ffi.metatype(t, mt)
    end

    return EngineImpl
end

return Loader
