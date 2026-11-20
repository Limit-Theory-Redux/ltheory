---Constructor and validation for flight profiles.
---A profile is a ship's flight limits: mass, drag and the largest force and torque
---per axis, in engine units. Registers with FlightProfileRegistry;
---Shared/Definitions/FlightProfileDefs.lua is the data source.

local FlightProfileRegistry = require("Shared.Registries.FlightProfileRegistry")

---@class FlightProfileConstructor
---@field id FlightProfileId
---@field name string
---@field mass number kg
---@field linearDrag number 1/s
---@field angularDrag number 1/s
---@field thrustForward number Largest force along each ship axis, N in gameunits
---@field thrustRight number
---@field thrustUp number
---@field torquePitch number Largest torque about each ship axis
---@field torqueYaw number
---@field torqueRoll number
---@field maxSpeed number gameunits/s, the speed forward thrust settles at

---@class FlightProfile: FlightProfileConstructor
---@overload fun(args: FlightProfileConstructor): FlightProfile
local FlightProfile = Class("FlightProfile")

local POSITIVE_FIELDS = {
    "mass", "linearDrag", "angularDrag",
    "thrustForward", "thrustRight", "thrustUp",
    "torquePitch", "torqueYaw", "torqueRoll",
    "maxSpeed",
}

function FlightProfile.new(args)
    if not args.id or args.id <= 0 then
        Log.Warn("No valid ID set for FlightProfile")
        return nil
    elseif not args.name or #args.name == 0 then
        Log.Warn("No name set for FlightProfile: " .. tostring(args.id))
        return nil
    elseif FlightProfileRegistry:has(args.id) then
        Log.Warn("Attempting to recreate FlightProfile: " .. tostring(args.id))
        return FlightProfileRegistry:get(args.id)
    end

    for _, field in ipairs(POSITIVE_FIELDS) do
        local value = args[field]
        assert(type(value) == "number" and value > 0,
            string.format("FlightProfile %s: %s must be a positive number", args.name, field))
    end

    local definition = setmetatable({}, FlightProfile)
    for field, value in pairs(args) do
        definition[field] = value
    end

    FlightProfileRegistry:new(args.id, definition)
    return definition
end

return FlightProfile
