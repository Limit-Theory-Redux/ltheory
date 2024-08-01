local Component = require('Component')

---@class NameComponent: Component
---@overload fun(name: string): NameComponent subclass external
local NameComponent = Subclass(Component, function(self, name)
    ---@cast self NameComponent
    self:setComponentName("Name")
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
