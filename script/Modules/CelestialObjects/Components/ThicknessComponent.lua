local Component = require("Core.ECS.Component")

---@class ThicknessComponent: Component
---@overload fun(self: ThicknessComponent, thickness: number): ThicknessComponent subclass internal
---@overload fun(thickness: number): ThicknessComponent subclass external
local ThicknessComponent = Subclass("ThicknessComponent", Component, function(self, thickness)
    self:setComponentName("CelestialThicknessComponent")
    self:setThickness(thickness)
end)

---@param thickness number
function ThicknessComponent:setThickness(thickness)
    self.thickness = thickness
end

---@return number|nil thickness
function ThicknessComponent:getThickness()
    return self.thickness
end

return ThicknessComponent
