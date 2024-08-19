local Entity = require("_ECS_WIP_TEMP.Entities.Entity") --!temp path

-- Components
local RandomNumberGeneratorComponent = require("_ECS_WIP_TEMP.Components.Core.RandomNumberGenerator") --!temp path
local NameComponent = require("_ECS_WIP_TEMP.Components.Core.EntityName")                             --!temp path

-- Utils
local Words = require("_ECS_WIP_TEMP.Systems.Generators.Words") --!temp path

---@class SpaceStationConstructor
---@field name string
---@field ownerId integer
---@field hullType SpaceStationHullType
---@field seed integer|nil

---@class SpaceStation: Entity
---@overload fun(self: SpaceStation, name: string, hullType: SpaceShipHullType, seed: integer|nil): SpaceStation subclass interal
---@overload fun(name: string, hullType: SpaceShipHullType, seed: integer|nil): SpaceStation subclass external
local SpaceStation = Subclass(Entity, function(self, name, hullType, seed)
    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.SpaceStationEntity)

    -- RandomNumberGeneratorComponent
    local _, rngComponent = self:addComponent(RandomNumberGeneratorComponent(seed, true))
    ---@cast rngComponent RandomNumberGeneratorComponent
    -- Name Component
    local genName = Words.getCoolName(rngComponent:getRNG())
    self:addComponent(NameComponent(genName))
end)

return SpaceStation
