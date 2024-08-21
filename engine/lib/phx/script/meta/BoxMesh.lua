---@meta

---@class BoxMesh
BoxMesh = {}

---@return BoxMesh
function BoxMesh.Create() end

---@param p Vec3f
---@param s Vec3f
---@param r Vec3f
---@param b Vec3f
function BoxMesh:add(p, s, r, b) end

---@param res integer
---@return Mesh
function BoxMesh:getMesh(res) end

