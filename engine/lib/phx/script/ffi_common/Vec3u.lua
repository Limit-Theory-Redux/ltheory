-- Vec3u -----------------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('libphx').lib
local Vec3u

do -- Global Symbol Table
    Vec3u = {
    }

    local mt = {
        __call = function(t, ...) return Vec3u_t(...) end,
    }

    if onDef_Vec3u then onDef_Vec3u(Vec3u, mt) end
    Vec3u = setmetatable(Vec3u, mt)
end

do -- Metatype for class instances
    local t  = ffi.typeof('Vec3u')
    local mt = {
        __index = {
            clone = function(x) return Vec3u_t(x) end,
        },
    }

    if onDef_Vec3u_t then onDef_Vec3u_t(t, mt) end
    Vec3u_t = ffi.metatype(t, mt)
end

return Vec3u
