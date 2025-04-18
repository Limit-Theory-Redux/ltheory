local Component = require('Components.Component')

---@class Transform
---@field position Position
---@field rotation Quat
---@field scale number

---@class TransformComponent: Component
---@overload fun(self:TransformComponent): TransformComponent subclass internal
---@overload fun(): TransformComponent subclass external
local TransformComponent = Subclass("TransformComponent", Component, function(self)
    self:setComponentName("PhysicsTransform")

    self:setTransform({
        position = Position(),
        rotation = Quat.Identity(),
        scale = 1.0
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

---@param scale number
function TransformComponent:setScale(scale)
    self.transform.scale = scale
end

---@return number
function TransformComponent:getScale()
    return self.transform.scale
end

return TransformComponent
