local Component = require("Core.ECS.Component")

---@class TypeComponent: Component
---@field subtype string|nil The enum-based subtype (e.g., "Rocky", "SingleStar")
---@overload fun(self: TypeComponent, subtype: string|nil): TypeComponent subclass internal
---@overload fun(subtype: string|nil): TypeComponent subclass external
local TypeComponent = Subclass("TypeComponent", Component, function(self, subtype)
    self:setComponentName("TypeComponent")
    self:setSubtype(subtype)
end)

---@param subtype string|nil
function TypeComponent:setSubtype(subtype)
    self.subtype = subtype
end

---@return string|nil
function TypeComponent:getSubtype()
    return self.subtype
end

return TypeComponent
