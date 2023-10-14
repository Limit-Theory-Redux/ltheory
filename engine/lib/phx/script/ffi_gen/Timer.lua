-- Timer -----------------------------------------------------------------------

function declareType()
    ffi.cdef [[
        typedef struct Timer {} Timer;
    ]]

    return 1, 'Timer'
end

function defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Timer

    do -- C Definitions
        ffi.cdef [[
            void   Timer_Free        (Timer*);
            Timer* Timer_Create      ();
            double Timer_GetAndReset (Timer*);
            double Timer_GetElapsed  (Timer const*);
            void   Timer_Reset       (Timer*);
        ]]
    end

    do -- Global Symbol Table
        Timer = {
            Free        = libphx.Timer_Free,
            Create      = libphx.Timer_Create,
            GetAndReset = libphx.Timer_GetAndReset,
            GetElapsed  = libphx.Timer_GetElapsed,
            Reset       = libphx.Timer_Reset,
        }

        if onDef_Timer then onDef_Timer(Timer, mt) end
        Timer = setmetatable(Timer, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Timer')
        local mt = {
            __index = {
                managed     = function(self) return ffi.gc(self, libphx.Timer_Free) end,
                free        = libphx.Timer_Free,
                getAndReset = libphx.Timer_GetAndReset,
                getElapsed  = libphx.Timer_GetElapsed,
                reset       = libphx.Timer_Reset,
            },
        }

        if onDef_Timer_t then onDef_Timer_t(t, mt) end
        Timer_t = ffi.metatype(t, mt)
    end

    return Timer
end

