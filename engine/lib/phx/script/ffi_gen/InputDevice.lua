-- InputDevice -----------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct InputDevice {} InputDevice;
    ]]

    return 1, 'InputDevice'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local InputDevice

    do -- C Definitions
        ffi.cdef [[
            bool InputDevice_Equal    (InputDevice const*, InputDevice const* other);
            cstr InputDevice_ToString (InputDevice const*);
        ]]
    end

    do -- Global Symbol Table
        InputDevice = {
            Equal    = libphx.InputDevice_Equal,
            ToString = libphx.InputDevice_ToString,
        }

        local mt = {
            __call = function(t, ...) return InputDevice_t(...) end,
        }

        if onDef_InputDevice then onDef_InputDevice(InputDevice, mt) end
        InputDevice = setmetatable(InputDevice, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('InputDevice')
        local mt = {
            __tostring = function(self) return ffi.string(libphx.InputDevice_ToString(self)) end,
            __index = {
                clone    = function(x) return InputDevice_t(x) end,
                equal    = libphx.InputDevice_Equal,
                toString = libphx.InputDevice_ToString,
            },
        }

        if onDef_InputDevice_t then onDef_InputDevice_t(t, mt) end
        InputDevice_t = ffi.metatype(t, mt)
    end

    return InputDevice
end

return Loader
