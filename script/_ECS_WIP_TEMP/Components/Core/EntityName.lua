local Component = require('_ECS_WIP_TEMP.Components.Component') --!temp path

---@class NameComponent: Component
---@overload fun(name: string): NameComponent subclass external
local NameComponent = Subclass(Component, function(self, name)
    ---@cast self NameComponent
    self:setComponentName("Name")

    -- Set Component Archetype
    self:setArchetype(Enums.ComponentArchetype.NameComponent)

    self:setName(name)
end)

---@param name string
function NameComponent:setName(name)
    if not name or type(name) ~= "string" then
        Log.Warn("Did not provide a valid string name for name component")
        return
    end

    self.name = name
end

---@return string
function NameComponent:getName()
    return self.name
end

return NameComponent
