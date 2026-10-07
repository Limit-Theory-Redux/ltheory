-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class GenDesc
GenDesc = {}

-- Generation with `shader` (a `fullscreen_ndc` vertex shader and a
-- generating fragment shader).
---@param shader Shader
---@return GenDesc
function GenDesc.Create(shader) end

-- Profiler and pass label.
---@param label string
function GenDesc:label(label) end

-- The bytes of the shader's `Params` block (a `blockType('Params')`
-- cdata). Copied; the face or slice members are overwritten per draw.
---@param bytes integer[]
---@param bytes_size integer
function GenDesc:params(bytes, bytes_size) end

-- Sampler input `slot` (0..3, group 3).
---@param slot integer
---@param view TexView
---@param sampler integer
function GenDesc:input(slot, view, sampler) end

