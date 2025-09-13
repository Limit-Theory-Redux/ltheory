local Component = require("Core.ECS.Component")

---@class TransformComponent: Component
---@field position Position
---@field rotation Quat
---@field scale number
---@field globalPosition Position
---@field globalRotation Quat
---@field globalScale number
---@overload fun(self:TransformComponent): TransformComponent subclass internal
---@overload fun(): TransformComponent subclass external
local TransformComponent = Subclass("TransformComponent", Component, function(self)
    self:setComponentName("Transform")

    self.position = Position()
    self.rotation = Quat.Identity()
    self.scale = 1.0
    self.globalPosition = Position()
    self.globalRotation = Quat.Identity()
    self.globalScale = 1.0
end)

---@param position Position
function TransformComponent:setPosition(position)
    self.position = position
end

---@return Position
function TransformComponent:getPosition()
    return self.position
end

---@param rotation Quat
function TransformComponent:setRotation(rotation)
    self.rotation = rotation
end

---@return Quat
function TransformComponent:getRotation()
    return self.rotation
end

---@param scale number
function TransformComponent:setScale(scale)
    self.scale = scale
end

---@return number
function TransformComponent:getScale()
    return self.scale
end

return TransformComponent
