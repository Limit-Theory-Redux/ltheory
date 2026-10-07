-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class BindGroupDesc
BindGroupDesc = {}

-- Start a bind group for `shader`'s group `group` (0..3).
---@param shader Shader
---@param group integer
---@return BindGroupDesc
function BindGroupDesc.Create(shader, group) end

-- Bind `view` with `sampler` to the sampler `name` the shader declares
-- in this group.
---@param name string
---@param view TexView
---@param sampler integer
function BindGroupDesc:texture(name, view, sampler) end

