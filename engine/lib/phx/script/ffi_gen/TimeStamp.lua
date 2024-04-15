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
            ---@return Self
            Now           = function(...)
                local instance = libphx.TimeStamp_Now(...)
                return Core.ManagedObject(instance, libphx.TimeStamp_Free)
            end,
            ---@param seconds number
            ---@return Self
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
                ---@param end TimeStamp
                ---@return number
                getDifference = libphx.TimeStamp_GetDifference,
                -- Number of seconds elapsed since this timestamp.
                ---@return number
                getElapsed    = libphx.TimeStamp_GetElapsed,
                ---@return number
                getElapsedMs  = libphx.TimeStamp_GetElapsedMs,
                ---@param seconds number
                ---@return Self
                getRelative   = function(...)
                    local instance = libphx.TimeStamp_GetRelative(...)
                    return Core.ManagedObject(instance, libphx.TimeStamp_Free)
                end,
                ---@return number
                toDouble      = libphx.TimeStamp_ToDouble,
                ---@return integer
                toSeconds     = libphx.TimeStamp_ToSeconds,
            },
        }

        if onDef_TimeStamp_t then onDef_TimeStamp_t(t, mt) end
        TimeStamp_t = ffi.metatype(t, mt)
    end

    return TimeStamp
end

return Loader
