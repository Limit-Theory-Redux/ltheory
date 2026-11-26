local Component = require("Core.ECS.Component")

---A natural deposit that regrows (an asteroid's ore): the entity's `QuantityComponent` is what is
---left of it, which `ProductionSystem` grows back toward the maximum. Miners take from the quantity,
---with a laser on one of the deposit's spots: directions from the body's centre, in its own axes,
---where the ore shows on the surface.
---@class YieldComponent: Component
---@field spots Vec3f[] Unit directions in the body's axes
---@overload fun(self: YieldComponent, maxQuantity: integer, regrowPerSecond: number|nil, spots: Vec3f[]|nil): YieldComponent subclass internal
---@overload fun(maxQuantity: integer, regrowPerSecond: number|nil, spots: Vec3f[]|nil): YieldComponent subclass external
local YieldComponent = Subclass("YieldComponent", Component, function(self, maxQuantity, regrowPerSecond, spots)
    self:setComponentName("EconomyYield")

    self.maxQuantity = maxQuantity
    self.regrowPerSecond = regrowPerSecond or 0
    self.spots = spots or {}
    -- The part of a unit grown since the last whole one.
    self.growth = 0
end)

---@return Vec3f[]
function YieldComponent:getSpots()
    return self.spots
end

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
