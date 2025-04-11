local Entity = require("Entities.Entity")
local Components = require("Components")

---@param definition ItemDefinition
---@param quantity number
---@return Entity
local function ItemEntity(definition, quantity)
    return Entity(
        "ItemEntity",
        Components.NameComponent(definition.name),
        Components.MassComponent(definition.mass),
        Components.QuantityComponent(quantity)
    )
end

return ItemEntity
