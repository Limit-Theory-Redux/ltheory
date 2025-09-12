local Registry = require("Core.ECS.Registry")
local NameComponent = require("Modules.Core.Components.NameComponent")

---@alias EntityId integer

-- The entity class is defined in a separate file, to avoid a circular
-- dependency between Entity and Registry.
local Entity = require("Core.ECS.EntityClass")

function Entity:__tostring()
    return format("%s(%d)", self:get(NameComponent):getName(), self.id)
end

---@generic T
---@param component T
---@return T|nil
function Entity:add(component)
    return Registry:add(self, component)
end

---@generic T
---@param componentType T
---@return T|nil
function Entity:get(componentType)
    return Registry:get(self, componentType)
end

---@generic T
---@param componentType T
---@return T|nil
function Entity:remove(componentType)
    return Registry:remove(self, componentType)
end

-- This function constructs a new entity with the specified name and list of components.
---@param name string
---@param ... any
---@return Entity
function Entity.Create(name, ...)
    local entity = Registry:createEntity()
    entity:add(NameComponent(name or "Entity"))
    for _, component in ipairs({ ... }) do
        entity:add(component)
    end
    return entity
end

return Entity
