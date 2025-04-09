local Component = require('Components.Component')

---@class ExpiryComponent: Component
---@overload fun(self: ExpiryComponent, expireAt: TimeStamp|nil): ExpiryComponent subclass internal
---@overload fun(expireAt: TimeStamp|nil): ExpiryComponent subclass external
local ExpiryComponent = Subclass("ExpiryComponent", Component, function(self, expireAt)
    self:setComponentName("EconomyExpiry")

    -- Set Component Archetype
    self:setArchetype(Enums.ComponentArchetype.ExpiryComponent)

    if expireAt then
        self:setExpiry(expireAt)
    end
end)

---@param expireAt TimeStamp
function ExpiryComponent:setExpiry(expireAt)
    self.expireAt = expireAt
end

---@return TimeStamp expireAt
function ExpiryComponent:getExpiry()
    return self.expireAt
end

---@return number secondsTillExpiry
function ExpiryComponent:getSecondsUntilExpire()
    return TimeStamp.Now():getDifference(self.expireAt)
end

return ExpiryComponent
