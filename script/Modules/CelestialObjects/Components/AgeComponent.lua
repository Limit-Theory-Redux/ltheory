local Component = require("Core.ECS.Component")

---@class AgeComponent: Component
---@overload fun(self: AgeComponent, age: integer): AgeComponent subclass internal
---@overload fun(age: integer): AgeComponent subclass external
local AgeComponent = Subclass("AgeComponent", Component, function(self, age)
    self:setComponentName("CelestialAgeComponent")

    self:setAge(age)
end)

---@param age integer
function AgeComponent:setAge(age)
    self.age = age
end

---@return integer|nil age
function AgeComponent:getAge()
    return self.age
end

return AgeComponent
