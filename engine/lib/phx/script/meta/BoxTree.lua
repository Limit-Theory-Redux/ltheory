-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class BoxTree
BoxTree = {}

---@return BoxTree
function BoxTree.Create() end

---@param mesh Mesh
---@return BoxTree
function BoxTree.FromMesh(mesh) end

---@param box3 Box3f
---@param data integer[]
---@param data_size integer
function BoxTree:add(box3, data, data_size) end

---@return integer
function BoxTree:getMemory() end

---@param matrix Matrix
---@param ro Vec3f
---@param rd Vec3f
---@return boolean
function BoxTree:intersectRay(matrix, ro, rd) end

---@param maxDepth integer
function BoxTree:draw(maxDepth) end

