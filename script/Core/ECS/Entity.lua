local Registry = require("Core.ECS.Registry")
local EntityComponent = require('Core.ECS.EntityComponent')
local ChildrenComponent = require("Core.ECS.ChildrenComponent")
local ParentComponent = require("Core.ECS.ParentComponent")
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

function Entity:clone()
    local clone = Entity(self.name)

    for component in self:iterComponents() do
        ---@type Component
        local clonedComponent = DeepClone(component)
        clonedComponent:addGuid()
        clone:addComponent(clonedComponent)
    end

    return clone, clone:getEntityId()
end

function Entity:attach(childEntityId)
    local child = Registry:getEntity(childEntityId)
    if not child then
        Log.Warn("Entity:attach - child entity not found: %s", tostring(childEntityId))
        return false
    end

    -- Ensure that the parent process is set correctly.
    local parentComponent = child:getComponent(ParentComponent)
    if parentComponent then
        -- If the child already has a parent, detach it from the existing parent.
        local existingParent = Registry:getEntity(parentComponent:getParent())
        if existingParent then
            existingParent:detach(childEntityId)
        else
            Log.Warn("Entity:attach - existing parent entity not found: %s, skipping", tostring(parentComponent:getParent()))
        end
        parentComponent:setParent(self:getEntityId())
    else
        child:addComponent(ParentComponent(self:getEntityId()))
    end
    
    -- Update children.
    if not self:getComponent(ChildrenComponent) then
        self:addComponent(ChildrenComponent())
    end
    local childrenComponent = self:getComponent(ChildrenComponent)
    childrenComponent:addChild(childEntityId)
    return true
end

function Entity:detach(childEntityId)
    local child = Registry:getEntity(childEntityId)
    if not child then
        Log.Warn("Entity:detach - child entity not found: %s", tostring(childEntityId))
        return false
    end

    -- Remove child from parent's ChildrenComponent
    local childrenComponent = self:getComponent(ChildrenComponent)
    if childrenComponent then
        childrenComponent:removeChild(childEntityId)
    end

    -- Remove ParentComponent from child.
    child:removeComponent(ParentComponent)
    return true
end

return Entity
