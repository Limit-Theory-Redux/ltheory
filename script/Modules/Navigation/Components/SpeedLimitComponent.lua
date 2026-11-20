local Component = require("Core.ECS.Component")

---@alias GameUnitsPerSecond number A speed in game units per second

---@class SpeedLimitComponent: Component
---@overload fun(self: SpeedLimitComponent, limit: GameUnitsPerSecond): SpeedLimitComponent subclass internal
---@overload fun(limit: GameUnitsPerSecond): SpeedLimitComponent subclass external
--- The highest speed allowed inside a region (a zone entity). The limit is authored in m/s and
--- converted with `UniverseScaleConfig:toGameUnits` before it is stored.
local SpeedLimitComponent = Subclass("SpeedLimitComponent", Component, function(self, limit)
    self:setComponentName("SpeedLimit")
    self:setLimit(limit)
end)

---@param limit GameUnitsPerSecond Greater than zero
function SpeedLimitComponent:setLimit(limit)
    assert(type(limit) == "number" and limit > 0, "SpeedLimit: the limit must be a number greater than zero")
    self._limit = limit
end

---@return GameUnitsPerSecond
function SpeedLimitComponent:getLimit()
    return self._limit
end

return SpeedLimitComponent
