-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class ShaderVar
ShaderVar = {}

---@param name string
---@param x number
function ShaderVar.PushFloat(name, x) end

---@param name string
---@param x number
---@param y number
function ShaderVar.PushFloat2(name, x, y) end

---@param name string
---@param x number
---@param y number
---@param z number
function ShaderVar.PushFloat3(name, x, y, z) end

---@param name string
---@param x number
---@param y number
---@param z number
---@param w number
function ShaderVar.PushFloat4(name, x, y, z, w) end

---@param name string
---@param x integer
function ShaderVar.PushInt(name, x) end

---@param name string
---@param x integer
---@param y integer
function ShaderVar.PushInt2(name, x, y) end

---@param name string
---@param x integer
---@param y integer
---@param z integer
function ShaderVar.PushInt3(name, x, y, z) end

---@param name string
---@param x integer
---@param y integer
---@param z integer
---@param w integer
function ShaderVar.PushInt4(name, x, y, z, w) end

---@param name string
---@param m Matrix
function ShaderVar.PushMatrix(name, m) end

---@param name string
---@param t Tex1D
function ShaderVar.PushTex1D(name, t) end

---@param name string
---@param t Tex2D
function ShaderVar.PushTex2D(name, t) end

---@param name string
---@param t Tex3D
function ShaderVar.PushTex3D(name, t) end

---@param name string
---@param t TexCube
function ShaderVar.PushTexCube(name, t) end

---@param name string
function ShaderVar.Pop(name) end

