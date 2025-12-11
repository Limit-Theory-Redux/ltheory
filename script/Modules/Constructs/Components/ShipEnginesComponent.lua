local Component = require("Core.ECS.Component")

---@class ShipEnginesComponent: Component
---@overload fun(self: ShipEnginesComponent): ShipEnginesComponent subclass internal
---@overload fun(): ShipEnginesComponent subclass external
local ShipEnginesComponent = Subclass("ShipEnginesComponent", Component, function(self)
    self:setComponentName("ShipEnginesComponent")

    self.engines = {}          -- List of engine positions/data
    self.exhaustScale = 1      -- Visual exhaust scale
    self.thrustPower = 1       -- Thrust multiplier
    self.afterburnerPower = 2  -- Afterburner multiplier
    self.afterburnerFuel = 100 -- Afterburner fuel capacity
    self.currentFuel = 100     -- Current afterburner fuel
    self.fuelRegenRate = 10    -- Fuel regeneration per second
end)

function ShipEnginesComponent:addEngine(position, power)
    table.insert(self.engines, {
        position = position,
        power = power or 1
    })
end

function ShipEnginesComponent:consumeFuel(amount)
    self.currentFuel = math.max(0, self.currentFuel - amount)
end

function ShipEnginesComponent:regenerateFuel(dt)
    if self.currentFuel < self.afterburnerFuel then
        self.currentFuel = math.min(
            self.afterburnerFuel,
            self.currentFuel + self.fuelRegenRate * dt
        )
    end
end

function ShipEnginesComponent:canAfterburn()
    return self.currentFuel > 0
end

return ShipEnginesComponent
