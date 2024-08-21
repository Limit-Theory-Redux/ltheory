local Entity = require("_ECS_WIP_TEMP.Entities.Entity") --!temp path

-- Components
local NameComponent = require("_ECS_WIP_TEMP.Components.Core.EntityName")          --!temp path
local SeedComponent = require("_ECS_WIP_TEMP.Components.Generation.SeedComponent") --!temp path
local TransformComponent = require("_ECS_WIP_TEMP.Components.Physics.Transform")   --!temp path

---@class PlanetEntity: Entity
---@overload fun(self: PlanetEntity, seed: integer): PlanetEntity subclass interal
---@overload fun(seed: integer): PlanetEntity subclass external
local PlanetEntity = Subclass(Entity, function(self, seed)
    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.PlanetEntity)

    -- Name Component
    self:addComponent(NameComponent())

    -- Seed Component
    self:addComponent(SeedComponent(seed))

    -- Transform Component
    self:addComponent(TransformComponent())
end)

return PlanetEntity
