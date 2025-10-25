local Component = require("Core.ECS.Component")

---@class TypeComponent: Component
---@overload fun(self: TypeComponent, name: string): TypeComponent subclass internal
---@overload fun(name: string|nil): TypeComponent subclass external
local TypeComponent = Subclass("TypeComponent", Component, function(self, name)
    self:setComponentName("TypeComponent")

    self:setType(name)
end)

---@param name string
function TypeComponent:setType(name)
    self.type = name
end

---@return string
function TypeComponent:getType()
    return self.type
end

return TypeComponent
