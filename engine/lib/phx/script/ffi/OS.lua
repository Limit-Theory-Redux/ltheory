-- OS --------------------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('ffi.libphx').lib
local OS

do -- C Definitions
    ffi.cdef [[
        uint32 OS_GetCpuCount ();
    ]]
end

do -- Global Symbol Table
    OS = {
        GetCpuCount = libphx.OS_GetCpuCount,
    }

    if onDef_OS then onDef_OS(OS, mt) end
    OS = setmetatable(OS, mt)
end

return OS
