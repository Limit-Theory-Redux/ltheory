-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Intersect
Intersect = {}

---@param src Matrix
---@param dst Matrix
---@return boolean
function Intersect.PointBox(src, dst) end

---@param p Vec3f
---@param tri Triangle
---@return boolean
function Intersect.PointTriangleBarycentric(p, tri) end

---@param ray Ray
---@param plane Plane
---@param pHit Position
---@return boolean
function Intersect.RayPlane(ray, plane, pHit) end

---@param ray Ray
---@param tri Triangle
---@param tEpsilon number
---@param tHit number
---@return boolean
function Intersect.RayTriangleBarycentric(ray, tri, tEpsilon, tHit) end

---@param ray Ray
---@param tri Triangle
---@param tHit number
---@return boolean
function Intersect.RayTriangleMoller1(ray, tri, tHit) end

---@param ray Ray
---@param tri Triangle
---@param tHit number
---@return boolean
function Intersect.RayTriangleMoller2(ray, tri, tHit) end

---@param lineSegment LineSegment
---@param plane Plane
---@param pHit Position
---@return boolean
function Intersect.LineSegmentPlane(lineSegment, plane, pHit) end

---@param a Vec4f
---@param b Vec4f
---@return boolean
function Intersect.RectRect(a, b) end

---@param a Vec4f
---@param b Vec4f
---@return boolean
function Intersect.RectRectFast(a, b) end

---@param sphere Sphere
---@param triangle Triangle
---@param pHit Vec3f
---@return boolean
function Intersect.SphereTriangle(sphere, triangle, pHit) end

