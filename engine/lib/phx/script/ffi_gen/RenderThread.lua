-- AUTO GENERATED. DO NOT MODIFY!
-- RenderThread ----------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    return 0, 'RenderThread'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local RenderThread

    do -- C Definitions
        ffi.cdef [[
            bool RenderThread_IsCommandMode      ();
            bool RenderThread_IsGLAvailable      ();
            void RenderThread_EnableCommandMode  ();
            void RenderThread_DisableCommandMode ();
        ]]
    end

    do -- Global Symbol Table
        RenderThread = {
            IsCommandMode      = libphx.RenderThread_IsCommandMode,
            IsGLAvailable      = libphx.RenderThread_IsGLAvailable,
            EnableCommandMode  = libphx.RenderThread_EnableCommandMode,
            DisableCommandMode = libphx.RenderThread_DisableCommandMode,
        }

        if onDef_RenderThread then onDef_RenderThread(RenderThread, mt) end
        RenderThread = setmetatable(RenderThread, mt)
    end

    return RenderThread
end

return Loader
