-- AUTO GENERATED. DO NOT MODIFY!
-- PipelineDesc ----------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct PipelineDesc {} PipelineDesc;
    ]]

    return 1, 'PipelineDesc'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local PipelineDesc

    do -- C Definitions
        ffi.cdef [[
            void          PipelineDesc_Free        (PipelineDesc*);
            PipelineDesc* PipelineDesc_Create      (Shader const* shader);
            void          PipelineDesc_Blend       (PipelineDesc*, BlendMode blend);
            void          PipelineDesc_Cull        (PipelineDesc*, CullFace cull);
            void          PipelineDesc_Depth       (PipelineDesc*, bool test, bool write, CompareFn compare);
            void          PipelineDesc_Topology    (PipelineDesc*, Topology topology);
            void          PipelineDesc_Vertex      (PipelineDesc*, VertexLayout vertex);
            void          PipelineDesc_Polygon     (PipelineDesc*, PolygonMode polygon);
            void          PipelineDesc_ColorFormat (PipelineDesc*, int index, TexFormat format);
            void          PipelineDesc_DepthFormat (PipelineDesc*, TexFormat format);
        ]]
    end

    do -- Global Symbol Table
        PipelineDesc = {
            Create      = function(shader)
                local _instance = libphx.PipelineDesc_Create(shader)
                return Core.ManagedObject(_instance, libphx.PipelineDesc_Free)
            end,
        }

        if onDef_PipelineDesc then onDef_PipelineDesc(PipelineDesc, mt) end
        PipelineDesc = setmetatable(PipelineDesc, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('PipelineDesc')
        local mt = {
            __index = {
                blend       = libphx.PipelineDesc_Blend,
                cull        = libphx.PipelineDesc_Cull,
                depth       = libphx.PipelineDesc_Depth,
                topology    = libphx.PipelineDesc_Topology,
                vertex      = libphx.PipelineDesc_Vertex,
                polygon     = libphx.PipelineDesc_Polygon,
                colorFormat = libphx.PipelineDesc_ColorFormat,
                depthFormat = libphx.PipelineDesc_DepthFormat,
            },
        }

        if onDef_PipelineDesc_t then onDef_PipelineDesc_t(t, mt) end
        PipelineDesc_t = ffi.metatype(t, mt)
    end

    return PipelineDesc
end

return Loader
