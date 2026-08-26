-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class ShaderState
ShaderState = {}

---@param shader Shader
---@return ShaderState
function ShaderState.Create(shader) end

---@param r Renderer
---@param vsName string
---@param fsName string
---@return ShaderState
function ShaderState.FromShaderLoad(r, vsName, fsName) end

---@param r Renderer
---@param name string
---@param x number
function ShaderState:setFloat(r, name, x) end

---@param r Renderer
---@param name string
---@param x number
---@param y number
function ShaderState:setFloat2(r, name, x, y) end

---@param r Renderer
---@param name string
---@param x number
---@param y number
---@param z number
function ShaderState:setFloat3(r, name, x, y, z) end

---@param r Renderer
---@param name string
---@param x number
---@param y number
---@param z number
---@param w number
function ShaderState:setFloat4(r, name, x, y, z, w) end

---@param r Renderer
---@param name string
---@param x integer
function ShaderState:setInt(r, name, x) end

---@param r Renderer
---@param name string
---@param x integer
---@param y integer
function ShaderState:setInt2(r, name, x, y) end

---@param r Renderer
---@param name string
---@param x integer
---@param y integer
---@param z integer
function ShaderState:setInt3(r, name, x, y, z) end

---@param r Renderer
---@param name string
---@param x integer
---@param y integer
---@param z integer
---@param w integer
function ShaderState:setInt4(r, name, x, y, z, w) end

---@param r Renderer
---@param name string
---@param m Matrix
function ShaderState:setMatrix(r, name, m) end

---@param r Renderer
---@param name string
---@param t Tex1D
function ShaderState:setTex1D(r, name, t) end

---@param r Renderer
---@param name string
---@param t Tex2D
function ShaderState:setTex2D(r, name, t) end

---@param r Renderer
---@param name string
---@param t Tex3D
function ShaderState:setTex3D(r, name, t) end

---@param r Renderer
---@param name string
---@param t TexCube
function ShaderState:setTexCube(r, name, t) end

---@param r Renderer
function ShaderState:start(r) end

---@param r Renderer
function ShaderState:stop(r) end

---@return Shader
function ShaderState:shader() end

