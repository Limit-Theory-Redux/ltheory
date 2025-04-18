local Component = require('Components.Component')

---@class SpatialShapeComponent: Component
---@overload fun(self:SpatialShapeComponent): SpatialShapeComponent subclass internal
---@overload fun(): SpatialShapeComponent subclass external
local SpatialShapeComponent = Subclass("SpatialShapeComponent", Component, function(self)
    self:setComponentName("Shape")

    self:init()
end)

function SpatialShapeComponent:init()
    self.shape = nil --* add real shapes here
    self.radius = 1.0
end

---@param shape SpatialShape
function SpatialShapeComponent:setShape(shape)
    self.shape = shape
end

---@return number SpatialShape
function SpatialShapeComponent:getShape()
    return self.shape
end

function SpatialShapeComponent:setRadius(radius)
    self.radius = radius
end

---@return number radius
function SpatialShapeComponent:getRadius()
    return self.radius
end

---@return number diameter
function SpatialShapeComponent:getDiameter()
    return self.radius * 2
end

return SpatialShapeComponent
