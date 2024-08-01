require("Iterator")

-- General Purpose Entity Object
---@class Entity
local Entity = Class(function(self)
    self:addGuid()
    self:addComponents()
end)

function Entity:addGuid()
    self.guid = Guid.Create()
end

function Entity:getGuid()
    return self.guid
end

function Entity:addComponents()
    if self.components then
        Log.Warn("This entity already has components, are you sure that you want to reinitialize?")
    end
    self.components = {}
end

---@return integer componentIndex
function Entity:addComponent(component)
    insert(self.components, component)
    return #self.components
end

---@param componentIndex integer
function Entity:removeComponent(componentIndex)
    remove(self.components, componentIndex)
end

---@return Component
function Entity:getComponent(componentIndex)
    return self.components[componentIndex]
end

---@param query string
---@return Component
function Entity:findComponent(query)
    local queryResults = {}
    for index, component in ipairs(self.components) do
        if string.find(component:getComponentName(), query) then
            insert(queryResults, index)
        end
    end

    if queryResults > 1 then
        Log.Error("Found more than one component for your query. Please be more specific.")
    end
    return queryResults[1]
end

---@param entityType EntityType
---@return Component
function Entity:findChildByType(entityType)
    local queryResults = {}
    for index, component in ipairs(self.components) do
        if component:getType() == entityType then
            insert(queryResults, index)
        end
    end

    if queryResults > 1 then
        Log.Error("Found more than one component for your query. Please be more specific.")
    end
    return queryResults[1]
end

---@return IteratorIndexed
function Entity:iterComponents()
    return IteratorIndexed(self.components)
end

function Entity:addEvents()
    self.eventHandlers = {}
end

---@return table<EventHandler>
function Entity:getEvents()
    return self.eventHandlers
end

---@return IteratorIndexed<integer, EventHandler>
function Entity:iterEvents()
    return IteratorIndexed(self.eventHandlers)
end

function Entity:registerEvent(eventName, ctxTable, callback)
    local handler = EventBus:subscribe(eventName, self, function(...) callback(ctxTable, ...) end)
    insert(self.eventHandlers, handler)
end

-- To be called from entity init
function Entity:registerEventHandlers()
    -- Set up event handlers
    ---@param component Component
    for _, component in self:iterComponents() do
        for _, eventInfo in component:iterEvents() do
            self:registerEvent(eventInfo.name, component, eventInfo.callback)
        end
    end
end

function Entity:unregisterEventHandlers()
    -- Set Up Event Handlers
    for _, handler in self:iterEvents() do
        handler:unsubscribe()
    end
end

function Entity:destroy() --todo: introduce proper clean up mechanism
    self:unregisterEventHandlers()
    --todo
end

return Entity
