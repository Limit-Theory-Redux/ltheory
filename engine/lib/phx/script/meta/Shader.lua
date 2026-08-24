-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Shader
Shader = {}

---@param r Renderer
---@param vs string
---@param fs string
---@return Shader
function Shader.Create(r, vs, fs) end

---@param r Renderer
---@param vsName string
---@param fsName string
---@return Shader
function Shader.Load(r, vsName, fsName) end

-- Reload shader from disk. Returns true on success.
-- On compile/link failure, keeps the old shader and returns false.
---@param r Renderer
---@return boolean
function Shader:reload(r) end

---@return string
function Shader:name() end

-- The shader's GPU resource id (as a plain scalar - see
-- `Renderer::add_entity`'s `mesh_id`/`shader_id` params for why this
-- isn't `ResourceId` itself), e.g. for code that needs to reference the
-- shader instead of calling `start`/`stop` itself (the batch API,
-- `Renderer:addEntity`). Unlike `Mesh::resource_id`, this is a plain
-- getter - `ShaderShared::handle` is always created eagerly in
-- `new`/`from_preprocessed`, never lazily.
---@return integer
function Shader:resourceId() end

---@return Shader
function Shader:clone() end

---@return ShaderState
function Shader:toShaderState() end

---@param r Renderer
---@param name string
---@return integer
function Shader:getVariable(r, name) end

---@param r Renderer
---@param name string
---@return boolean
function Shader:hasVariable(r, name) end

function Shader:resetTexIndex() end

---@param r Renderer
---@param name string
---@param value number
function Shader:setFloat(r, name, value) end

---@param r Renderer
---@param index integer
---@param value number
function Shader:iSetFloat(r, index, value) end

---@param r Renderer
---@param name string
---@param x number
---@param y number
function Shader:setFloat2(r, name, x, y) end

---@param r Renderer
---@param index integer
---@param x number
---@param y number
function Shader:iSetFloat2(r, index, x, y) end

---@param r Renderer
---@param name string
---@param x number
---@param y number
---@param z number
function Shader:setFloat3(r, name, x, y, z) end

---@param r Renderer
---@param index integer
---@param x number
---@param y number
---@param z number
function Shader:iSetFloat3(r, index, x, y, z) end

---@param r Renderer
---@param name string
---@param x number
---@param y number
---@param z number
---@param w number
function Shader:setFloat4(r, name, x, y, z, w) end

---@param r Renderer
---@param index integer
---@param x number
---@param y number
---@param z number
---@param w number
function Shader:iSetFloat4(r, index, x, y, z, w) end

---@param r Renderer
---@param name string
---@param value integer
function Shader:setInt(r, name, value) end

---@param r Renderer
---@param index integer
---@param value integer
function Shader:iSetInt(r, index, value) end

---@param r Renderer
---@param name string
---@param x integer
---@param y integer
function Shader:setInt2(r, name, x, y) end

---@param r Renderer
---@param index integer
---@param x integer
---@param y integer
function Shader:iSetInt2(r, index, x, y) end

---@param r Renderer
---@param name string
---@param x integer
---@param y integer
---@param z integer
function Shader:setInt3(r, name, x, y, z) end

---@param r Renderer
---@param index integer
---@param x integer
---@param y integer
---@param z integer
function Shader:iSetInt3(r, index, x, y, z) end

---@param r Renderer
---@param name string
---@param x integer
---@param y integer
---@param z integer
---@param w integer
function Shader:setInt4(r, name, x, y, z, w) end

---@param r Renderer
---@param index integer
---@param x integer
---@param y integer
---@param z integer
---@param w integer
function Shader:iSetInt4(r, index, x, y, z, w) end

---@param r Renderer
---@param name string
---@param value Matrix
function Shader:setMatrix(r, name, value) end

---@param r Renderer
---@param index integer
---@param value Matrix
function Shader:iSetMatrix(r, index, value) end

---@param r Renderer
---@param name string
---@param value Matrix
function Shader:setMatrixT(r, name, value) end

---@param r Renderer
---@param index integer
---@param value Matrix
function Shader:iSetMatrixT(r, index, value) end

-- Batched per-instance uniforms: mWorld, mWorldIT and scale in a single
-- command instead of three separate SetUniform* commands. The instance
-- values are unique per mesh (no dedup win), so the three GL uniform
-- calls are batched on the render thread and the producer pays one
-- command + one FFI crossing instead of three of each.
---@param r Renderer
---@param worldIndex integer
---@param worldItIndex integer
---@param scaleIndex integer
---@param world Matrix
---@param worldIt Matrix
---@param scale number
function Shader:indexSetInstanceUniforms(r, worldIndex, worldItIndex, scaleIndex, world, worldIt, scale) end

---@param r Renderer
---@param name string
---@param value Tex1D
function Shader:setTex1D(r, name, value) end

---@param r Renderer
---@param index integer
---@param value Tex1D
function Shader:iSetTex1D(r, index, value) end

---@param r Renderer
---@param name string
---@param value Tex2D
function Shader:setTex2D(r, name, value) end

---@param r Renderer
---@param index integer
---@param value Tex2D
function Shader:iSetTex2D(r, index, value) end

---@param r Renderer
---@param name string
---@param value Tex3D
function Shader:setTex3D(r, name, value) end

---@param r Renderer
---@param index integer
---@param value Tex3D
function Shader:iSetTex3D(r, index, value) end

---@param r Renderer
---@param name string
---@param value TexCube
function Shader:setTexCube(r, name, value) end

---@param r Renderer
---@param index integer
---@param value TexCube
function Shader:iSetTexCube(r, index, value) end

---@param r Renderer
function Shader:start(r) end

---@param r Renderer
function Shader:stop(r) end

