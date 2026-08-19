local BeamRegistry = require("Shared.Registries.BeamRegistry")

---@class BeamDefinitionConstructor
---@field id integer
---@field name string
---@field tickInterval number
---@field visual table

---@class BeamDefinition: BeamDefinitionConstructor
---@overload fun(args: BeamDefinitionConstructor): BeamDefinition
local BeamDefinition = Class("BeamDefinition")

function BeamDefinition.new(args)
    if not args.id or args.id <= 0 then
        Log.Warn("No valid ID set for BeamDefinition")
        return nil
    elseif not args.name or #args.name == 0 then
        Log.Warn("No name set for BeamDefinition: " .. tostring(args.id))
        return nil
    elseif BeamRegistry[args.id] then
        Log.Warn("Attempting to recreate BeamDefinition: " .. tostring(args.id))
        return BeamRegistry[args.id]
    end

    local definition = setmetatable({}, BeamDefinition)
    definition.kind = Enums.Weapon.Effect.Beam
    for field, value in pairs(args) do
        definition[field] = value
    end

    BeamRegistry:new(args.id, definition)
    return definition
end

return BeamDefinition
