local Entity = require("_ECS_WIP_TEMP.Entities.Entity") --!temp path

-- Components
local RandomNumberGeneratorComponent = require("_ECS_WIP_TEMP.Components.Core.RandomNumberGenerator") --!temp path
local NameComponent = require("_ECS_WIP_TEMP.Components.Core.EntityName")                             --!temp path
local TypeComponent = require("_ECS_WIP_TEMP.Components.Core.EntityType")                             --!temp path

---@class Star: Entity
---@overload fun(self: table, seed: integer): Star subclass interal
---@overload fun(seed: integer): Star subclass external
local Star = Subclass(Entity, function(self, seed)
    ---@cast self Star

    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.StarEntity)

    -- Name Component
    self:addComponent(NameComponent())
    -- Type Component
    self:addComponent(TypeComponent())
end)

return Star
