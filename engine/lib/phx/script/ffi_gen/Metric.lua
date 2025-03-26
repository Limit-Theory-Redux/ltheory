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
            Metric Metric_None;
            Metric Metric_DrawCalls;
            Metric Metric_Immediate;
            Metric Metric_PolysDrawn;
            Metric Metric_TrisDrawn;
            Metric Metric_VertsDrawn;
            Metric Metric_Flush;
            Metric Metric_FBOSwap;
            Metric Metric_SIZE;

            cstr   Metric_ToString(Metric);

            int  Metric_Get     (Metric this);
            cstr Metric_GetName (Metric this);
        ]]
    end

    do -- Global Symbol Table
        Metric = {
            None       = libphx.Metric_None,
            DrawCalls  = libphx.Metric_DrawCalls,
            Immediate  = libphx.Metric_Immediate,
            PolysDrawn = libphx.Metric_PolysDrawn,
            TrisDrawn  = libphx.Metric_TrisDrawn,
            VertsDrawn = libphx.Metric_VertsDrawn,
            Flush      = libphx.Metric_Flush,
            FBOSwap    = libphx.Metric_FBOSwap,
            SIZE       = libphx.Metric_SIZE,

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
