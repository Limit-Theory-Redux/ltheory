local GlobalStorage = require("_ECS_WIP_TEMP.Systems.GlobalStorage") --!temp path

---@class Entity
---@field components table<ComponentInfo>

-- General Purpose Entity Object
---@class Entity
local Entity = Class(function(self)
    ---@cast self Entity
    self:addGuid()
    self:addComponents()
end)

function Entity:addGuid()
    self.guid = Guid.Create()
end

---@return integer
function Entity:getGuid()
    return self.guid
end

---@param archetype EntityArchetype
function Entity:setArchetype(archetype)
    self.archetype = archetype
end

---@return EntityArchetype
function Entity:getArchetype()
    return self.archetype
end

---@return EntityInfo
function Entity:getEntityInfo()
    return { archetype = self.archetype, id = self.guid }
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
    insert(self.components, { id = component:getGuid(), archetype = component:getArchetype() })
    GlobalStorage.storeComponent(component)
    return #self.components, component
end

---@param componentInfoIndex integer
---@return boolean wasSuccessful
function Entity:removeComponent(componentInfoIndex)
    local componentInfo = remove(self.components, componentInfoIndex)
    return GlobalStorage.dropComponent(componentInfo.archetype, componentInfo.id)
end

---@param archetype ComponentArchetype
---@return integer resultCount
---@return table<Component>
function Entity:findComponentsByArchetype(archetype)
    local queryResults = {}
    ---@param componentInfo ComponentInfo
    for index, componentInfo in ipairs(self.components) do
        if componentInfo.archetype == archetype then
            local component = GlobalStorage.getComponentData(componentInfo)
            insert(queryResults, component)
        end
    end
    return #queryResults, queryResults
end

---@param query string
---@return Component|nil
function Entity:findComponentByName(query)
    local queryResults = {}
    for index, componentInfo in ipairs(self.components) do
        local component = GlobalStorage.getComponentData(componentInfo)
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
        local component = GlobalStorage.getComponentData(componentInfo)
        insert(components, component)
    end
    return Iterator(components)
end

function Entity:destroy() --todo: introduce proper clean up mechanism
    --todo
end

return Entity
