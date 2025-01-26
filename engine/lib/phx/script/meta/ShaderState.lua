-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class ShaderState
ShaderState = {}

---@param shader Shader
---@return ShaderState
function ShaderState.Create(shader) end

---@param vsName string
---@param fsName string
---@return ShaderState
function ShaderState.FromShaderLoad(vsName, fsName) end

---@param name string
---@param x number
function ShaderState:setFloat(name, x) end

---@param name string
---@param x number
---@param y number
function ShaderState:setFloat2(name, x, y) end

---@param name string
---@param x number
---@param y number
---@param z number
function ShaderState:setFloat3(name, x, y, z) end

---@param name string
---@param x number
---@param y number
---@param z number
---@param w number
function ShaderState:setFloat4(name, x, y, z, w) end

---@param name string
---@param x integer
function ShaderState:setInt(name, x) end

---@param name string
---@param x integer
---@param y integer
function ShaderState:setInt2(name, x, y) end

---@param name string
---@param x integer
---@param y integer
---@param z integer
function ShaderState:setInt3(name, x, y, z) end

---@param name string
---@param x integer
---@param y integer
---@param z integer
---@param w integer
function ShaderState:setInt4(name, x, y, z, w) end

---@param name string
---@param m Matrix
function ShaderState:setMatrix(name, m) end

---@param name string
---@param t Tex1D
function ShaderState:setTex1D(name, t) end

---@param name string
---@param t Tex2D
function ShaderState:setTex2D(name, t) end

---@param name string
---@param t Tex3D
function ShaderState:setTex3D(name, t) end

---@param name string
---@param t TexCube
function ShaderState:setTexCube(name, t) end

function ShaderState:start() end

function ShaderState:stop() end

---@return Shader
function ShaderState:shader() end

