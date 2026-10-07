-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class RenderPass
RenderPass = {}

---@param r Renderer
function RenderPass:finish(r) end

-- Bind a pipeline (shader and fixed-function state) for the draws that
-- follow.
---@param r Renderer
---@param pipeline integer
function RenderPass:setPipeline(r, pipeline) end

-- Stage pass input `slot` (0..3, group 3: texture units 12..15). All
-- staged inputs are bound together before the next draw.
---@param r Renderer
---@param slot integer
---@param view TexView
---@param sampler integer
function RenderPass:setInput(r, slot, view, sampler) end

-- Unbind pass input `slot`.
---@param r Renderer
---@param slot integer
function RenderPass:clearInput(r, slot) end

-- Bind a bind group created with `Renderer:createBindGroup` to its
-- group's units.
---@param r Renderer
---@param group integer
---@param bindGroup integer
function RenderPass:setBindGroup(r, group, bindGroup) end

-- Draw `mesh` with the current pipeline.
---@param r Renderer
---@param mesh Mesh
function RenderPass:drawMesh(r, mesh) end

-- Draw the built-in unit quad (pipeline vertex layout `Fullscreen`),
-- scaled to the viewport by the vertex shader.
---@param r Renderer
function RenderPass:drawFullscreen(r) end

-- Restrict drawing to a sub-rectangle of the target. The UI projection
-- follows the new size, like the viewport stack of old did.
---@param r Renderer
---@param x integer
---@param y integer
---@param width integer
---@param height integer
function RenderPass:setViewport(r, x, y, width, height) end

---@param r Renderer
---@param x integer
---@param y integer
---@param width integer
---@param height integer
function RenderPass:setScissor(r, x, y, width, height) end

---@param r Renderer
function RenderPass:clearScissor(r) end

-- Replace the model-view part of the UI transform (`mWorldViewUI`) for
-- the draws that follow.
---@param r Renderer
---@param transform Matrix
function RenderPass:setUiTransform(r, transform) end

