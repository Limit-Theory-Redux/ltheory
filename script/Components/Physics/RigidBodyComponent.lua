local Component = require('Components.Component')

---@class RigidBodyComponent: Component
---@overload fun(self:RigidBodyComponent): RigidBodyComponent subclass internal
---@overload fun(): RigidBodyComponent subclass external
local RigidBodyComponent = Subclass("RigidBodyComponent", Component, function(self)
    self:setComponentName("PhysicsRigidBody")
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
