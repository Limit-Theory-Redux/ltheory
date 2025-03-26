-- AUTO GENERATED. DO NOT MODIFY!
-- FrameStage ------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 FrameStage;
    ]]

    return 2, 'FrameStage'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local FrameStage

    do -- C Definitions
        ffi.cdef [[
            cstr       FrameStage_ToString(FrameStage);
        ]]
    end

    do -- Global Symbol Table
        FrameStage = {
            PreSim     = 0,
            Sim        = 1,
            PostSim    = 2,
            PreRender  = 3,
            Render     = 4,
            PostRender = 5,
            PreInput   = 6,
            Input      = 7,
            PostInput  = 8,

            ToString   = libphx.FrameStage_ToString,
        }

        if onDef_FrameStage then onDef_FrameStage(FrameStage, mt) end
        FrameStage = setmetatable(FrameStage, mt)
    end

    return FrameStage
end

return Loader
