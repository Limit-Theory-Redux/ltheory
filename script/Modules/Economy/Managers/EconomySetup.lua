local Economy = require("Modules.Economy.Components")
local NameComponent = require("Modules.Core.Components.NameComponent")
local InventoryManager = require("Modules.Economy.Managers.InventoryManager")

---Gives entities their part in the economy: a station its production, market making, credits and
---stock; an asteroid a regrowing yield.
---@class EconomySetup
local EconomySetup = {}

---@param station Entity A station entity (it has an inventory and a market)
---@param definition StationEconomyDefinition From `Shared/Definitions/StationEconomyDefs.lua`
---@param name string|nil The station's name, kept in its `NameComponent`
function EconomySetup.equipStation(station, definition, name)
    if name then station:get(NameComponent):setName(name) end
    station:add(Economy.PlayerBankAccount(definition.credits))
    station:add(Economy.Production(definition.recipes))
    station:add(Economy.MarketMaker())
    local inventory = station:get(Economy.Inventory)
    for _, stack in ipairs(definition.stock) do
        InventoryManager:add(inventory, stack.item.id, stack.count)
    end
end

---@param asteroid Entity
---@param item ItemDefinition
---@param quantity integer What it holds, and the most it grows back to
---@param regrowPerSecond number|nil
function EconomySetup.equipAsteroid(asteroid, item, quantity, regrowPerSecond)
    for _, componentType in ipairs({ Economy.Item, Economy.Quantity, Economy.Yield }) do
        if asteroid:get(componentType) then asteroid:remove(componentType) end
    end
    asteroid:add(Economy.Item(item.id))
    asteroid:add(Economy.Quantity(quantity))
    asteroid:add(Economy.Yield(quantity, regrowPerSecond))
end

return EconomySetup
