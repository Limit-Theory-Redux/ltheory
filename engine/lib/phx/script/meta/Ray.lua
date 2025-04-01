-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Ray
Ray = {}

---@param t number
---@return Position
function Ray:getPoint(t) end

---@param plane Plane
---@param pHit Position
---@return boolean
function Ray:intersectPlane(plane, pHit) end

---@param tri Triangle
---@param tEpsilon number
---@param tHit number
---@return boolean
function Ray:intersectTriangleBarycentric(tri, tEpsilon, tHit) end

---@param tri Triangle
---@param tHit number
---@return boolean
function Ray:intersectTriangleMoller1(tri, tHit) end

---@param tri Triangle
---@param tHit number
---@return boolean
function Ray:intersectTriangleMoller2(tri, tHit) end

---@return LineSegment
function Ray:toLineSegment() end

---@param lineSegment LineSegment
---@return Ray
function Ray.FromLineSegment(lineSegment) end

