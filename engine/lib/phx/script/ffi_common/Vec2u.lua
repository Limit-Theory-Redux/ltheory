-- Vec2u -----------------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('libphx').lib
local Vec2u

do -- Global Symbol Table
    Vec2u = {
    }

    local mt = {
        __call = function(t, ...) return Vec2u_t(...) end,
    }

    if onDef_Vec2u then onDef_Vec2u(Vec2u, mt) end
    Vec2u = setmetatable(Vec2u, mt)
end

do -- Metatype for class instances
    local t  = ffi.typeof('Vec2u')
    local mt = {
        __index = {
            clone = function(x) return Vec2u_t(x) end,
        },
    }

    if onDef_Vec2u_t then onDef_Vec2u_t(t, mt) end
    Vec2u_t = ffi.metatype(t, mt)
end

return Vec2u
