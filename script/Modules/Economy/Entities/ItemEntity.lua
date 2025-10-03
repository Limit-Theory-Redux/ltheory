local Entity = require("Core.ECS.Entity")
local Economy = require("Modules.Economy.Components")
local Physics = require("Modules.Physics.Components")

---@param definition ItemDefinition
---@param quantity number
---@return Entity
return function(definition, quantity)
    return Entity.Create(
        definition.name,
        Physics.Mass(definition.mass),
        Economy.ItemType(definition.id),
        Economy.Quantity(quantity)
    )
end
