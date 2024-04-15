-- Timer -----------------------------------------------------------------------

---@class Timer
---@field Create fun(): Timer
---@field GetAndReset fun(self): number
---@field GetElapsed fun(self): number
---@field Reset fun(self)

local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct Timer {} Timer;
    ]]

    return 1, 'Timer'
end

function Loader.defineType()
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
            ---@return Timer
            Create      = function(...)
                local instance = libphx.Timer_Create(...)
                return Core.ManagedObject(instance, libphx.Timer_Free)
            end,
        }

        if onDef_Timer then onDef_Timer(Timer, mt) end
        Timer = setmetatable(Timer, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Timer')
        local mt = {
            __index = {
                ---@return number
                getAndReset = libphx.Timer_GetAndReset,
                ---@return number
                getElapsed  = libphx.Timer_GetElapsed,
                reset       = libphx.Timer_Reset,
            },
        }

        if onDef_Timer_t then onDef_Timer_t(t, mt) end
        Timer_t = ffi.metatype(t, mt)
    end

    return Timer
end

return Loader
