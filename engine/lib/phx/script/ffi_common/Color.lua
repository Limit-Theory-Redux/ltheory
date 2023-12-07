-- Color -----------------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('libphx').lib
local Color

do -- Global Symbol Table
    Color = {
    }

    local mt = {
        __call = function(t, ...) return Color_t(...) end,
    }

    if onDef_Color then onDef_Color(Color, mt) end
    Color = setmetatable(Color, mt)
end

do -- Metatype for class instances
    local t  = ffi.typeof('Color')
    local mt = {
        __index = {
            clone = function(x) return Color_t(x) end,
        },
    }

    if onDef_Color_t then onDef_Color_t(t, mt) end
    Color_t = ffi.metatype(t, mt)
end

return Color
