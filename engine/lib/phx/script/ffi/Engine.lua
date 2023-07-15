-- Engine ----------------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('ffi.libphx').lib
local Engine

do -- C Definitions
    ffi.cdef [[
        void   Engine_Free       ();
        void   Engine_Abort      ();
        int    Engine_GetBits    ();
        double Engine_GetTime    (Engine const*);
        cstr   Engine_GetVersion ();
        void   Engine_Terminate  ();
        void   Engine_Update     ();
    ]]
end

do -- Global Symbol Table
    Engine = {
        Free       = libphx.Engine_Free,
        Abort      = libphx.Engine_Abort,
        GetBits    = libphx.Engine_GetBits,
        GetTime    = libphx.Engine_GetTime,
        GetVersion = libphx.Engine_GetVersion,
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
            getTime = libphx.Engine_GetTime,
        },
    }

    if onDef_Engine_t then onDef_Engine_t(t, mt) end
    Engine_t = ffi.metatype(t, mt)
end

return Engine
