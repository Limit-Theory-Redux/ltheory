local Entity = require("_ECS_WIP_TEMP.Entities.Entity") --!temp path

-- Components
local RandomNumberGeneratorComponent = require("_ECS_WIP_TEMP.Components.Core.RandomNumberGenerator") --!temp path
local NameComponent = require("_ECS_WIP_TEMP.Components.Core.EntityName")                             --!temp path

---@class Planet: Entity
---@overload fun(self: table, seed: integer): Planet subclass interal
---@overload fun(seed: integer): Planet subclass external
local Planet = Subclass(Entity, function(self, seed)
    ---@cast self Planet

    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.PlanetEntity)

    -- Name Component
    self:addComponent(NameComponent())
end)

return Planet
