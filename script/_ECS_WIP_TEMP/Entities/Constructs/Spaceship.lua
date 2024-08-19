local Entity = require("_ECS_WIP_TEMP.Entities.Entity") --!temp path

-- Components
local RandomNumberGeneratorComponent = require("_ECS_WIP_TEMP.Components.Core.RandomNumberGenerator") --!temp path
local NameComponent = require("_ECS_WIP_TEMP.Components.Core.EntityName")                             --!temp path
local TypeComponent = require("_ECS_WIP_TEMP.Components.Core.EntityType")                             --!temp path

---@class SpaceshipConstructor
---@field name string
---@field ownerId integer
---@field hullType SpaceshipHullType
---@field seed integer|nil

---@class Spaceship: Entity
---@overload fun(self: table, name: string, ownerId: integer, hullType: SpaceshipHullType, seed: integer|nil): Spaceship subclass interal
---@overload fun(name: string, ownerId: integer, hullType: SpaceshipHullType, seed: integer|nil): Spaceship subclass external
local Spaceship = Subclass(Entity, function(self, name, ownerId, hullType, seed)
    ---@cast self Spaceship

    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.SpaceshipEntity)

    -- RandomNumberGeneratorComponent
    local _, rngComponent = self:addComponent(RandomNumberGeneratorComponent(seed, true))
    ---@cast rngComponent RandomNumberGeneratorComponent
    -- Name Component
    self:addComponent(NameComponent(name))
    -- Type Component
    self:addComponent(TypeComponent())
end)

return Spaceship
