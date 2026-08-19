local WeaponRegistry = require("Shared.Registries.WeaponRegistry")

---@class WeaponDefinitionConstructor
---@field id integer
---@field name string
---@field category integer
---@field range number
---@field damage number
---@field damagePerSecond number|nil
---@field cooldown number
---@field interShotGap number
---@field capacitorCost number
---@field turretScale number
---@field laserProfileId integer|nil
---@field laserProfile LaserProfileDefinition|nil
---@field accuracy table
---@field tracking table
---@field firePolicy table
---@field ai table
---@field effect ProjectileDefinition|BeamDefinition

---@class WeaponDefinition: WeaponDefinitionConstructor
---@overload fun(args: WeaponDefinitionConstructor): WeaponDefinition
local WeaponDefinition = Class("WeaponDefinition")

function WeaponDefinition.new(args)
    if not args.id or args.id <= 0 then
        Log.Warn("No valid ID set for WeaponDefinition")
        return nil
    elseif not args.name or #args.name == 0 then
        Log.Warn("No name set for WeaponDefinition: " .. tostring(args.id))
        return nil
    elseif WeaponRegistry[args.id] then
        Log.Warn("Attempting to recreate WeaponDefinition: " .. tostring(args.id))
        return WeaponRegistry[args.id]
    end

    local definition = setmetatable({}, WeaponDefinition)
    for field, value in pairs(args) do
        definition[field] = value
    end

    WeaponRegistry:new(args.id, definition)
    return definition
end

return WeaponDefinition
