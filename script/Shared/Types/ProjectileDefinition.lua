local ProjectileRegistry = require("Shared.Registries.ProjectileRegistry")

---@class ProjectileDefinitionConstructor
---@field id integer
---@field name string
---@field speed number
---@field lifetime number
---@field scale number
---@field visual table
---@field guidance table|nil
---@field archetype string|nil
---@field shaderKey string|nil

---@class ProjectileDefinition: ProjectileDefinitionConstructor
---@overload fun(args: ProjectileDefinitionConstructor): ProjectileDefinition
local ProjectileDefinition = Class("ProjectileDefinition")

function ProjectileDefinition.new(args)
    if not args.id or args.id <= 0 then
        Log.Warn("No valid ID set for ProjectileDefinition")
        return nil
    elseif not args.name or #args.name == 0 then
        Log.Warn("No name set for ProjectileDefinition: " .. tostring(args.id))
        return nil
    elseif ProjectileRegistry[args.id] then
        Log.Warn("Attempting to recreate ProjectileDefinition: " .. tostring(args.id))
        return ProjectileRegistry[args.id]
    end

    local definition = setmetatable({}, ProjectileDefinition)
    definition.kind = Enums.Weapon.Effect.Projectile
    for field, value in pairs(args) do
        definition[field] = value
    end

    ProjectileRegistry:new(args.id, definition)
    return definition
end

return ProjectileDefinition
