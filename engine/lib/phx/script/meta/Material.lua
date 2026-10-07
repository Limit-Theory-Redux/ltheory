-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Material
Material = {}

-- A material drawing with `shader`. `blend` picks the scene bucket and
-- the blending of the pipeline; the other arguments complete its state.
---@param r Renderer
---@param shader Shader
---@param blend BlendMode
---@param cull CullFace
---@param depthTest boolean
---@param depthWrite boolean
---@return Material
function Material.Create(r, shader, blend, cull, depthTest, depthWrite) end

-- Size in bytes of the `MaterialParams` block (0 if the shader has none).
---@return integer
function Material:getParamsSize() end

-- Bind `view` to the group-1 sampler `name` the shader declares, sampled
-- with `sampler`. Takes effect at the next `commit` (the bind group is
-- recreated). The caller keeps the texture alive.
---@param name string
---@param view TexView
---@param sampler integer
function Material:setTexture(name, view, sampler) end

-- Send the parameters to the GPU (one `WriteBuffer`) and recreate the
-- bind group if a texture changed. Not allowed while a pass is open.
---@param r Renderer
function Material:commit(r) end

-- The scene bucket (blend mode) the material draws in.
---@return BlendMode
function Material:getBlend() end

