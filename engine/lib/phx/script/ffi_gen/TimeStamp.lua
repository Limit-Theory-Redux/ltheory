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
            ---@return TimeStamp*
            Now           = function(...)
                local instance = libphx.TimeStamp_Now(...)
                return Core.ManagedObject(instance, libphx.TimeStamp_Free)
            end,
            ---@param seconds double
            ---@return TimeStamp*
            GetFuture     = function(...)
                local instance = libphx.TimeStamp_GetFuture(...)
                return Core.ManagedObject(instance, libphx.TimeStamp_Free)
            end,
        }

        if onDef_TimeStamp then onDef_TimeStamp(TimeStamp, mt) end
        TimeStamp = setmetatable(TimeStamp, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('TimeStamp')
        local mt = {
            __index = {
                ---@param end TimeStamp const*
                ---@return double
                getDifference = libphx.TimeStamp_GetDifference,
                -- Number of seconds elapsed since this timestamp.
                ---@return double
                getElapsed    = libphx.TimeStamp_GetElapsed,
                ---@return double
                getElapsedMs  = libphx.TimeStamp_GetElapsedMs,
                ---@param seconds double
                ---@return TimeStamp*
                getRelative   = function(...)
                    local instance = libphx.TimeStamp_GetRelative(...)
                    return Core.ManagedObject(instance, libphx.TimeStamp_Free)
                end,
                ---@return double
                toDouble      = libphx.TimeStamp_ToDouble,
                ---@return uint64
                toSeconds     = libphx.TimeStamp_ToSeconds,
            },
        }

        if onDef_TimeStamp_t then onDef_TimeStamp_t(t, mt) end
        TimeStamp_t = ffi.metatype(t, mt)
    end

    return TimeStamp
end

return Loader
