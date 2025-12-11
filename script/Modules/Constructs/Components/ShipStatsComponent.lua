local Component = require("Core.ECS.Component")

---@class ShipStatsComponent: Component
---@overload fun(self: ShipStatsComponent, maxHealth: number, maxShield: number): ShipStatsComponent subclass internal
---@overload fun(maxHealth: number, maxShield: number): ShipStatsComponent subclass external
local ShipStatsComponent = Subclass("ShipStatsComponent", Component, function(self, maxHealth, maxShield)
    self:setComponentName("ShipStatsComponent")

    self.maxHealth = maxHealth or 100
    self.currentHealth = self.maxHealth
    self.maxShield = maxShield or 50
    self.currentShield = self.maxShield
    self.speed = 10
    self.acceleration = 5
    self.turnRate = 1
    self.mass = 1000
end)

function ShipStatsComponent:takeDamage(amount)
    if self.currentShield > 0 then
        local shieldDamage = math.min(amount, self.currentShield)
        self.currentShield = self.currentShield - shieldDamage
        amount = amount - shieldDamage
    end

    if amount > 0 then
        self.currentHealth = math.max(0, self.currentHealth - amount)
    end
end

function ShipStatsComponent:heal(amount)
    self.currentHealth = math.min(self.maxHealth, self.currentHealth + amount)
end

function ShipStatsComponent:rechargeShield(amount)
    self.currentShield = math.min(self.maxShield, self.currentShield + amount)
end

function ShipStatsComponent:isDestroyed()
    return self.currentHealth <= 0
end

return ShipStatsComponent
