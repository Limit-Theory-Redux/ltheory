-- Vec3i -----------------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('libphx').lib
local Vec3i

do -- Global Symbol Table
    Vec3i = {
    }

    local mt = {
        __call = function(t, ...) return Vec3i_t(...) end,
    }

    if onDef_Vec3i then onDef_Vec3i(Vec3i, mt) end
    Vec3i = setmetatable(Vec3i, mt)
end

do -- Metatype for class instances
    local t  = ffi.typeof('Vec3i')
    local mt = {
        __index = {
            clone = function(x) return Vec3i_t(x) end,
        },
    }

    if onDef_Vec3i_t then onDef_Vec3i_t(t, mt) end
    Vec3i_t = ffi.metatype(t, mt)
end

return Vec3i
