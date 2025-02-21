-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Octree
Octree = {}

---@param box0 Box3f
---@return Octree
function Octree.Create(box0) end

---@param mesh Mesh
---@return Octree
function Octree.FromMesh(mesh) end

---@return number
function Octree:getAvgLoad() end

---@return integer
function Octree:getMaxLoad() end

---@return integer
function Octree:getMemory() end

---@param matrix Matrix
---@param ro Vec3f
---@param rd Vec3f
---@return boolean
function Octree:intersectRay(matrix, ro, rd) end

---@param box0 Box3f
---@param id integer
function Octree:add(box0, id) end

function Octree:draw() end

