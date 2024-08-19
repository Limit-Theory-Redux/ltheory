local Component = require('Component')

---@class Transform
---@field position Vec3f
---@field rotation Vec3f
---@field scale Vec3f

---@class TransformComponent: Component
---@overload fun(name: string): TransformComponent subclass external
local TransformComponent = Subclass(Component, function(self)
    ---@cast self TransformComponent
    self:setComponentName("Transform")

    -- Set Component Archetype
    self:setArchetype(Enums.ComponentArchetype.Transform)

    self:setTransform({
        position = Vec3f(0, 0, 0),
        rotation = Vec3f(0, 0, 0),
        scale = Vec3f(1, 1, 1)
    })
end)

---@param transform Transform
function TransformComponent:setTransform(transform)
    self.transform = transform
end

---@return Transform
function TransformComponent:getTransform()
    return self.transform
end

---@param position Vec3f
function TransformComponent:setPosition(position)
    self.transform.position = position
end

---@param position Vec3f
function TransformComponent:getPosition(position)
    return self.transform.position
end

---@param rotation Vec3f
function TransformComponent:setRotation(rotation)
    self.transform.rotation = rotation
end

---@param rotation Vec3f
function TransformComponent:getRotation(rotation)
    return self.transform.rotation
end

---@param scale Vec3f
function TransformComponent:setScale(scale)
    self.transform.scale = scale
end

---@param scale Vec3f
function TransformComponent:getScale(scale)
    return self.transform.scale
end

return TransformComponent
