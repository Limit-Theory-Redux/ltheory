-- AUTO GENERATED. DO NOT MODIFY!
-- Metric ----------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 Metric;
    ]]

    return 2, 'Metric'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Metric

    do -- C Definitions
        ffi.cdef [[
            cstr   Metric_ToString(Metric);

            int  Metric_Get     (Metric this);
            cstr Metric_GetName (Metric this);
        ]]
    end

    do -- Global Symbol Table
        Metric = {
            None       = 0,
            DrawCalls  = 1,
            Immediate  = 2,
            PolysDrawn = 3,
            TrisDrawn  = 4,
            VertsDrawn = 5,
            Flush      = 6,
            FBOSwap    = 7,
            SIZE       = 8,

            ToString   = libphx.Metric_ToString,

            Get     = libphx.Metric_Get,
            GetName = libphx.Metric_GetName,
        }

        if onDef_Metric then onDef_Metric(Metric, mt) end
        Metric = setmetatable(Metric, mt)
    end

    return Metric
end

return Loader
