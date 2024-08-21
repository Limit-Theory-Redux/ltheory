local Entity = require("_ECS_WIP_TEMP.Entities.Entity") --!temp path

-- Components
local NameComponent = require("_ECS_WIP_TEMP.Components.Core.EntityName")          --!temp path
local SeedComponent = require("_ECS_WIP_TEMP.Components.Generation.SeedComponent") --!temp path
local TransformComponent = require("_ECS_WIP_TEMP.Components.Physics.Transform")   --!temp path

---@class StarEntity: Entity
---@overload fun(self: StarEntity, seed: integer): StarEntity subclass interal
---@overload fun(seed: integer): StarEntity subclass external
local StarEntity = Subclass(Entity, function(self, seed)
    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.StarEntity)

    -- Name Component
    self:addComponent(NameComponent())

    -- Seed Component
    self:addComponent(SeedComponent(seed))

    -- Transform Component
    self:addComponent(TransformComponent())
end)

return StarEntity
