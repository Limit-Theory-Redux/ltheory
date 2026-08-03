-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class ShaderVar
ShaderVar = {}

---@param r Renderer
---@param name string
---@param x number
function ShaderVar.PushFloat(r, name, x) end

---@param r Renderer
---@param name string
---@param x number
---@param y number
function ShaderVar.PushFloat2(r, name, x, y) end

---@param r Renderer
---@param name string
---@param x number
---@param y number
---@param z number
function ShaderVar.PushFloat3(r, name, x, y, z) end

---@param r Renderer
---@param name string
---@param x number
---@param y number
---@param z number
---@param w number
function ShaderVar.PushFloat4(r, name, x, y, z, w) end

---@param r Renderer
---@param name string
---@param x integer
function ShaderVar.PushInt(r, name, x) end

---@param r Renderer
---@param name string
---@param x integer
---@param y integer
function ShaderVar.PushInt2(r, name, x, y) end

---@param r Renderer
---@param name string
---@param x integer
---@param y integer
---@param z integer
function ShaderVar.PushInt3(r, name, x, y, z) end

---@param r Renderer
---@param name string
---@param x integer
---@param y integer
---@param z integer
---@param w integer
function ShaderVar.PushInt4(r, name, x, y, z, w) end

---@param r Renderer
---@param name string
---@param m Matrix
function ShaderVar.PushMatrix(r, name, m) end

---@param r Renderer
---@param name string
---@param t Tex1D
function ShaderVar.PushTex1D(r, name, t) end

---@param r Renderer
---@param name string
---@param t Tex2D
function ShaderVar.PushTex2D(r, name, t) end

---@param r Renderer
---@param name string
---@param t Tex3D
function ShaderVar.PushTex3D(r, name, t) end

---@param r Renderer
---@param name string
---@param t TexCube
function ShaderVar.PushTexCube(r, name, t) end

---@param r Renderer
---@param name string
function ShaderVar.Pop(r, name) end

