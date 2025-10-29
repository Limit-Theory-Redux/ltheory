local Component = require("Core.ECS.Component")

---@class CompositionComponent: Component
---@overload fun(self: CompositionComponent, composition: string): CompositionComponent subclass internal
---@overload fun(composition: string): CompositionComponent subclass external
local CompositionComponent = Subclass("CompositionComponent", Component, function(self, composition)
    self:setComponentName("CelestialCompositionComponent")
    self:setComposition(composition)
end)

---@param composition string
function CompositionComponent:setComposition(composition)
    self.composition = composition
end

---@return string|nil composition
function CompositionComponent:getComposition()
    return self.composition
end

return CompositionComponent
