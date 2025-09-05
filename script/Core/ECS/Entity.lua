local Registry = require("Core.ECS.Registry")
local EntityComponent = require('Core.ECS.EntityComponent')
local NameComponent = require("Core.ECS.NameComponent")

---@class Entity

-- General Purpose Entity.
---@param self Entity
---@param name string The name of the entity
local Entity = Class("Entity", function(self, name, ...)
    self.guid = Registry:createEntity()
    self:addComponent(EntityComponent(self)) -- Store a component that refers back to this object.
    self:addComponent(NameComponent(name or "Entity"))
    for _, component in ipairs({ ... }) do
        self:addComponent(component)
    end
end)

function Entity:__tostring()
    return format("%s(%d)", self:getName(), self:getEntityId())
end

---@return integer
function Entity:getGuid()
    return self.guid
end

---@return string
function Entity:getName()
    return self:getComponent(NameComponent):getName()
end

---@param name string
function Entity:setName(name)
    self:getComponent(NameComponent):setName(name)
end

---@return EntityId
function Entity:getEntityId()
    return self.guid
end

---@param component Component
function Entity:addComponent(component)
    Registry:add(self.guid, component)
end

---@param componentType any
---@return boolean wasSuccessful
function Entity:removeComponent(componentType)
    return Registry:remove(self.guid, componentType)
end

---@generic T
---@param componentType T
---@return T|nil
function Entity:getComponent(componentType)
    return Registry:get(self.guid, componentType)
end

function Entity:iterComponents()
    return Registry:iterComponents(self.guid)
end

---@return boolean success
function Entity:destroy()
    return Registry:destroyEntity(self.guid)
end

return Entity
