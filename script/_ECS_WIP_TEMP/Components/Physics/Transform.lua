local Component = require('_ECS_WIP_TEMP.Components.Component')

---@class Transform
---@field position Position
---@field rotation Quat
---@field scale Vec3f

---@class TransformComponent: Component
local TransformComponent = Subclass(Component, function(self)
    ---@cast self TransformComponent
    self:setComponentName("PhysicsTransform")

    -- Set Component Archetype
    self:setArchetype(Enums.ComponentArchetype.TransformComponent)

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

---@param position Position
function TransformComponent:setPosition(position)
    self.transform.position = position
end

---@return Position
function TransformComponent:getPosition()
    return self.transform.position
end

---@param rotation Quat
function TransformComponent:setRotation(rotation)
    self.transform.rotation = rotation
end

---@return Quat
function TransformComponent:getRotation()
    return self.transform.rotation
end

---@param scale Vec3f
function TransformComponent:setScale(scale)
    self.transform.scale = scale
end

---@return Vec3f
function TransformComponent:getScale()
    return self.transform.scale
end

return TransformComponent
