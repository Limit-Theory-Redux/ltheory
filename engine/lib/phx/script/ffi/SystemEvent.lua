-- SystemEvent -----------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('ffi.libphx').lib
local SystemEvent

do -- C Definitions
    ffi.cdef [[
        SystemEvent SystemEvent_Exit;

        cstr        SystemEvent_ToString(SystemEvent);
    ]]
end

do -- Global Symbol Table
    SystemEvent = {
        Exit     = libphx.SystemEvent_Exit,

        ToString = libphx.SystemEvent_ToString,
    }

    if onDef_SystemEvent then onDef_SystemEvent(SystemEvent, mt) end
    SystemEvent = setmetatable(SystemEvent, mt)
end

return SystemEvent
