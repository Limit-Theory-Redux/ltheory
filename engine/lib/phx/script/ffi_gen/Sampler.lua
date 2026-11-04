-- AUTO GENERATED. DO NOT MODIFY!
-- Sampler ---------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    return 0, 'Sampler'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Sampler

    do -- C Definitions
        ffi.cdef [[
            uint32 Sampler_Get (Renderer* r, SamplerDesc const* desc);
        ]]
    end

    do -- Global Symbol Table
        Sampler = {
            Get = libphx.Sampler_Get,
        }

        if onDef_Sampler then onDef_Sampler(Sampler, mt) end
        Sampler = setmetatable(Sampler, mt)
    end

    return Sampler
end

return Loader
