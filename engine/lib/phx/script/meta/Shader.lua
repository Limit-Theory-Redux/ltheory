---@meta

---@class Shader
Shader = {}

---@param vs string
---@param fs string
---@return Shader
function Shader.Create(vs, fs) end

---@param vsName string
---@param fsName string
---@return Shader
function Shader.Load(vsName, fsName) end

---@return Shader
function Shader:clone() end

---@return ShaderState
function Shader:toShaderState() end

---@return integer
function Shader:getHandle() end

---@param name string
---@return integer
function Shader:getVariable(name) end

---@param name string
---@return boolean
function Shader:hasVariable(name) end

function Shader:start() end

function Shader:stop() end

function Shader.ResetTexIndex() end

---@param name string
---@param value number
function Shader.SetFloat(name, value) end

---@param index integer
---@param value number
function Shader.ISetFloat(index, value) end

---@param name string
---@param x number
---@param y number
function Shader.SetFloat2(name, x, y) end

---@param index integer
---@param x number
---@param y number
function Shader.ISetFloat2(index, x, y) end

---@param name string
---@param x number
---@param y number
---@param z number
function Shader.SetFloat3(name, x, y, z) end

---@param index integer
---@param x number
---@param y number
---@param z number
function Shader.ISetFloat3(index, x, y, z) end

---@param name string
---@param x number
---@param y number
---@param z number
---@param w number
function Shader.SetFloat4(name, x, y, z, w) end

---@param index integer
---@param x number
---@param y number
---@param z number
---@param w number
function Shader.ISetFloat4(index, x, y, z, w) end

---@param name string
---@param value integer
function Shader.SetInt(name, value) end

---@param index integer
---@param value integer
function Shader.ISetInt(index, value) end

---@param name string
---@param x integer
---@param y integer
function Shader.SetInt2(name, x, y) end

---@param index integer
---@param x integer
---@param y integer
function Shader.ISetInt2(index, x, y) end

---@param name string
---@param x integer
---@param y integer
---@param z integer
function Shader.SetInt3(name, x, y, z) end

---@param index integer
---@param x integer
---@param y integer
---@param z integer
function Shader.ISetInt3(index, x, y, z) end

---@param name string
---@param x integer
---@param y integer
---@param z integer
---@param w integer
function Shader.SetInt4(name, x, y, z, w) end

---@param index integer
---@param x integer
---@param y integer
---@param z integer
---@param w integer
function Shader.ISetInt4(index, x, y, z, w) end

---@param name string
---@param value Matrix
function Shader.SetMatrix(name, value) end

---@param index integer
---@param value Matrix
function Shader.ISetMatrix(index, value) end

---@param name string
---@param value Matrix
function Shader.SetMatrixT(name, value) end

---@param index integer
---@param value Matrix
function Shader.ISetMatrixT(index, value) end

---@param name string
---@param value Tex1D
function Shader.SetTex1D(name, value) end

---@param index integer
---@param value Tex1D
function Shader.ISetTex1D(index, value) end

---@param name string
---@param value Tex2D
function Shader.SetTex2D(name, value) end

---@param index integer
---@param value Tex2D
function Shader.ISetTex2D(index, value) end

---@param name string
---@param value Tex3D
function Shader.SetTex3D(name, value) end

---@param index integer
---@param value Tex3D
function Shader.ISetTex3D(index, value) end

---@param name string
---@param value TexCube
function Shader.SetTexCube(name, value) end

---@param index integer
---@param value TexCube
function Shader.ISetTexCube(index, value) end

