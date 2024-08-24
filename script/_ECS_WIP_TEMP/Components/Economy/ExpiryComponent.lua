local Component = require('Component')

---@class ExpiryComponent: Component
---@overload fun(self: ExpiryComponent, playerId: integer|nil): ExpiryComponent subclass internal
---@overload fun(playerId: integer|nil): ExpiryComponent subclass external
local ExpiryComponent = Subclass(Component, function(self)
    self:setComponentName("EconomyExpiry")
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
