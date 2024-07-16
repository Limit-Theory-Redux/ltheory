-- UpdatePass ------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 UpdatePass;
    ]]

    return 2, 'UpdatePass'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local UpdatePass

    do -- C Definitions
        ffi.cdef [[
            UpdatePass UpdatePass_PreSim;
            UpdatePass UpdatePass_Sim;
            UpdatePass UpdatePass_PostSim;
            UpdatePass UpdatePass_PreFrame;
            UpdatePass UpdatePass_Frame;
            UpdatePass UpdatePass_PostFrame;
            UpdatePass UpdatePass_FrameInterpolation;
            UpdatePass UpdatePass_PreInput;
            UpdatePass UpdatePass_Input;
            UpdatePass UpdatePass_PostInput;

            cstr       UpdatePass_ToString(UpdatePass);
        ]]
    end

    do -- Global Symbol Table
        UpdatePass = {
            PreSim             = libphx.UpdatePass_PreSim,
            Sim                = libphx.UpdatePass_Sim,
            PostSim            = libphx.UpdatePass_PostSim,
            PreFrame           = libphx.UpdatePass_PreFrame,
            Frame              = libphx.UpdatePass_Frame,
            PostFrame          = libphx.UpdatePass_PostFrame,
            FrameInterpolation = libphx.UpdatePass_FrameInterpolation,
            PreInput           = libphx.UpdatePass_PreInput,
            Input              = libphx.UpdatePass_Input,
            PostInput          = libphx.UpdatePass_PostInput,

            ToString           = libphx.UpdatePass_ToString,
        }

        if onDef_UpdatePass then onDef_UpdatePass(UpdatePass, mt) end
        UpdatePass = setmetatable(UpdatePass, mt)
    end

    return UpdatePass
end

return Loader
