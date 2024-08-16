local Component = require('Component')

---@class RigidBodyComponent: Component
---@overload fun(name: string): RigidBodyComponent subclass external
local RigidBodyComponent = Subclass(Component, function(self)
    ---@cast self RigidBodyComponent
    self:setComponentName("PhysicsRigidBody")

    -- Set Component Archetype
    self:setArchetype(Enums.ComponentArchetype.Transform)

    self:addRigidBody()
end)

function RigidBodyComponent:addRigidBody()
    self.rigidBody = nil
end

---@param rigidBody RigidBody
function RigidBodyComponent:setRigidBody(rigidBody)
    self.rigidBody = rigidBody
end

---@return RigidBody
function RigidBodyComponent:getRigidBody()
    return self.rigidBody
end

return RigidBodyComponent
