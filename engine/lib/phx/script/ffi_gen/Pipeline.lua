-- AUTO GENERATED. DO NOT MODIFY!
-- Pipeline --------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    return 0, 'Pipeline'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Pipeline

    do -- C Definitions
        ffi.cdef [[
            uint32 Pipeline_Get (Renderer* r, PipelineDesc const* desc);
        ]]
    end

    do -- Global Symbol Table
        Pipeline = {
            Get = libphx.Pipeline_Get,
        }

        if onDef_Pipeline then onDef_Pipeline(Pipeline, mt) end
        Pipeline = setmetatable(Pipeline, mt)
    end

    return Pipeline
end

return Loader
