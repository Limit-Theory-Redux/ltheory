-- AUTO GENERATED. DO NOT MODIFY!
-- TexUsage --------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint32 TexUsage;
    ]]

    return 2, 'TexUsage'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local TexUsage

    do -- C Definitions
        ffi.cdef [[
            cstr     TexUsage_ToString(TexUsage);
        ]]
    end

    do -- Global Symbol Table
        TexUsage = {
            Sampled    = 1,
            Attachment = 2,
            CopySrc    = 4,
            CopyDst    = 8,

            ToString   = libphx.TexUsage_ToString,
        }

        if onDef_TexUsage then onDef_TexUsage(TexUsage, mt) end
        TexUsage = setmetatable(TexUsage, mt)
    end

    return TexUsage
end

return Loader
