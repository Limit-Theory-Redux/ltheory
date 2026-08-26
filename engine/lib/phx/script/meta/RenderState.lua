-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class RenderState
RenderState = {}

---@param r Renderer
function RenderState.PushAllDefaults(r) end

---@param r Renderer
---@param value BlendMode
function RenderState.PushBlendMode(r, value) end

---@param r Renderer
---@param value CullFace
function RenderState.PushCullFace(r, value) end

---@param r Renderer
---@param value boolean
function RenderState.PushDepthTest(r, value) end

---@param r Renderer
---@param value boolean
function RenderState.PushDepthWritable(r, value) end

---@param r Renderer
---@param value boolean
function RenderState.PushWireframe(r, value) end

---@param r Renderer
function RenderState.PopAll(r) end

---@param r Renderer
function RenderState.PopBlendMode(r) end

---@param r Renderer
function RenderState.PopWireframe(r) end

---@param r Renderer
function RenderState.PopDepthTest(r) end

---@param r Renderer
function RenderState.PopCullFace(r) end

---@param r Renderer
function RenderState.PopDepthWritable(r) end

