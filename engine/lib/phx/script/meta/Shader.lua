-- AUTO GENERATED. DO NOT MODIFY!
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

function Shader:resetTexIndex() end

---@param name string
---@param value number
function Shader:setFloat(name, value) end

---@param index integer
---@param value number
function Shader:iSetFloat(index, value) end

---@param name string
---@param x number
---@param y number
function Shader:setFloat2(name, x, y) end

---@param index integer
---@param x number
---@param y number
function Shader:iSetFloat2(index, x, y) end

---@param name string
---@param x number
---@param y number
---@param z number
function Shader:setFloat3(name, x, y, z) end

---@param index integer
---@param x number
---@param y number
---@param z number
function Shader:iSetFloat3(index, x, y, z) end

---@param name string
---@param x number
---@param y number
---@param z number
---@param w number
function Shader:setFloat4(name, x, y, z, w) end

---@param index integer
---@param x number
---@param y number
---@param z number
---@param w number
function Shader:iSetFloat4(index, x, y, z, w) end

---@param name string
---@param value integer
function Shader:setInt(name, value) end

---@param index integer
---@param value integer
function Shader:iSetInt(index, value) end

---@param name string
---@param x integer
---@param y integer
function Shader:setInt2(name, x, y) end

---@param index integer
---@param x integer
---@param y integer
function Shader:iSetInt2(index, x, y) end

---@param name string
---@param x integer
---@param y integer
---@param z integer
function Shader:setInt3(name, x, y, z) end

---@param index integer
---@param x integer
---@param y integer
---@param z integer
function Shader:iSetInt3(index, x, y, z) end

---@param name string
---@param x integer
---@param y integer
---@param z integer
---@param w integer
function Shader:setInt4(name, x, y, z, w) end

---@param index integer
---@param x integer
---@param y integer
---@param z integer
---@param w integer
function Shader:iSetInt4(index, x, y, z, w) end

---@param name string
---@param value Matrix
function Shader:setMatrix(name, value) end

---@param index integer
---@param value Matrix
function Shader:iSetMatrix(index, value) end

---@param name string
---@param value Matrix
function Shader:setMatrixT(name, value) end

---@param index integer
---@param value Matrix
function Shader:iSetMatrixT(index, value) end

---@param name string
---@param value Tex1D
function Shader:setTex1D(name, value) end

---@param index integer
---@param value Tex1D
function Shader:iSetTex1D(index, value) end

---@param name string
---@param value Tex2D
function Shader:setTex2D(name, value) end

---@param index integer
---@param value Tex2D
function Shader:iSetTex2D(index, value) end

---@param name string
---@param value Tex3D
function Shader:setTex3D(name, value) end

---@param index integer
---@param value Tex3D
function Shader:iSetTex3D(index, value) end

---@param name string
---@param value TexCube
function Shader:setTexCube(name, value) end

---@param index integer
---@param value TexCube
function Shader:iSetTexCube(index, value) end

function Shader:start() end

function Shader:stop() end

