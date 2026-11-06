-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class PipelineDesc
PipelineDesc = {}

-- A description with the defaults: opaque, no culling, no depth test,
-- writes on, filled triangles from a `Mesh`.
---@param shader Shader
---@return PipelineDesc
function PipelineDesc.Create(shader) end

---@param blend BlendMode
function PipelineDesc:blend(blend) end

---@param cull CullFace
function PipelineDesc:cull(cull) end

---@param test boolean
---@param write boolean
---@param compare CompareFn
function PipelineDesc:depth(test, write, compare) end

---@param topology Topology
function PipelineDesc:topology(topology) end

---@param vertex VertexLayout
function PipelineDesc:vertex(vertex) end

---@param polygon PolygonMode
function PipelineDesc:polygon(polygon) end
