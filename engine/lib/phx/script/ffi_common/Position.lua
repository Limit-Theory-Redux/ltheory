-- Position -----------------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('libphx').lib
local Position

do -- Global Symbol Table
    Position = {
    }

    local mt = {
        __call = function(t, ...) return Position_t(...) end,
    }

    if onDef_Position then onDef_Position(Position, mt) end
    Position = setmetatable(Position, mt)
end

do -- Metatype for class instances
    local t  = ffi.typeof('Position')
    local mt = {
        __index = {
            clone = function(x) return Position_t(x) end,
        },
    }

    if onDef_Position_t then onDef_Position_t(t, mt) end
    Position_t = ffi.metatype(t, mt)
end

return Position
