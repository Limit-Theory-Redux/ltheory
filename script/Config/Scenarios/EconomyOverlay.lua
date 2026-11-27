-- The HUD lines of a scenario with an economy: a line for each station the spawner made (credits,
-- stock) and each of its ships (what it does, its credits or earnings, cargo). A station's credits are
-- its account and, apart, what its bids on its own market hold.

local Registry          = require("Core.ECS.Registry")
local Items             = require("Shared.Registries.Items")
local Economy           = require("Modules.Economy.Components")
local InventoryManager  = require("Modules.Economy.Managers.InventoryManager")
local AccountManager    = require("Modules.Economy.Managers.AccountManager")
local SpawnerSystem     = require("Modules.Spawning.Systems.SpawnerSystem")
local MarketplaceSystem = require("Modules.Economy.Systems.MarketplaceSystem")

---"Item count" for each item the inventory holds, with "(+n on sale)" for what the holder's asks on
---its own market hold of it (goods out of the inventory until they sell or come back).
---@param inventory InventoryComponent
---@param holder Entity|nil A station, whose own market is looked at
---@return string
local function stockText(inventory, holder)
    local onSale = {}
    local marketplace = holder and holder:get(Economy.Marketplace)
    if marketplace then
        for _, order in ipairs(marketplace:getAsks()) do
            if Registry:hasEntity(order) and order:get(Economy.Ownership):getOwner() == holder.id then
                local itemId = order:get(Economy.Item):getItem()
                onSale[itemId] = (onSale[itemId] or 0) + order:get(Economy.Quantity):getQuantity()
            end
        end
    end
    local ids, seen = {}, {}
    for itemId in pairs(inventory:getInventory()) do ids[#ids + 1], seen[itemId] = itemId, true end
    for itemId in pairs(onSale) do if not seen[itemId] then ids[#ids + 1] = itemId end end
    table.sort(ids)
    local parts = {}
    for _, itemId in ipairs(ids) do
        local sale = onSale[itemId] and string.format(" (+%d on sale)", onSale[itemId]) or ""
        parts[#parts + 1] = string.format("%s %d%s", Items:getDefinition(itemId).name, InventoryManager:count(inventory, itemId), sale)
    end
    return #parts > 0 and table.concat(parts, ", ") or "empty"
end

---@return string[]
return function()
    local lines = {}
    for _, station in ipairs(SpawnerSystem:getStations()) do
        if Registry:hasEntity(station.entity) then
            local held = MarketplaceSystem:creditsHeldBy(station.entity, station.entity)
            lines[#lines + 1] = string.format("%s: %d cr%s | %s", station.name, AccountManager:balanceOf(station.entity),
                held > 0 and string.format(" (+%d in bids)", held) or "",
                stockText(station.entity:get(Economy.Inventory), station.entity))
        end
    end
    for _, ship in ipairs(SpawnerSystem:getAgents()) do
        local agent = Registry:hasEntity(ship.entity) and ship.entity:get(Economy.EconomyAgent)
        if agent then
            -- A ship its faction pays for shares its leader's account: what it earned tells it apart.
            local credits = ship.entity:get(Economy.Ownership) and string.format("earned %d", agent.earned)
                or string.format("%d cr", AccountManager:balanceOf(ship.entity))
            lines[#lines + 1] = string.format("%s: %s | %s | %s", ship.name, agent:getActivity(), credits,
                stockText(ship.entity:get(Economy.Inventory)))
        end
    end
    return lines
end
