---@meta

---@class LineSegment
LineSegment = {}

---@param out Ray
function LineSegment:toRay(out) end

---@param ray Ray
---@param out LineSegment
function LineSegment.FromRay(ray, out) end

---@return string
function LineSegment:getString() end

