local Component = require("Core.ECS.Component")

---Where an entity is and how it moves while it has no physics body: its position, velocity (game
---units a second), rotation, and the share of its velocity it loses a second. `PresenceSystem`
---moves it; while the entity is materialized it follows its body or drives it (`PresenceComponent`).
---@class VirtualBodyComponent: Component
---@field position Position
---@field velocity Vec3d
---@field rotation Quat
---@field drag number Share of the velocity lost a second
---@overload fun(self: VirtualBodyComponent, position: Position, velocity: Vec3d|nil, rotation: Quat|nil, drag: number|nil): VirtualBodyComponent subclass internal
---@overload fun(position: Position, velocity: Vec3d|nil, rotation: Quat|nil, drag: number|nil): VirtualBodyComponent subclass external
local VirtualBodyComponent = Subclass("VirtualBodyComponent", Component, function(self, position, velocity, rotation, drag)
    self:setComponentName("VirtualBody")

    self.position = position
    self.velocity = velocity or Vec3d(0, 0, 0)
    self.rotation = rotation or Quat.Identity()
    self.drag = drag or 0
end)

---@return Position
function VirtualBodyComponent:getPosition()
    return self.position
end

---@param position Position
function VirtualBodyComponent:setPosition(position)
    self.position = position
end

---@return Vec3d
function VirtualBodyComponent:getVelocity()
    return self.velocity
end

---@param velocity Vec3d
function VirtualBodyComponent:setVelocity(velocity)
    self.velocity = velocity
end

---@return Quat
function VirtualBodyComponent:getRotation()
    return self.rotation
end

---@param rotation Quat
function VirtualBodyComponent:setRotation(rotation)
    self.rotation = rotation
end

---@return number
function VirtualBodyComponent:getDrag()
    return self.drag
end

return VirtualBodyComponent
