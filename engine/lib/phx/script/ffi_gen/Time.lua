-- Time ------------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    return 0, 'Time'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Time

    do -- C Definitions
        ffi.cdef [[
            Time*  Time_GetLocal ();
            Time*  Time_GetUtc   ();
            uint32 Time_GetRaw   ();
        ]]
    end

    do -- Global Symbol Table
        Time = {
            GetLocal = function(...)
                local instance = libphx.Time_GetLocal(...)
                return Core.ManagedObject(instance, libphx.Time_Free)
            end,
            GetUtc   = function(...)
                local instance = libphx.Time_GetUtc(...)
                return Core.ManagedObject(instance, libphx.Time_Free)
            end,
            GetRaw   = libphx.Time_GetRaw,
        }

        local mt = {
            __call = function(t, ...) return Time_t(...) end,
        }

        if onDef_Time then onDef_Time(Time, mt) end
        Time = setmetatable(Time, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Time')
        local mt = {
            __index = {
                clone = function(x) return Time_t(x) end,
            },
        }

        if onDef_Time_t then onDef_Time_t(t, mt) end
        Time_t = ffi.metatype(t, mt)
    end

    return Time
end

return Loader
