-- AUTO GENERATED. DO NOT MODIFY!
-- RenderPass ------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct RenderPass {} RenderPass;
    ]]

    return 1, 'RenderPass'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local RenderPass

    do -- C Definitions
        ffi.cdef [[
            void RenderPass_Free   (RenderPass*);
            void RenderPass_Finish (RenderPass*, Renderer* r);
        ]]
    end

    do -- Global Symbol Table
        RenderPass = {}

        if onDef_RenderPass then onDef_RenderPass(RenderPass, mt) end
        RenderPass = setmetatable(RenderPass, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('RenderPass')
        local mt = {
            __index = {
                finish = libphx.RenderPass_Finish,
            },
        }

        if onDef_RenderPass_t then onDef_RenderPass_t(t, mt) end
        RenderPass_t = ffi.metatype(t, mt)
    end

    return RenderPass
end

return Loader
