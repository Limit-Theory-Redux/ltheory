-- TimeStamp -------------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('ffi.libphx').lib
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
        Free          = libphx.TimeStamp_Free,
        Now           = libphx.TimeStamp_Now,
        GetFuture     = libphx.TimeStamp_GetFuture,
        GetDifference = libphx.TimeStamp_GetDifference,
        GetElapsed    = libphx.TimeStamp_GetElapsed,
        GetElapsedMs  = libphx.TimeStamp_GetElapsedMs,
        GetRelative   = libphx.TimeStamp_GetRelative,
        ToDouble      = libphx.TimeStamp_ToDouble,
        ToSeconds     = libphx.TimeStamp_ToSeconds,
    }

    if onDef_TimeStamp then onDef_TimeStamp(TimeStamp, mt) end
    TimeStamp = setmetatable(TimeStamp, mt)
end

do -- Metatype for class instances
    local t  = ffi.typeof('TimeStamp')
    local mt = {
        __index = {
            managed       = function (self) return ffi.gc(self, libphx.TimeStamp_Free) end,
            free          = libphx.TimeStamp_Free,
            getDifference = libphx.TimeStamp_GetDifference,
            getElapsed    = libphx.TimeStamp_GetElapsed,
            getElapsedMs  = libphx.TimeStamp_GetElapsedMs,
            getRelative   = libphx.TimeStamp_GetRelative,
            toDouble      = libphx.TimeStamp_ToDouble,
            toSeconds     = libphx.TimeStamp_ToSeconds,
        },
    }

    if onDef_TimeStamp_t then onDef_TimeStamp_t(t, mt) end
    TimeStamp_t = ffi.metatype(t, mt)
end

return TimeStamp
