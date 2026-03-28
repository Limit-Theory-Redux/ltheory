local Component = require("Core.ECS.Component")

---@class EnginesComponent: Component
---@field private _engines table
---@field private _exhaustScale number
---@field private _thrustPower number
---@field private _afterburnerPower number
---@field private _afterburnerFuel number
---@field private _currentFuel number
---@field private _fuelRegenRate number
---@overload fun(self: EnginesComponent, exhaustScale: number|nil, thrustPower: number|nil, afterburnerPower: number|nil, afterburnerFuel: number|nil, fuelRegenRate: number|nil): EnginesComponent subclass internal
---@overload fun(exhaustScale: number|nil, thrustPower: number|nil, afterburnerPower: number|nil, afterburnerFuel: number|nil, fuelRegenRate: number|nil): EnginesComponent subclass external
local EnginesComponent = Subclass("EnginesComponent", Component, function(
    self,
    exhaustScale,
    thrustPower,
    afterburnerPower,
    afterburnerFuel,
    fuelRegenRate
)
    self:setComponentName("EnginesComponent")

    self._engines = {} -- data only

    self._exhaustScale = exhaustScale or 1
    self._thrustPower = thrustPower or 1
    self._afterburnerPower = afterburnerPower or 2
    self._afterburnerFuel = afterburnerFuel or 100
    self._currentFuel = self._afterburnerFuel
    self._fuelRegenRate = fuelRegenRate or 10
end)

-- ENGINE MANAGEMENT
function EnginesComponent:addEngine(position, power)
    table.insert(self._engines, {
        position = position,
        power = power or 1
    })
end

function EnginesComponent:getEngines()
    return self._engines
end

-- GETTERS
function EnginesComponent:getExhaustScale() return self._exhaustScale end
function EnginesComponent:getThrustPower() return self._thrustPower end
function EnginesComponent:getAfterburnerPower() return self._afterburnerPower end
function EnginesComponent:getAfterburnerFuel() return self._afterburnerFuel end
function EnginesComponent:getCurrentFuel() return self._currentFuel end
function EnginesComponent:getFuelRegenRate() return self._fuelRegenRate end

-- SETTERS
function EnginesComponent:setExhaustScale(v) self._exhaustScale = v end
function EnginesComponent:setThrustPower(v) self._thrustPower = v end
function EnginesComponent:setAfterburnerPower(v) self._afterburnerPower = v end
function EnginesComponent:setAfterburnerFuel(v) self._afterburnerFuel = v end
function EnginesComponent:setCurrentFuel(v)
    self._currentFuel = math.max(0, math.min(v, self._afterburnerFuel))
end
function EnginesComponent:setFuelRegenRate(v) self._fuelRegenRate = v end

return EnginesComponent
