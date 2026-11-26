-- The HUD lines of a scenario with an economy: a line for each station the spawner made (credits,
-- stock) and each of its agents (what it does, credits, cargo).

local Registry          = require("Core.ECS.Registry")
local Items             = require("Shared.Registries.Items")
local Economy           = require("Modules.Economy.Components")
local InventoryManager  = require("Modules.Economy.Managers.InventoryManager")
local AccountManager    = require("Modules.Economy.Managers.AccountManager")
local SpawnerSystem     = require("Modules.Spawning.Systems.SpawnerSystem")

---"Item count" for each item the inventory holds.
---@param inventory InventoryComponent
---@return string
local function stockText(inventory)
    local ids = {}
    for itemId in pairs(inventory:getInventory()) do ids[#ids + 1] = itemId end
    table.sort(ids)
    local parts = {}
    for _, itemId in ipairs(ids) do
        parts[#parts + 1] = string.format("%s %d", Items:getDefinition(itemId).name, InventoryManager:count(inventory, itemId))
    end
    return #parts > 0 and table.concat(parts, ", ") or "empty"
end

---@return string[]
return function()
    local lines = {}
    for _, station in ipairs(SpawnerSystem:getStations()) do
        if Registry:hasEntity(station.entity) then
            lines[#lines + 1] = string.format("%s: %d cr | %s", station.name, AccountManager:balanceOf(station.entity),
                stockText(station.entity:get(Economy.Inventory)))
        end
    end
    for _, ship in ipairs(SpawnerSystem:getAgents()) do
        local agent = Registry:hasEntity(ship.entity) and ship.entity:get(Economy.EconomyAgent)
        if agent then
            lines[#lines + 1] = string.format("%s: %s | %d cr | %s", ship.name, agent:getActivity(),
                AccountManager:balanceOf(ship.entity), stockText(ship.entity:get(Economy.Inventory)))
        end
    end
    return lines
end
