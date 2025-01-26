-- AUTO GENERATED. DO NOT MODIFY!
-- InstantTime -----------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct InstantTime {} InstantTime;
    ]]

    return 1, 'InstantTime'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local InstantTime

    do -- C Definitions
        ffi.cdef [[
            void         InstantTime_Free          (InstantTime*);
            InstantTime* InstantTime_Now           ();
            double       InstantTime_DurationSince (InstantTime const*, InstantTime const* earlier);
        ]]
    end

    do -- Global Symbol Table
        InstantTime = {
            Now           = function()
                local _instance = libphx.InstantTime_Now()
                return Core.ManagedObject(_instance, libphx.InstantTime_Free)
            end,
        }

        if onDef_InstantTime then onDef_InstantTime(InstantTime, mt) end
        InstantTime = setmetatable(InstantTime, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('InstantTime')
        local mt = {
            __index = {
                durationSince = libphx.InstantTime_DurationSince,
            },
        }

        if onDef_InstantTime_t then onDef_InstantTime_t(t, mt) end
        InstantTime_t = ffi.metatype(t, mt)
    end

    return InstantTime
end

return Loader
