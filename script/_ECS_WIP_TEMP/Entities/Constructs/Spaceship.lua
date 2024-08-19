local Entity = require("_ECS_WIP_TEMP.Entities.Entity") --!temp path

-- Components
local RandomNumberGeneratorComponent = require("_ECS_WIP_TEMP.Components.Core.RandomNumberGenerator") --!temp path
local NameComponent = require("_ECS_WIP_TEMP.Components.Core.EntityName")                             --!temp path

---@class SpaceshipConstructor
---@field name string
---@field ownerId integer
---@field hullType SpaceshipHullType
---@field seed integer|nil

---@class Spaceship: Entity
---@overload fun(self: Spaceship, name: string, ownerId: integer, hullType: SpaceshipHullType, seed: integer|nil): Spaceship subclass interal
---@overload fun(name: string, ownerId: integer, hullType: SpaceshipHullType, seed: integer|nil): Spaceship subclass external
local Spaceship = Subclass(Entity, function(self, name, ownerId, hullType, seed)

    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.SpaceshipEntity)

    -- RandomNumberGeneratorComponent
    local _, rngComponent = self:addComponent(RandomNumberGeneratorComponent(seed, true))
    ---@cast rngComponent RandomNumberGeneratorComponent
    -- Name Component
    self:addComponent(NameComponent(name))
end)

return Spaceship
