-- AUTO GENERATED. DO NOT MODIFY!
-- RenderPassDesc --------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct RenderPassDesc {} RenderPassDesc;
    ]]

    return 1, 'RenderPassDesc'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local RenderPassDesc

    do -- C Definitions
        ffi.cdef [[
            void            RenderPassDesc_Free            (RenderPassDesc*);
            RenderPassDesc* RenderPassDesc_Create          (cstr label);
            void            RenderPassDesc_Color           (RenderPassDesc*, int index, TexView const* view, LoadOp load, float r, float g, float b, float a);
            void            RenderPassDesc_Depth           (RenderPassDesc*, TexView const* view, LoadOp load, float d);
            void            RenderPassDesc_Backbuffer      (RenderPassDesc*, int width, int height, LoadOp load, float r, float g, float b, float a);
            void            RenderPassDesc_BackbufferDepth (RenderPassDesc*, LoadOp load, float d);
        ]]
    end

    do -- Global Symbol Table
        RenderPassDesc = {
            Create          = function(label)
                local _instance = libphx.RenderPassDesc_Create(label)
                return Core.ManagedObject(_instance, libphx.RenderPassDesc_Free)
            end,
        }

        if onDef_RenderPassDesc then onDef_RenderPassDesc(RenderPassDesc, mt) end
        RenderPassDesc = setmetatable(RenderPassDesc, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('RenderPassDesc')
        local mt = {
            __index = {
                color           = libphx.RenderPassDesc_Color,
                depth           = libphx.RenderPassDesc_Depth,
                backbuffer      = libphx.RenderPassDesc_Backbuffer,
                backbufferDepth = libphx.RenderPassDesc_BackbufferDepth,
            },
        }

        if onDef_RenderPassDesc_t then onDef_RenderPassDesc_t(t, mt) end
        RenderPassDesc_t = ffi.metatype(t, mt)
    end

    return RenderPassDesc
end

return Loader
