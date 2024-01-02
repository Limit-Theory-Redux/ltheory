-- Vec4u -----------------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('libphx').lib
local Vec4u

do -- Global Symbol Table
    Vec4u = {
    }

    local mt = {
        __call = function(t, ...) return Vec4u_t(...) end,
    }

    if onDef_Vec4u then onDef_Vec4u(Vec4u, mt) end
    Vec4u = setmetatable(Vec4u, mt)
end

do -- Metatype for class instances
    local t  = ffi.typeof('Vec4u')
    local mt = {
        __index = {
            clone = function(x) return Vec4u_t(x) end,
        },
    }

    if onDef_Vec4u_t then onDef_Vec4u_t(t, mt) end
    Vec4u_t = ffi.metatype(t, mt)
end

return Vec4u
