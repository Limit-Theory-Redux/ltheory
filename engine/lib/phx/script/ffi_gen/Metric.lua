-- AUTO GENERATED. DO NOT MODIFY!
-- Metric ----------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    return 0, 'Metric'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Metric

    do -- C Definitions
        ffi.cdef [[
            int  Metric_Get     (Metric this);
            cstr Metric_GetName (Metric this);
        ]]
    end

    do -- Global Symbol Table
        Metric = {
            Get     = libphx.Metric_Get,
            GetName = libphx.Metric_GetName,
        }

        if onDef_Metric then onDef_Metric(Metric, mt) end
        Metric = setmetatable(Metric, mt)
    end

    return Metric
end

return Loader
