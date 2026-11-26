local Component = require("Core.ECS.Component")

---@class QuantityComponent: Component
---@overload fun(self: QuantityComponent, quantity: number|nil): QuantityComponent subclass internal
---@overload fun(quantity: number|nil): QuantityComponent subclass external
local QuantityComponent = Subclass("QuantityComponent", Component, function(self, quantity)
    self:setComponentName("EconomyQuantity")

    self:init(quantity)
end)

function QuantityComponent:init(quantity)
    self.quantity = quantity or 0
end

---@param quantity integer
function QuantityComponent:setQuantity(quantity)
    self.quantity = quantity
end

---@return integer quantity
function QuantityComponent:getQuantity()
    return self.quantity
end

---@return integer availableQuantity
function QuantityComponent:getAvailableQuantity()
    local availableQuantity = self.quantity
    for _, lockedQuantity in pairs(self.lockedQuantity or {}) do
        availableQuantity = availableQuantity - lockedQuantity
    end
    return availableQuantity
end

---@param owner Entity
---@param amount integer
function QuantityComponent:setLockedQuantity(owner, amount)
    self.lockedQuantity = self.lockedQuantity or {}
    self.lockedQuantity[owner] = (self.lockedQuantity[owner] or 0) + amount
end

---@param owner Entity|nil
---@return integer locked The quantity `owner` has locked, or every owner's together when nil
function QuantityComponent:getLockedQuantity(owner)
    if not self.lockedQuantity then return 0 end
    if owner then return self.lockedQuantity[owner] or 0 end
    local total = 0
    for _, amount in pairs(self.lockedQuantity) do total = total + amount end
    return total
end

---@param owner Entity
---@param amount integer|nil
function QuantityComponent:unlockQuantity(owner, amount)
    if not self.lockedQuantity or not self.lockedQuantity[owner] then return end
    if not amount then
        self.lockedQuantity[owner] = nil
        return
    end

    self.lockedQuantity[owner] = self.lockedQuantity[owner] - amount

    if self.lockedQuantity[owner] <= 0 then
        self.lockedQuantity[owner] = nil
    end
end

return QuantityComponent
