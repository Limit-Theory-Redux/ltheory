-- SystemEventState ------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('ffi.libphx').lib
local SystemEventState

do -- C Definitions
    ffi.cdef [[
        bool SystemEventState_IsExit (SystemEventState const*);
    ]]
end

do -- Global Symbol Table
    SystemEventState = {
        IsExit = libphx.SystemEventState_IsExit,
    }

    if onDef_SystemEventState then onDef_SystemEventState(SystemEventState, mt) end
    SystemEventState = setmetatable(SystemEventState, mt)
end

do -- Metatype for class instances
    local t  = ffi.typeof('SystemEventState')
    local mt = {
        __index = {
            isExit = libphx.SystemEventState_IsExit,
        },
    }

    if onDef_SystemEventState_t then onDef_SystemEventState_t(t, mt) end
    SystemEventState_t = ffi.metatype(t, mt)
end

return SystemEventState
