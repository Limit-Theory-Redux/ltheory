---@meta

---@class LodMesh
LodMesh = {}

---@return LodMesh
function LodMesh.Create() end

---@return LodMesh
function LodMesh:clone() end

---@param mesh Mesh
---@param distanceMin number
---@param distanceMax number
function LodMesh:add(mesh, distanceMin, distanceMax) end

---@param distanceSquared number
function LodMesh:draw(distanceSquared) end

---@param distanceSquared number
---@return Mesh?
function LodMesh:get(distanceSquared) end

