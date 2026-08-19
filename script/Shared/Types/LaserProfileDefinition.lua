local LaserProfileRegistry = require("Shared.Registries.LaserProfileRegistry")

---@class LaserProfileDefinitionConstructor
---@field id integer
---@field name string
---@field family integer
---@field wavelengthNm number
---@field photonEnergyEv number
---@field powerWatts number
---@field strength number
---@field baseDamagePerSecond number
---@field presentation table

---@class LaserProfileDefinition: LaserProfileDefinitionConstructor
---@overload fun(args: LaserProfileDefinitionConstructor): LaserProfileDefinition
local LaserProfileDefinition = Class("LaserProfileDefinition")

function LaserProfileDefinition.new(args)
    if not args.id or args.id <= 0 then
        Log.Warn("No valid ID set for LaserProfileDefinition")
        return nil
    elseif not args.name or #args.name == 0 then
        Log.Warn("No name set for LaserProfileDefinition: " .. tostring(args.id))
        return nil
    elseif not args.family or args.family <= 0 then
        Log.Warn("No valid family set for LaserProfileDefinition: " .. tostring(args.id))
        return nil
    elseif not args.wavelengthNm or args.wavelengthNm <= 0 then
        Log.Warn("No valid wavelength set for LaserProfileDefinition: " .. tostring(args.id))
        return nil
    elseif not args.strength or args.strength <= 0 then
        Log.Warn("No valid strength set for LaserProfileDefinition: " .. tostring(args.id))
        return nil
    elseif LaserProfileRegistry[args.id] then
        Log.Warn("Attempting to recreate LaserProfileDefinition: " .. tostring(args.id))
        return LaserProfileRegistry[args.id]
    end

    local definition = setmetatable({}, LaserProfileDefinition)
    definition.kind = Enums.Weapon.Effect.Beam
    for field, value in pairs(args) do
        definition[field] = value
    end
    definition.photonEnergyEv = definition.photonEnergyEv
        or (1239.841984 / definition.wavelengthNm)
    definition.damagePerSecond = definition.baseDamagePerSecond
        and definition.baseDamagePerSecond * definition.strength
        or nil

    LaserProfileRegistry:new(args.id, definition)
    return definition
end

return LaserProfileDefinition
