local Component = require('_ECS_WIP_TEMP.Components.Component')

---@class MassComponent: Component
---@overload fun(self:MassComponent, mass: number): MassComponent subclass internal
---@overload fun(mass: number): MassComponent subclass external
local MassComponent = Subclass(Component, function(self, mass)
    self:setComponentName("PhysicsMass")

    -- Set Component Archetype
    self:setArchetype(Enums.ComponentArchetype.MassComponent)

    self:setMass(mass or 0)
end)

---@param mass number
function MassComponent:setMass(mass)
    self.mass = mass
end

---@return number mass
function MassComponent:getMass()
    return self.mass
end

return MassComponent