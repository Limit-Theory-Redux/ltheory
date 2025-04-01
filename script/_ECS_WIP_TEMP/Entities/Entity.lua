local Registry = require("_ECS_WIP_TEMP.Systems.Storage.Registry") --!temp path

---@type EntityInfo
local EntityInfo = require("_ECS_WIP_TEMP.Shared.Types.EntityInfo")
---@type ComponentInfo
local ComponentInfo = require("_ECS_WIP_TEMP.Shared.Types.ComponentInfo")

---@class Entity
---@field components table<ComponentInfo>

-- General Purpose Entity Object
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

---@return EntityInfo
function Entity:getEntityInfo()
    return EntityInfo { archetype = self.archetype, id = self.guid }
end

function Entity:addComponents()
    if self.components then
        Log.Warn("This entity already has components, are you sure that you want to reinitialize?")
    end
    self.components = {}
end

---@return integer componentInfoIndex
---@return Component
function Entity:addComponent(component)
    component:setEntity(self:getEntityInfo())
    insert(self.components, ComponentInfo { id = component:getGuid(), archetype = component:getArchetype(), entity = self:getEntityInfo() })
    Registry:storeComponent(component)
    return #self.components, component
end

---@param componentInfoIndex integer
---@return boolean wasSuccessful
function Entity:removeComponent(componentInfoIndex)
    local componentInfo = remove(self.components, componentInfoIndex)
    return Registry:dropComponent(componentInfo.archetype, componentInfo.id)
end

---@param archetype ComponentArchetype
---@return integer resultCount
---@return table<Component>
function Entity:findComponentsByArchetype(archetype)
    local queryResults = {}
    ---@param componentInfo ComponentInfo
    for index, componentInfo in ipairs(self.components) do
        if componentInfo.archetype == archetype then
            local component = Registry:getComponentData(componentInfo)
            insert(queryResults, component)
        end
    end
    return #queryResults, queryResults
end

---@param archetype ComponentArchetype
---@return table<Component>
function Entity:findComponentByArchetype(archetype)
    local queryResults = {}
    ---@param componentInfo ComponentInfo
    for index, componentInfo in ipairs(self.components) do
        if componentInfo.archetype == archetype then
            local component = Registry:getComponentData(componentInfo)
            insert(queryResults, component)
        end
    end
    if #queryResults > 1 then
        Log.Error("Found more than one component for your query. Please be more specific.")
    end
    return queryResults[1]
end

---@param query string
---@return Component|nil
function Entity:findComponentByName(query)
    local queryResults = {}
    for index, componentInfo in ipairs(self.components) do
        local component = Registry:getComponentData(componentInfo)
        local componentName = component and component:getComponentName()
        if componentName and string.match(componentName, query) then
            insert(queryResults, component)
        end
    end

    if #queryResults > 1 then
        Log.Error("Found more than one component for your query. Please be more specific.")
    end

    return queryResults[1]
end

---@return ComponentInfo
function Entity:getComponentInfo(componentInfoIndex)
    return self.components[componentInfoIndex]
end

function Entity:iterComponents()
    local components = {}
    for index, componentInfo in ipairs(self.components) do
        local component = Registry:getComponentData(componentInfo)
        insert(components, component)
    end
    return Iterator(components)
end

-- does not handle clearing from Registry
function Entity:clearComponents()
    self.components = {}
end

---@return boolean success
function Entity:destroy()
    local success = Registry:dropEntity(self.archetype, self.guid)

    if success then
        local noFail = true
        for component in self:iterComponents() do
            local success = Registry:dropComponent(component.archetype, component.guid)

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
    local clone = ShallowClone(self)
    clone:addGuid()
    clone:clearComponents()

    for component in self:iterComponents() do
        local clonedComponent = DeepClone(component)
        clonedComponent:addGuid()
        clone:addComponent(clonedComponent)
    end

    local cloneEntityInfo = Registry:storeEntity(clone)

    return clone, cloneEntityInfo
end

return Entity
