local Component = require("Core.ECS.Component")

---@class HealthComponent: Component
---@field _maxHealth number
---@field _currentHealth number
---@overload fun(self: HealthComponent, maxHealth: number|nil): HealthComponent subclass internal
---@overload fun(maxHealth: number|nil): HealthComponent subclass external
local HealthComponent = Subclass("HealthComponent", Component, function(self, maxHealth)
    self:setComponentName("HealthComponent")

    self._maxHealth = maxHealth or 100
    self._currentHealth = self._maxHealth
end)

-- GETTERS
function HealthComponent:getMaxHealth() return self._maxHealth end
function HealthComponent:getCurrentHealth() return self._currentHealth end

-- SETTERS
function HealthComponent:setCurrentHealth(value)
    self._currentHealth = math.max(0, math.min(value, self._maxHealth))
end

function HealthComponent:setMaxHealth(value)
    self._maxHealth = math.max(1, value)
    -- Clamp currentHealth to new maxHealth
    self._currentHealth = math.min(self._currentHealth, self._maxHealth)
end

-- Convenience method
function HealthComponent:isDestroyed()
    return self._currentHealth <= 0
end

return HealthComponent
