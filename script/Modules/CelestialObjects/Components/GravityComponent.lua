local Component = require("Core.ECS.Component")

---@class GravityComponent: Component
---@overload fun(self: GravityComponent, gravity: number): GravityComponent subclass internal
---@overload fun(gravity: number): GravityComponent subclass external
local GravityComponent = Subclass("GravityComponent", Component, function(self, gravity)
    self:setComponentName("CelestialGravityComponent")
    self:setGravity(gravity)
end)

---@param gravity number
function GravityComponent:setGravity(gravity)
    self.gravity = gravity
end

---@return number|nil gravity
function GravityComponent:getGravity()
    return self.gravity
end

return GravityComponent
