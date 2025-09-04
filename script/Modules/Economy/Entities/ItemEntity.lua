local Entity = require("Core.ECS.Entity")
local Economy = require("Modules.Economy.Components")
local Physics = require("Modules.Physics.Components")

---@class ItemEntity: Entity
---@param definition ItemDefinition
---@param quantity number
---@return ItemEntity
return function(definition, quantity)
    return Entity(
        definition.name,
        Physics.Mass(definition.mass),
        Economy.Quantity(quantity)
    )
end
