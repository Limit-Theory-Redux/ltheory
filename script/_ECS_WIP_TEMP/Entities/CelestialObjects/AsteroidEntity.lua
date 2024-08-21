local Entity = require("_ECS_WIP_TEMP.Entities.Entity") --!temp path

-- Components
local NameComponent = require("_ECS_WIP_TEMP.Components.Core.EntityName")          --!temp path
local SeedComponent = require("_ECS_WIP_TEMP.Components.Generation.SeedComponent") --!temp path
local TransformComponent = require("_ECS_WIP_TEMP.Components.Physics.Transform")   --!temp path

---@class Asteroid: Entity
---@overload fun(self: Asteroid, seed: integer): Asteroid subclass interal
---@overload fun(seed: integer): Asteroid subclass external
local Asteroid = Subclass(Entity, function(self, seed)
    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.AsteroidEntity)

    -- Name Component
    self:addComponent(NameComponent())

    -- Seed Component
    self:addComponent(SeedComponent(seed))

    -- Transform Component
    self:addComponent(TransformComponent())
end)

return Asteroid
