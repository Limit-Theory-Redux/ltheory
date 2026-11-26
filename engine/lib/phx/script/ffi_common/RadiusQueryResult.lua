-- RadiusQueryResult -----------------------------------------------------------
local ffi = require('ffi')
local libphx = require('libphx').lib
local RadiusQueryResult

do -- Global Symbol Table
    RadiusQueryResult = {
    }

    local mt = {
        __call = function(t, ...) return RadiusQueryResult_t(...) end,
    }

    if onDef_RadiusQueryResult then onDef_RadiusQueryResult(RadiusQueryResult, mt) end
    RadiusQueryResult = setmetatable(RadiusQueryResult, mt)
end

do -- Metatype for class instances
    local t  = ffi.typeof('RadiusQueryResult')
    local mt = {
        __index = {
            clone = function(x) return RadiusQueryResult_t(x) end,
        },
    }

    if onDef_RadiusQueryResult_t then onDef_RadiusQueryResult_t(t, mt) end
    RadiusQueryResult_t = ffi.metatype(t, mt)
end

return RadiusQueryResult
