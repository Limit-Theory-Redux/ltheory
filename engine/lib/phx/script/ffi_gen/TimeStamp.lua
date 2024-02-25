-- TimeStamp -------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct TimeStamp {} TimeStamp;
    ]]

    return 1, 'TimeStamp'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local TimeStamp

    do -- C Definitions
        ffi.cdef [[
            void       TimeStamp_Free          (TimeStamp*);
            TimeStamp* TimeStamp_Now           ();
            TimeStamp* TimeStamp_GetFuture     (double seconds);
            double     TimeStamp_GetDifference (TimeStamp const*, TimeStamp const* end);
            double     TimeStamp_GetElapsed    (TimeStamp const*);
            double     TimeStamp_GetElapsedMs  (TimeStamp const*);
            TimeStamp* TimeStamp_GetRelative   (TimeStamp const*, double seconds);
            double     TimeStamp_ToDouble      (TimeStamp const*);
            uint64     TimeStamp_ToSeconds     (TimeStamp const*);
        ]]
    end

    do -- Global Symbol Table
        TimeStamp = {
            Now           = function(...)
                local instance = libphx.TimeStamp_Now(...)
                ffi.gc(instance, libphx.TimeStamp_Free)
                return instance
            end,
            GetFuture     = function(...)
                local instance = libphx.TimeStamp_GetFuture(...)
                ffi.gc(instance, libphx.TimeStamp_Free)
                return instance
            end,
        }

        if onDef_TimeStamp then onDef_TimeStamp(TimeStamp, mt) end
        TimeStamp = setmetatable(TimeStamp, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('TimeStamp')
        local mt = {
            __index = {
                getDifference = libphx.TimeStamp_GetDifference,
                getElapsed    = libphx.TimeStamp_GetElapsed,
                getElapsedMs  = libphx.TimeStamp_GetElapsedMs,
                getRelative   = function(...)
                    local instance = libphx.TimeStamp_GetRelative(...)
                    ffi.gc(instance, libphx.TimeStamp_Free)
                    return instance
                end,
                toDouble      = libphx.TimeStamp_ToDouble,
                toSeconds     = libphx.TimeStamp_ToSeconds,
            },
        }

        if onDef_TimeStamp_t then onDef_TimeStamp_t(t, mt) end
        TimeStamp_t = ffi.metatype(t, mt)
    end

    return TimeStamp
end

return Loader
