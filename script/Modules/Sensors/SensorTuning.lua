-- The dials a tuning panel can change, with their ranges and steps, and the
-- scales its bars use for the model's numbers.
--
--   local spec = SensorTuning.dials[2]                      -- resolution
--   dials[spec.key] = SensorTuning.step(spec, dials[spec.key], 1)
--   local x = SensorTuning.position(spec, dials[spec.key])   -- 0 to 1 along its slider

local SensorText = require("Modules.Sensors.SensorText")

local SensorTuning = {}

local LOG_STEPS = 10 -- steps per factor of ten on a log dial

---@class SensorDialSpec
---@field key string the dial's key in `SensorDials`
---@field name string
---@field min number|nil
---@field max number|nil
---@field log boolean|nil steps multiply instead of add
---@field step number|nil the step of a dial that adds
---@field toggle boolean|nil on or off
---@field format fun(value: any): string

--- The dials, in the panel's order.
---@type SensorDialSpec[]
SensorTuning.dials = {
    { key = "sensitivity", name = "SENS", min = 0.1, max = 10, log = true,
        format = function(v) return string.format("×%.3g", v) end },
    { key = "resolution", name = "RES", min = 0.005, max = 2, log = true,
        format = function(v) return string.format("%.3g°", v) end },
    { key = "heatBand", name = "HEAT", toggle = true,
        format = function(v) return v and "on" or "off" end },
    { key = "power", name = "POWER", min = 50, max = 2e4, log = true, format = SensorText.power },
    { key = "scanTime", name = "SCAN", min = 0.5, max = 10, step = 0.5,
        format = function(v) return string.format("%.1f s", v) end },
    { key = "ownMw", name = "OWN", min = 1, max = 2000, log = true, format = SensorText.power },
}

local function clamp(x, lo, hi) return math.min(hi, math.max(lo, x)) end

--- A dial's value one step up (`direction` 1) or down (-1), within its range;
--- a toggle flips.
---@param spec SensorDialSpec
---@param value any
---@param direction integer
---@return any
function SensorTuning.step(spec, value, direction)
    if spec.toggle then return not value end
    if spec.log then
        return clamp(value * 10 ^ (direction / LOG_STEPS), spec.min, spec.max)
    end
    return clamp(value + direction * spec.step, spec.min, spec.max)
end

--- Where a value sits along its dial's slider, 0 to 1 (a toggle: 0 or 1).
---@param spec SensorDialSpec
---@param value any
---@return number
function SensorTuning.position(spec, value)
    if spec.toggle then return value and 1 or 0 end
    if spec.log then
        return clamp(math.log(value / spec.min) / math.log(spec.max / spec.min), 0, 1)
    end
    return clamp((value - spec.min) / (spec.max - spec.min), 0, 1)
end

--- Where a positive number sits on a log bar from 10^lo to 10^hi, 0 to 1.
---@param x number
---@param lo number
---@param hi number
---@return number
function SensorTuning.logBar(x, lo, hi)
    if not (x > 0) then return 0 end
    return clamp((math.log10(x) - lo) / (hi - lo), 0, 1)
end

return SensorTuning
