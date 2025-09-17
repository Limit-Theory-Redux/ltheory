local Component = require("Core.ECS.Component")

---@class RigidBodyComponent: Component
---@overload fun(self:RigidBodyComponent, rigidBody: RigidBody): RigidBodyComponent subclass internal
---@overload fun(): RigidBodyComponent subclass external
local RigidBodyComponent = Subclass("RigidBodyComponent", Component, function(self, rigidBody)
    self:setComponentName("PhysicsRigidBody")
    self:setRigidBody(rigidBody)
end)

---@param rigidBody RigidBody
function RigidBodyComponent:setRigidBody(rigidBody)
    self.rigidBody = rigidBody
end

---@return RigidBody
function RigidBodyComponent:getRigidBody()
    return self.rigidBody
end

return RigidBodyComponent
