local Component = require('Components.Component')

---@class QuantityComponent: Component
---@overload fun(self: QuantityComponent, quantity: number|nil): QuantityComponent subclass internal
---@overload fun(quantity: number|nil): QuantityComponent subclass external
local QuantityComponent = Subclass("QuantityComponent", Component, function(self, quantity)
    self:setComponentName("EconomyQuantity")

    -- Set Component Archetype
    self:setArchetype(Enums.ComponentArchetype.QuantityComponent)

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
    for owner, lockedQuantity in pairs(self.lockedQuantity) do
        availableQuantity = availableQuantity - lockedQuantity
    end
    return availableQuantity
end

---@param owner Player
---@param amount integer
function QuantityComponent:setLockedQuantity(owner, amount)
    self.lockedQuantity = self.lockedQuantity or {}
    self.lockedQuantity[owner] = (self.lockedQuantity[owner] or 0) + amount
end

---@param owner Player|nil
function QuantityComponent:getLockedQuantity(owner)
    return self.lockedQuantity or self.lockedQuantity and self.lockedQuantity[owner]
end

---@param owner Player
---@param amount integer|nil
function QuantityComponent:unlockQuantity(owner, amount)
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
