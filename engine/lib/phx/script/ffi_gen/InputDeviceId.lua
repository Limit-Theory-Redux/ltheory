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
            void InputDeviceId_Free     (InputDeviceId*);
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
end

return Loader
