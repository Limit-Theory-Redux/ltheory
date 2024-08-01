-- General Purpose Component Object
---@class Component
local Component = Class(function(self)
    ---@cast self Component
    self.componentName = nil
    self:addEvents()
end)

---@param name string
function Component:setComponentName(name)
    if not name or type(name) ~= "string" then
        Log.Warn("Did not provide a valid string name for component name")
        return
    end

    self.componentName = name .. "Component"
end

---@return string|nil
function Component:getComponentName()
    if not self.componentName then
        Log.Warn("Component name is not set")
        return nil
    end

    return self.componentName
end

function Component:addEvents()
    self.events = {}
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
