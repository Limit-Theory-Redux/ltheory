-- AUTO GENERATED. DO NOT MODIFY!
-- SystemEvent -----------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 SystemEvent;
    ]]

    return 2, 'SystemEvent'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local SystemEvent

    do -- C Definitions
        ffi.cdef [[
            cstr        SystemEvent_ToString(SystemEvent);
        ]]
    end

    do -- Global Symbol Table
        SystemEvent = {
            Exit     = 0,

            ToString = libphx.SystemEvent_ToString,
        }

        if onDef_SystemEvent then onDef_SystemEvent(SystemEvent, mt) end
        SystemEvent = setmetatable(SystemEvent, mt)
    end

    return SystemEvent
end

return Loader
