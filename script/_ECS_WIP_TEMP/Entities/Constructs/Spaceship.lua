local Entity = require("Entity")

-- Components
local RandomNumberGeneratorComponent = require("Components.RandomNumberGenerator")
local NameComponent = require("Components.EntityName")
local TypeComponent = require("Components.EntityType")

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

    -- Name Component
    self:addComponent(NameComponent(name))
    -- Type Component
    self:addComponent(TypeComponent(Enums.EntityType.Spaceship))
end)

return Spaceship
