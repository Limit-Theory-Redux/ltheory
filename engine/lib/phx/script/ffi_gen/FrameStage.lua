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
            FrameStage FrameStage_PreSim;
            FrameStage FrameStage_Sim;
            FrameStage FrameStage_PostSim;
            FrameStage FrameStage_PreRender;
            FrameStage FrameStage_Render;
            FrameStage FrameStage_PostRender;
            FrameStage FrameStage_PreInput;
            FrameStage FrameStage_Input;
            FrameStage FrameStage_PostInput;

            cstr       FrameStage_ToString(FrameStage);
        ]]
    end

    do -- Global Symbol Table
        FrameStage = {
            PreSim     = libphx.FrameStage_PreSim,
            Sim        = libphx.FrameStage_Sim,
            PostSim    = libphx.FrameStage_PostSim,
            PreRender  = libphx.FrameStage_PreRender,
            Render     = libphx.FrameStage_Render,
            PostRender = libphx.FrameStage_PostRender,
            PreInput   = libphx.FrameStage_PreInput,
            Input      = libphx.FrameStage_Input,
            PostInput  = libphx.FrameStage_PostInput,

            ToString   = libphx.FrameStage_ToString,
        }

        if onDef_FrameStage then onDef_FrameStage(FrameStage, mt) end
        FrameStage = setmetatable(FrameStage, mt)
    end

    return FrameStage
end

return Loader
