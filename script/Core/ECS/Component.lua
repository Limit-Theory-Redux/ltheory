-- General Purpose Component Object
---@class Component
local Component = Class("Component", function(self)
    ---@cast self Component
    self:addGuid()
    self.componentName = nil
    self:addEvents()
end)

--- Naming Convention: ComponentCategory .. ComponentName; e.g. PhysicsRigidBody
---@param name string
function Component:setComponentName(name)
    if not name or type(name) ~= "string" then
        Log.Warn("Did not provide a valid string name for component name")
        return
    end

    self.componentName = name .. "Component"

    local mt = getmetatable(self)
    if mt then
        mt.__tostring = function(self)
            return format("%s(%s)", self.componentName or "Unnamed", tostring(self:getGuid()))
        end
        setmetatable(self, mt)
    end
end

---@return string|nil
function Component:getComponentName()
    if not self.componentName then
        Log.Warn("Component name is not set")
        return nil
    end

    return self.componentName
end

function Component:addGuid()
    self.guid = Guid.Create()
end

---@return integer
function Component:getGuid()
    return self.guid
end

---@generic T: Component
---@return T
function Component:getArchetype()
    return type(self)
end

---@param entityId integer
function Component:setEntityId(entityId)
    self.entityId = entityId
end

---@return EntityId
function Component:getEntityId()
    return self.entityId
end

function Component:addEvents()
    self.events = {}
end

function Component:removeEvent(eventName)
    self.events[eventName] = nil
end

---@return table
function Component:getEvents()
    return self.events
end

---@return IteratorIndexed
function Component:iterEvents()
    return IteratorIndexed(self.events)
end

---@param eventName string
function Component:registerEventHandler(eventName, callbackFn)
    insert(self.events, { name = eventName, callback = callbackFn })
end

return Component
