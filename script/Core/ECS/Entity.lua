local Registry = require("Core.ECS.Registry")

---@class Entity
---@field components table<any, ComponentInfo>
---@field name string

-- General Purpose Entity Object. Contains a reference to its components, but does not own the component data.
---@param self Entity
---@param name string The name of the entity
local Entity = Class("Entity", function(self, name, ...)
    self.name = name or "Entity"
    self.guid = Guid.Create()
    self:addComponents(...)
    self:enable()
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
    return self.name
end

---@param name string
function Entity:setName(name)
    self.name = name
end

--- Enables the entity
function Entity:enable()
    self.enabled = true
end

--- Disables the entity
function Entity:disable()
    self.enabled = false
end

---@return boolean
function Entity:isEnabled()
    return self.enabled
end

---@return EntityId
function Entity:getEntityId()
    return self.guid
end

---@param ... Component A variable list of components to add
function Entity:addComponents(...)
    if self.components then
        Log.Warn("This entity already has components, are you sure that you want to reinitialize?")
    end
    self.components = {}

    for _, component in ipairs({ ... }) do
        self:addComponent(component)
    end
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

return Entity
