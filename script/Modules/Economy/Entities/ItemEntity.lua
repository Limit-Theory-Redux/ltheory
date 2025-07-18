local Entity = require("Core.ECS.Entity")
local Components = loadComponents("Economy", "Physics")

---@param definition ItemDefinition
---@param quantity number
---@return Entity
local function ItemEntity(definition, quantity)
    return Entity(
        definition.name,
        Components.MassComponent(definition.mass),
        Components.QuantityComponent(quantity)
    )
end

return ItemEntity
