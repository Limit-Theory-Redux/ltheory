-- Engine ----------------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('libphx').lib
local Engine

function declareType()
    ffi.cdef [[
        typedef struct Engine {} Engine;
    ]]

    return 1, 'Engine'
end

do -- C Definitions
    ffi.cdef [[
        Window* Engine_Window     (Engine*);
        Input*  Engine_Input      (Engine*);
        void    Engine_Free       ();
        void    Engine_Abort      ();
        int     Engine_GetBits    ();
        double  Engine_GetTime    (Engine const*);
        cstr    Engine_GetVersion ();
        void    Engine_Exit       (Engine*);
        void    Engine_Terminate  ();
        void    Engine_Update     ();
    ]]
end

do -- Global Symbol Table
    Engine = {
        Window     = libphx.Engine_Window,
        Input      = libphx.Engine_Input,
        Free       = libphx.Engine_Free,
        Abort      = libphx.Engine_Abort,
        GetBits    = libphx.Engine_GetBits,
        GetTime    = libphx.Engine_GetTime,
        GetVersion = libphx.Engine_GetVersion,
        Exit       = libphx.Engine_Exit,
        Terminate  = libphx.Engine_Terminate,
        Update     = libphx.Engine_Update,
    }

    if onDef_Engine then onDef_Engine(Engine, mt) end
    Engine = setmetatable(Engine, mt)
end

do -- Metatype for class instances
    local t  = ffi.typeof('Engine')
    local mt = {
        __index = {
            window  = libphx.Engine_Window,
            input   = libphx.Engine_Input,
            getTime = libphx.Engine_GetTime,
            exit    = libphx.Engine_Exit,
        },
    }

    if onDef_Engine_t then onDef_Engine_t(t, mt) end
    Engine_t = ffi.metatype(t, mt)
end

return Engine
