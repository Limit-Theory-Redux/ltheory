local Component = require("Core.ECS.Component")

---@class TypeComponent: Component
---@overload fun(self: TypeComponent, type: integer): TypeComponent subclass internal
---@overload fun(type: integer|nil): TypeComponent subclass external
local TypeComponent = Subclass("TypeComponent", Component, function(self, type)
    self:setComponentName("TypeComponent")

    self:setType(type)
end)

---@param type integer
function TypeComponent:setType(type)
    self.type = type
end

---@return integer
function TypeComponent:getType()
    return self.type
end

return TypeComponent
