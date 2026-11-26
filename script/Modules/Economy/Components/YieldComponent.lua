local Component = require("Core.ECS.Component")

---A natural deposit that regrows (an asteroid's ore): the entity's `QuantityComponent` is what is
---left of it, which `ProductionSystem` grows back toward the maximum. Miners take from the quantity.
---@class YieldComponent: Component
---@overload fun(self: YieldComponent, maxQuantity: integer, regrowPerSecond: number|nil): YieldComponent subclass internal
---@overload fun(maxQuantity: integer, regrowPerSecond: number|nil): YieldComponent subclass external
local YieldComponent = Subclass("YieldComponent", Component, function(self, maxQuantity, regrowPerSecond)
    self:setComponentName("EconomyYield")

    self.maxQuantity = maxQuantity
    self.regrowPerSecond = regrowPerSecond or 0
    -- The part of a unit grown since the last whole one.
    self.growth = 0
end)

---@return integer
function YieldComponent:getMaxQuantity()
    return self.maxQuantity
end

---@return number
function YieldComponent:getRegrowRate()
    return self.regrowPerSecond
end

---@return number
function YieldComponent:getGrowth()
    return self.growth
end

---@param growth number
function YieldComponent:setGrowth(growth)
    self.growth = growth
end

return YieldComponent
