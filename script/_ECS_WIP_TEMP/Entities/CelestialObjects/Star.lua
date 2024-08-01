local Entity = require("_ECS_WIP_TEMP.Entity")

-- Components
local RandomNumberGeneratorComponent = require("_ECS_WIP_TEMP.Components.RandomNumberGenerator")
local NameComponent = require("_ECS_WIP_TEMP.Components.EntityName")
local TypeComponent = require("_ECS_WIP_TEMP.Components.EntityType")

---@class Star: Entity
---@overload fun(self: table, seed: integer): Star subclass interal
---@overload fun(seed: integer): Star subclass external
local Star = Subclass(Entity, function(self, seed)
    ---@cast self Star

    -- RandomNumberGeneratorComponent
    local rngComponentIndex = self:addComponent(RandomNumberGeneratorComponent(seed, true))
    local rngComponent = self:getComponent(rngComponentIndex)
    ---@cast rngComponent RandomNumberGeneratorComponent

    -- Name Component
    self:addComponent(NameComponent())
    -- Type Component
    self:addComponent(TypeComponent(Enums.EntityType.Star))
end)

return Star
