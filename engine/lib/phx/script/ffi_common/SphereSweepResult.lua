-- SphereSweepResult -----------------------------------------------------------
local ffi = require('ffi')
local libphx = require('libphx').lib
local SphereSweepResult

do -- Global Symbol Table
    SphereSweepResult = {
    }

    local mt = {
        __call = function(t, ...) return SphereSweepResult_t(...) end,
    }

    if onDef_SphereSweepResult then onDef_SphereSweepResult(SphereSweepResult, mt) end
    SphereSweepResult = setmetatable(SphereSweepResult, mt)
end

do -- Metatype for class instances
    local t  = ffi.typeof('SphereSweepResult')
    local mt = {
        __index = {
            clone = function(x) return SphereSweepResult_t(x) end,
        },
    }

    if onDef_SphereSweepResult_t then onDef_SphereSweepResult_t(t, mt) end
    SphereSweepResult_t = ffi.metatype(t, mt)
end

return SphereSweepResult
