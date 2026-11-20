local Component = require("Core.ECS.Component")

---@class NavOrderStatusComponent: Component
---@overload fun(self: NavOrderStatusComponent): NavOrderStatusComponent subclass internal
---@overload fun(): NavOrderStatusComponent subclass external
--- Where an order is in its life, and why it failed. The order system writes the status; an
--- issuer asks for a cancel with `requestCancel`, and the system ends the order at its next update.
local NavOrderStatusComponent = Subclass("NavOrderStatusComponent", Component, function(self)
    self:setComponentName("NavOrderStatus")
    self._status = Enums.NavOrderStatus.Queued
    self._failure = nil
    self._cancelRequested = false
end)

---@return NavOrderStatusId
function NavOrderStatusComponent:getStatus() return self._status end

---@param status NavOrderStatusId
function NavOrderStatusComponent:setStatus(status) self._status = status end

---@return NavOrderFailureId|nil Set only while the status is `Failed`
function NavOrderStatusComponent:getFailure() return self._failure end

---@param failure NavOrderFailureId
function NavOrderStatusComponent:setFailure(failure) self._failure = failure end

---@return boolean
function NavOrderStatusComponent:isCancelRequested() return self._cancelRequested end

function NavOrderStatusComponent:requestCancel() self._cancelRequested = true end

---@return boolean Whether the order has ended: done, failed or cancelled
function NavOrderStatusComponent:hasEnded()
    local status = self._status
    return status == Enums.NavOrderStatus.Done
        or status == Enums.NavOrderStatus.Failed
        or status == Enums.NavOrderStatus.Cancelled
end

return NavOrderStatusComponent
