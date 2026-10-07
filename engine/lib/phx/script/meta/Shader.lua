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

-- A LuaJIT `ffi.typeof` struct declaration with the byte layout of the
-- shader's uniform block `name` (empty if the shader has no such block).
-- The Lua side wraps it as `shader:blockType(name)`.
---@param name string
---@return string
function Shader:blockDecl(name) end

-- A hash of the layout of the uniform block `name` (0 if absent). A hot
-- reload that changes it invalidates every ctype and parameter copy made
-- from the old block (`BlockLayout::layout_hash`).
---@param name string
---@return integer
function Shader:blockHash(name) end

-- The names of the shader's uniform blocks, one per line.
---@return string
function Shader:blockNames() end

-- Size in bytes of the uniform block `name` (0 if absent).
---@param name string
---@return integer
function Shader:blockSize(name) end

-- Bumped each time hot reload relinks the shader (it is part of the
-- key of every pipeline made with it).
---@return integer
function Shader:generation() end

-- The shader's GPU resource id (as a plain scalar: `ResourceId` itself
-- is not an FFI type), e.g. for caches keyed by the shader's program
-- (`Render.Pipelines`; a hot reload gives the shader a new resource).
-- Unlike `Mesh::resource_id`, this is a plain getter - `ShaderShared::handle` is always created eagerly in
-- `new`/`from_preprocessed`, never lazily.
---@return integer
function Shader:resourceId() end

---@return Shader
function Shader:clone() end

