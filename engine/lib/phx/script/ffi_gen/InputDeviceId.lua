-- InputDeviceId ---------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct InputDeviceId {} InputDeviceId;
    ]]

    return 1, 'InputDeviceId'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local InputDeviceId

    do -- C Definitions
        ffi.cdef [[
            void InputDeviceId_Free      (InputDeviceId*);
            cstr InputDeviceId_GetString (InputDeviceId const*);
        ]]
    end

    do -- Global Symbol Table
        InputDeviceId = {}

        if onDef_InputDeviceId then onDef_InputDeviceId(InputDeviceId, mt) end
        InputDeviceId = setmetatable(InputDeviceId, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('InputDeviceId')
        local mt = {
            __tostring = function(self) return ffi.string(libphx.InputDeviceId_GetString(self)) end,
            __index = {
                getString = libphx.InputDeviceId_GetString,
            },
        }

        if onDef_InputDeviceId_t then onDef_InputDeviceId_t(t, mt) end
        InputDeviceId_t = ffi.metatype(t, mt)
    end

    return InputDeviceId
end

return Loader
