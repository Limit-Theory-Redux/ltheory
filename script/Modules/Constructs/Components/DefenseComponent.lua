local Component = require("Core.ECS.Component")

---@class DefenseComponent: Component
---@field private _maxHealth number
---@field private _currentHealth number
---@field private _maxShield number
---@field private _currentShield number
---@field private _armor number
---@field private _shieldRegen number
---@overload fun(self: DefenseComponent, maxHealth: number|nil, maxShield: number|nil, armor: number|nil, shieldRegen: number|nil): DefenseComponent subclass internal
---@overload fun(maxHealth: number|nil, maxShield: number|nil, armor: number|nil, shieldRegen: number|nil): DefenseComponent subclass external
local DefenseComponent = Subclass("DefenseComponent", Component, function(
    self,
    maxHealth,
    maxShield,
    armor,
    shieldRegen
)
    self:setComponentName("DefenseComponent")

    self._maxHealth = maxHealth or 100
    self._currentHealth = self._maxHealth

    self._maxShield = maxShield or 0
    self._currentShield = self._maxShield

    self._armor = armor or 0
    self._shieldRegen = shieldRegen or 0
end)

-- GETTERS
function DefenseComponent:getMaxHealth() return self._maxHealth end
function DefenseComponent:getCurrentHealth() return self._currentHealth end
function DefenseComponent:getMaxShield() return self._maxShield end
function DefenseComponent:getCurrentShield() return self._currentShield end
function DefenseComponent:getArmor() return self._armor end
function DefenseComponent:getShieldRegen() return self._shieldRegen end

-- SETTERS
function DefenseComponent:setCurrentHealth(value)
    self._currentHealth = math.max(0, math.min(value, self._maxHealth))
end

function DefenseComponent:setCurrentShield(value)
    self._currentShield = math.max(0, math.min(value, self._maxShield))
end

function DefenseComponent:setArmor(value)
    self._armor = value
end

function DefenseComponent:setShieldRegen(value)
    self._shieldRegen = value
end

-- REGEN
---Advance shield recharge. Minimal per-tick state transition owned by the
---component; host states call this from their defense/regen update stage.
---@param dt number
function DefenseComponent:update(dt)
    if self._maxShield <= 0 or self._shieldRegen <= 0 then
        return
    end
    if self._currentShield < self._maxShield then
        self:setCurrentShield(self._currentShield + self._shieldRegen * dt)
    end
end


return DefenseComponent
