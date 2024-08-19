local Entity = require("_ECS_WIP_TEMP.Entities.Entity") --!temp path

-- Components
local RandomNumberGeneratorComponent = require("_ECS_WIP_TEMP.Components.Core.RandomNumberGenerator") --!temp path
local NameComponent = require("_ECS_WIP_TEMP.Components.Core.EntityName")                             --!temp path

---@class Asteroid: Entity
---@overload fun(self: table, seed: integer): Asteroid subclass interal
---@overload fun(seed: integer): Asteroid subclass external
local Asteroid = Subclass(Entity, function(self, seed)
    ---@cast self Asteroid

    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.AsteroidEntity)

    -- Name Component
    self:addComponent(NameComponent())
end)

return Asteroid
