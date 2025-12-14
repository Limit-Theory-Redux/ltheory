local Component = require("Core.ECS.Component")

---@class MetallicityComponent: Component
---@overload fun(self: MetallicityComponent, metallicity: number): AgeComponent subclass internal
---@overload fun(metallicity: number): MetallicityComponent subclass external
local MetallicityComponent = Subclass("MetallicityComponent", Component, function(self, metallicity)
    self:setComponentName("CelestialMetallicityComponent")

    self:setMetallicity(metallicity)
end)

---@param metallicity number
function MetallicityComponent:setMetallicity(metallicity)
    self.metallicity = metallicity
end

---@return number|nil metallicity
function MetallicityComponent:getMetallicity()
    return self.metallicity
end

return MetallicityComponent
