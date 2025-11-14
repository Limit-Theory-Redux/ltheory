local Component = require("Core.ECS.Component")

---@class TransformComponent: Component
---@field position Position
---@field rotation Quat
---@field scale gameunit
---@field globalPosition Position
---@field globalRotation Quat
---@field globalScale gameunit
---@field dirty boolean
---@overload fun(self:TransformComponent): TransformComponent subclass internal
---@overload fun(): TransformComponent subclass external
local TransformComponent = Subclass("TransformComponent", Component, function(self)
    self:setComponentName("Transform")

    self.posLocal = Position()
    self.rotLocal = Quat.Identity()
    self.scaleLocal = 1.0
    self.pos = Position()
    self.rot = Quat.Identity()
    self.scale = 1.0
    self.dirty = false
end)

---@param position Position
function TransformComponent:setPos(position)
    self.pos = position
    self.dirty = true
end

---@return Position
function TransformComponent:getPos()
    return self.pos
end

---@param rotation Quat
function TransformComponent:setRot(rotation)
    self.rot = rotation
    self.dirty = true
end

---@return Quat
function TransformComponent:getRot()
    return self.rot
end

---@param scale gameunit
function TransformComponent:setScale(scale)
    self.scale = scale
    self.dirty = true
end

---@return gameunit
function TransformComponent:getScale()
    return self.scale
end

---@param position Position
function TransformComponent:setPosLocal(position)
    self.posLocal = position
    self.dirty = true
end

---@return Position
function TransformComponent:getPosLocal()
    return self.posLocal
end

---@param rotation Quat
function TransformComponent:setRotLocal(rotation)
    self.rotLocal = rotation
    self.dirty = true
end

---@return Quat
function TransformComponent:getRotLocal()
    return self.rot
end

---@param scale gameunit
function TransformComponent:setScaleLocal(scale)
    self.scaleLocal = scale
    self.dirty = true
end

---@return gameunit
function TransformComponent:getScaleLocal()
    return self.scaleLocal
end

function TransformComponent:isDirty()
    return self.dirty
end

---@param dirty boolean
function TransformComponent:setDirty(dirty)
    self.dirty = dirty
end

return TransformComponent
