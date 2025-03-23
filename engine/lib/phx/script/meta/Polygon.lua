-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Polygon
Polygon = {}

---@return Plane
function Polygon:toPlane() end

---@return Plane
function Polygon:toPlaneFast() end

---@param splitPlane Plane
---@param back Polygon
---@param front Polygon
function Polygon:splitSafe(splitPlane, back, front) end

---@param splitPlane Plane
---@param back Polygon
---@param front Polygon
function Polygon:split(splitPlane, back, front) end

---@return Vec3f
function Polygon:getCentroid() end

---@return Error
function Polygon:validate() end

