-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class BSP
BSP = {}

---@param mesh Mesh
---@return BSP
function BSP.Create(mesh) end

---@param ray Ray
---@param tHit number
---@return boolean
function BSP:intersectRay(ray, tHit) end

---@param lineSegment LineSegment
---@param pHit Vec3f
---@return boolean
function BSP:intersectLineSegment(lineSegment, pHit) end

---@param sphere Sphere
---@param pHit Vec3f
---@return boolean
function BSP:intersectSphere(sphere, pHit) end

---@param nodeRef BSPNodeRef
---@param relationship BSPNodeRel
---@return BSPNodeRef
function BSP:getNode(nodeRef, relationship) end

---@param nodeRef BSPNodeRef
---@param color Color
function BSP:drawNode(nodeRef, color) end

---@param nodeRef BSPNodeRef
function BSP:drawNodeSplit(nodeRef) end

---@param lineSegment LineSegment
---@param eye Position
function BSP:drawLineSegment(lineSegment, eye) end

---@param sphere Sphere
function BSP:drawSphere(sphere) end

---@param totalTime number
function BSP:printRayProfilingData(totalTime) end

---@param totalTime number
function BSP:printSphereProfilingData(totalTime) end

---@param sphere Sphere
---@param sphereProf IntersectSphereProfiling
---@return boolean
function BSP:getIntersectSphereTriangles(sphere, sphereProf) end

---@param leafIndex integer
---@return BSPNodeRef
function BSP:getLeaf(leafIndex) end

