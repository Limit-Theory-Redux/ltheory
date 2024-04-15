-- Engine ----------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct Engine {} Engine;
    ]]

    return 1, 'Engine'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Engine

    do -- C Definitions
        ffi.cdef [[
            void    Engine_Free       (Engine*);
            Window* Engine_Window     (Engine*);
            Input*  Engine_Input      (Engine*);
            HmGui*  Engine_HmGui      (Engine*);
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
            Abort      = libphx.Engine_Abort,
            ---@return integer
            GetBits    = libphx.Engine_GetBits,
            ---@return string
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
                ---@return Window
                window  = libphx.Engine_Window,
                ---@return Input
                input   = libphx.Engine_Input,
                ---@return HmGui
                hmGui   = libphx.Engine_HmGui,
                ---@return number
                getTime = libphx.Engine_GetTime,
                exit    = libphx.Engine_Exit,
            },
        }

        if onDef_Engine_t then onDef_Engine_t(t, mt) end
        Engine_t = ffi.metatype(t, mt)
    end

    return Engine
end

return Loader
