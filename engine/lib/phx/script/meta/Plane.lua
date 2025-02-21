-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Plane
Plane = {}

---@param p Vec3f
---@return PointClassification
function Plane:classifyPoint(p) end

---@param polygon Polygon
---@return PolygonClassification
function Plane:classifyPolygon(polygon) end

---@return Error
function Plane:validate() end

---@param polygon Polygon
---@return Plane
function Plane.FromPolygon(polygon) end

---@param polygon Polygon
---@return Plane
function Plane.FromPolygonFast(polygon) end

