local Economy = require("Modules.Economy.Components")
local NameComponent = require("Modules.Core.Components.NameComponent")
local InventoryManager = require("Modules.Economy.Managers.InventoryManager")
local PeopleManager = require("Modules.People.Managers.PeopleManager")

---Gives entities their part in the economy: a station its production, market making, credits and
---stock; an asteroid a regrowing yield; a ship an agent's equipment and credits.
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

-- How many ore spots an asteroid's deposit shows.
local ORE_SPOTS = 5

---A unit direction, uniformly over the sphere.
---@param rng RNG
---@return Vec3f
local function randomDirection(rng)
    while true do
        local x, y, z = rng:getUniformRange(-1, 1), rng:getUniformRange(-1, 1), rng:getUniformRange(-1, 1)
        local length = math.sqrt(x * x + y * y + z * z)
        if length > 0.1 and length <= 1 then return Vec3f(x / length, y / length, z / length) end
    end
end

---@param asteroid Entity
---@param item ItemDefinition
---@param quantity integer What it holds, and the most it grows back to
---@param regrowPerSecond number|nil
---@param seed integer|nil Places the ore spots; the entity id when nil
function EconomySetup.equipAsteroid(asteroid, item, quantity, regrowPerSecond, seed)
    for _, componentType in ipairs({ Economy.Item, Economy.Quantity, Economy.Yield }) do
        if asteroid:get(componentType) then asteroid:remove(componentType) end
    end
    asteroid:add(Economy.Item(item.id))
    asteroid:add(Economy.Quantity(quantity))
    local rng = RNG.Create(seed or asteroid.id)
    local spots = {}
    for i = 1, ORE_SPOTS do spots[i] = randomDirection(rng) end
    asteroid:add(Economy.Yield(quantity, regrowPerSecond, spots))
end

---@class EconomyAgentSpec
---@field kind string|nil The ship's kind, for display (a key of `ShipKindDefs`)
---@field capacity number The hold's mass capacity
---@field miningLaser boolean|nil Whether the ship carries a mining laser
---@field credits number|nil Of the ship's own account, when it has no owner and no crew owns it
---@field owner Entity|nil An entity with a `PlayerBankAccount` that pays for the ship (a faction's
---leader, or an independent captain who owns it)
---@field name string|nil

---Makes a ship (one that takes navigation orders) an economic agent, with the equipment the jobs it
---can choose depend on: its hold and, maybe, a mining laser (its weapons, if any, are its own). The
---ship must be manned (`PeopleManager:man`): its commander gets the agent and decides for it.
---@param ship Entity
---@param spec EconomyAgentSpec
function EconomySetup.equipAgent(ship, spec)
    if spec.name then ship:get(NameComponent):setName(spec.name) end
    local inventory = ship:get(Economy.Inventory) or ship:add(Economy.Inventory())
    inventory:setCapacity(spec.capacity)
    if spec.miningLaser and not ship:get(Economy.MiningLaser) then ship:add(Economy.MiningLaser()) end
    if spec.owner then
        if not ship:get(Economy.Ownership) then ship:add(Economy.Ownership(spec.owner.id)) end
    else
        ship:add(Economy.PlayerBankAccount(spec.credits or 0))
    end
    local commander = PeopleManager:commanderOf(ship)
    assert(commander, "EconomySetup.equipAgent: the ship has no commander; man it first")
    commander:add(Economy.EconomyAgent(spec.kind))
end

return EconomySetup
