local Component = require('_ECS_WIP_TEMP.Components.Component')

---@class MassComponent: Component
---@overload fun(self:MassComponent): MassComponent subclass internal
---@overload fun(): MassComponent subclass external
local MassComponent = Subclass(Component, function(self)
    self:setComponentName("PhysicsMass")

    -- Set Component Archetype
    self:setArchetype(Enums.ComponentArchetype.MassComponent)

    self:init()
end)

function MassComponent:init()
    self.mass = 0
end

---@param mass number
function MassComponent:setMass(mass)
    self.mass = mass
end

---@return number mass
function MassComponent:getMass()
    return self.mass
end

return MassComponent
