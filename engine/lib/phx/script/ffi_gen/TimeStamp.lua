-- AUTO GENERATED. DO NOT MODIFY!
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
            double     TimeStamp_GetDifference (TimeStamp const*, TimeStamp const* endTime);
            double     TimeStamp_GetElapsed    (TimeStamp const*);
            double     TimeStamp_GetElapsedMs  (TimeStamp const*);
            TimeStamp* TimeStamp_GetRelative   (TimeStamp const*, double seconds);
            double     TimeStamp_ToDouble      (TimeStamp const*);
            uint64     TimeStamp_ToSeconds     (TimeStamp const*);
        ]]
    end

    do -- Global Symbol Table
        TimeStamp = {
            Now           = function()
                local _instance = libphx.TimeStamp_Now()
                return Core.ManagedObject(_instance, libphx.TimeStamp_Free)
            end,
            GetFuture     = function(seconds)
                local _instance = libphx.TimeStamp_GetFuture(seconds)
                return Core.ManagedObject(_instance, libphx.TimeStamp_Free)
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
                getRelative   = function(self, seconds)
                    local _instance = libphx.TimeStamp_GetRelative(self, seconds)
                    return Core.ManagedObject(_instance, libphx.TimeStamp_Free)
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
