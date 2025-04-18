-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class SDF
SDF = {}

---@param sx integer
---@param sy integer
---@param sz integer
---@return SDF
function SDF.Create(sx, sy, sz) end

---@param tex Tex3D
---@return SDF
function SDF.FromTex3D(tex) end

---@return Mesh
function SDF:toMesh() end

---@param value number
function SDF:clear(value) end

function SDF:computeNormals() end

---@param x integer
---@param y integer
---@param z integer
---@param value number
function SDF:set(x, y, z, value) end

---@param x integer
---@param y integer
---@param z integer
---@param normal Vec3f
function SDF:setNormal(x, y, z, normal) end

