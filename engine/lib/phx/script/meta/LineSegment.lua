---@meta

LineSegment = LineSegment

---@param out Ray
function LineSegment:toRay(self, out) end

---@param ray Ray
---@param out LineSegment
function LineSegment.FromRay(ray, out) end

---@return string
function LineSegment:toString(self) end

