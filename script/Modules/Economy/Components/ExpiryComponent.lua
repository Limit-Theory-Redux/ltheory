local Component = require("Core.ECS.Component")

---When something stops being valid, in seconds of the economy's simulated clock
---(`MarketplaceSystem:now()`), so that a paused or sped-up game keeps its orders in step.
---@class ExpiryComponent: Component
---@overload fun(self: ExpiryComponent, expireAt: number|nil): ExpiryComponent subclass internal
---@overload fun(expireAt: number|nil): ExpiryComponent subclass external
local ExpiryComponent = Subclass("ExpiryComponent", Component, function(self, expireAt)
    self:setComponentName("EconomyExpiry")

    self:setExpiry(expireAt)
end)

---@param expireAt number|nil Simulated seconds; nil never expires
function ExpiryComponent:setExpiry(expireAt)
    self.expireAt = expireAt
end

---@return number|nil expireAt
function ExpiryComponent:getExpiry()
    return self.expireAt
end

---@param now number Simulated seconds
---@return number secondsUntilExpiry `math.huge` when it never expires
function ExpiryComponent:getSecondsUntilExpire(now)
    if not self.expireAt then return math.huge end
    return self.expireAt - now
end

---@param now number Simulated seconds
---@return boolean
function ExpiryComponent:isExpired(now)
    return self.expireAt ~= nil and now >= self.expireAt
end

return ExpiryComponent
