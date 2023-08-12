-- InputDeviceId ---------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('ffi.libphx').lib
local InputDeviceId

do -- C Definitions
    ffi.cdef [[
        cstr InputDeviceId_ToString (InputDeviceId const*);
    ]]
end

do -- Global Symbol Table
    InputDeviceId = {
        ToString = libphx.InputDeviceId_ToString,
    }

    if onDef_InputDeviceId then onDef_InputDeviceId(InputDeviceId, mt) end
    InputDeviceId = setmetatable(InputDeviceId, mt)
end

do -- Metatype for class instances
    local t  = ffi.typeof('InputDeviceId')
    local mt = {
        __tostring = function(self) return ffi.string(libphx.InputDeviceId_ToString(self)) end,
        __index = {
            toString = libphx.InputDeviceId_ToString,
        },
    }

    if onDef_InputDeviceId_t then onDef_InputDeviceId_t(t, mt) end
    InputDeviceId_t = ffi.metatype(t, mt)
end

return InputDeviceId
