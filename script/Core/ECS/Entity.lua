local Registry = require("Core.ECS.Registry")
local ChildrenComponent = require("Core.ECS.ChildrenComponent")
local ParentComponent = require("Core.ECS.ParentComponent")
local NameComponent = require("Core.ECS.NameComponent")

---@class Entity

-- General Purpose Entity Object. Contains a reference to its components, but does not own the component data.
---@param self Entity
---@param name string The name of the entity
local Entity = Class("Entity", function(self, name, ...)
    self.guid = Guid.Create()
    self.components = {}
    self:addComponent(NameComponent(name or "Entity"))
    for _, component in ipairs({ ... }) do
        self:addComponent(component)
    end
end)

function Entity:__tostring()
    return format("%s(%d)", self.name, self:getEntityId())
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
---@return ComponentInfo componentInfo
function Entity:addComponent(component)
    component:setEntityId(self:getEntityId())
    local componentInfo = Registry:storeComponent(component)
    self.components[component:getArchetype()] = componentInfo
    return componentInfo
end

---@param componentType any
---@return boolean wasSuccessful
function Entity:removeComponent(componentType)
    if self.components[componentType] == nil then
        return false
    end
    Registry:dropComponent(self.components[componentType])
    self.components[componentType] = nil
    return true
end

---@generic T: Component
---@param archetype T
---@return T|nil
function Entity:getComponent(archetype)
    local componentInfo = self.components[archetype]
    if not componentInfo then
        return nil
    end

    return Registry:getComponent(componentInfo)
end

function Entity:iterComponents()
    local components = {}
    for _, info in pairs(self.components) do
        insert(components, Registry:getComponent(info))
    end
    return Iterator(components)
end

function Entity:clearComponents()
    for type, info in pairs(self.components) do
        Registry:dropComponent(info)
    end
    self.components = {}
end

---@return boolean success
function Entity:destroy()
    local success = Registry:dropEntity(self.guid)
    if success then
        local noFail = true
        for _, info in pairs(self.components) do
            local success = Registry:dropComponent(info)

            if not success then
                noFail = false
            end
        end

        if noFail then
            self:clearComponents()
            self = nil
            return true
        end
    end
    -- revert
    Registry:storeEntity(self)

    for component in self:iterComponents() do
        Registry:storeComponent(component)
    end
    return false
end

function Entity:clone()
    local clone = Entity(self.name)

    for component in self:iterComponents() do
        ---@type Component
        local clonedComponent = DeepClone(component)
        clonedComponent:addGuid()
        clone:addComponent(clonedComponent)
    end

    local cloneEntityId = Registry:storeEntity(clone)

    return clone, cloneEntityId
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
