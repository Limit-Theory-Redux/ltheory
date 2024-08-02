local Entity = require("Entity")

-- Components
local RandomNumberGeneratorComponent = require("Components.RandomNumberGenerator")
local NameComponent = require("Components.EntityName")
local TypeComponent = require("Components.EntityType")

---@class SpaceStationConstructor
---@field name string
---@field ownerId integer
---@field hullType SpaceStationHullType
---@field seed integer|nil

---@class SpaceStation: Entity
---@overload fun(self: table, name: string, hullType: SpaceShipHullType, seed: integer|nil): SpaceStation subclass interal
---@overload fun(name: string, hullType: SpaceShipHullType, seed: integer|nil): SpaceStation subclass external
local SpaceStation = Subclass(Entity, function(self, name, hullType, seed)
    ---@cast self SpaceStation

    -- Name Component
    self:addComponent(NameComponent(name))
    -- Type Component
    self:addComponent(TypeComponent(Enums.EntityType.SpaceStation))
end)

return SpaceStation
