-- AUTO GENERATED. DO NOT MODIFY!
-- RendererState ---------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 RendererState;
    ]]

    return 2, 'RendererState'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local RendererState

    do -- C Definitions
        ffi.cdef [[
            cstr          RendererState_ToString(RendererState);
        ]]
    end

    do -- Global Symbol Table
        RendererState = {
            Started        = 0,
            AlreadyRunning = 1,
            Failed         = 2,

            ToString       = libphx.RendererState_ToString,
        }

        if onDef_RendererState then onDef_RendererState(RendererState, mt) end
        RendererState = setmetatable(RendererState, mt)
    end

    return RendererState
end

return Loader
