local Entity = require("Core.ECS.Entity")

---@class ItemEntity: Entity
---@param definition ItemDefinition
---@param quantity number
return function(definition, quantity)
    local Economy = require("Modules.Economy")
    local Physics = require("Modules.Physics")

    return Entity(
        definition.name,
        Physics.Components.Mass(definition.mass),
        Economy.Components.Quantity(quantity)
    )
end
