local Registry = require("Systems.Storage.Registry")

---@class Entity
---@field components table<any, ComponentInfo>

-- General Purpose Entity Object. Contains a reference to its components, but does not own the component data.
---@param self Entity
---@class Entity
local Entity = Class("Entity", function(self)
    self:addGuid()
    self:addComponents()
    self:Enable()
end)

function Entity:addGuid()
    self.guid = Guid.Create()
end

---@return integer
function Entity:getGuid()
    return self.guid
end

---Enables Entity
function Entity:Enable()
    self.enabled = true
end

---Disables Entity
function Entity:Disable()
    self.enabled = false
end

---@return boolean # If Entity is Enabled
function Entity:isEnabled()
    return self.enabled
end

---@param archetype EntityArchetype
function Entity:setArchetype(archetype)
    self.archetype = archetype

    local mt = getmetatable(self)
    if mt then
        mt.__tostring = function(self)
            return format("%s(%s)", Enums.EntityArchetype:getName(self.archetype), tostring(self:getGuid()))
        end
        setmetatable(self, mt)
    end
end

---@return EntityArchetype
function Entity:getArchetype()
    return self.archetype
end

---@return EntityId
function Entity:getEntityId()
    return self.guid
end

function Entity:addComponents()
    if self.components then
        Log.Warn("This entity already has components, are you sure that you want to reinitialize?")
    end
    self.components = {}
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

---@generic T
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
    local clone = Entity()

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
