local Registry = require("Core.ECS.Registry")
local QuickProfiler = require("Shared.Tools.QuickProfiler")
local Economy = require("Modules.Economy.Components")
local InventoryManager = require("Modules.Economy.Managers.InventoryManager")
local AccountManager = require("Modules.Economy.Managers.AccountManager")
local MarketplaceSystem = require("Modules.Economy.Systems.MarketplaceSystem")

---Runs production: each line of a `ProductionComponent` starts a cycle when the entity's inventory
---holds the recipe's inputs, takes them, and after the recipe's duration puts the outputs into the
---inventory and pays the recipe's income. A line with outputs idles while the entity already holds
---`stockCapCycles` cycles' worth of every one of them (its asks on its own market included). Also regrows natural yields (`YieldComponent`). Runs on
---simulated time, at PreSim.
---@class ProductionSystem
local ProductionSystem = Class("ProductionSystem", function(self)
    self:registerVars()
    self:registerEvents()
end)

function ProductionSystem:registerVars()
    self.profiler = QuickProfiler("ProductionSystem", false, false)
end

function ProductionSystem:registerEvents()
    EventBus:subscribe(Event.PreSim, self, self.onPreSim)
end

---@param e EventData
function ProductionSystem:onPreSim(e)
    self:update(e:deltaTime())
end

---@param inventory InventoryComponent
---@param recipe RecipeDefinition
---@return boolean
local function hasInputs(inventory, recipe)
    for _, stack in ipairs(recipe.inputs) do
        if InventoryManager:count(inventory, stack.item.id) < stack.count then return false end
    end
    return true
end

---@param inventory InventoryComponent
---@param recipe RecipeDefinition
---@return boolean
local function hasRoomForOutputs(inventory, recipe)
    for _, stack in ipairs(recipe.outputs) do
        if InventoryManager:getRoomFor(inventory, stack.item.id) < stack.count then return false end
    end
    return true
end

---@param dt number Seconds
function ProductionSystem:update(dt)
    if dt <= 0 then return end
    self.profiler:start()
    for entity, production, inventory in Registry:iterEntities(Economy.Production, Economy.Inventory) do
        local stocked = nil
        for _, line in ipairs(production:getLines()) do
            if not line.running and #line.recipe.outputs > 0 then
                if stocked == nil then stocked = self:isStocked(entity, production, inventory) end
                if stocked then goto next_line end
            end
            self:updateLine(entity, inventory, line, dt)
            ::next_line::
        end
    end
    for _, yield, quantity in Registry:iterEntities(Economy.Yield, Economy.Quantity) do
        self:regrow(yield, quantity, dt)
    end
    self.profiler:stop()
end

---Whether the entity holds `stockCapCycles` cycles' worth of every output of its lines, in its
---inventory and its asks on its own market.
---@param entity Entity
---@param production ProductionComponent
---@param inventory InventoryComponent
---@return boolean
function ProductionSystem:isStocked(entity, production, inventory)
    local _, outputs = production:getFlows()
    local any = false
    for itemId, perCycle in pairs(outputs) do
        any = true
        local held = InventoryManager:count(inventory, itemId)
        if entity:get(Economy.Marketplace) then held = held + MarketplaceSystem:goodsHeldBy(entity, entity, itemId) end
        if held < perCycle * production:getStockCapCycles() then return false end
    end
    return any
end

---@param entity Entity
---@param inventory InventoryComponent
---@param line ProductionLine
---@param dt number
function ProductionSystem:updateLine(entity, inventory, line, dt)
    local recipe = line.recipe
    if not line.running then
        if not hasInputs(inventory, recipe) or not hasRoomForOutputs(inventory, recipe) then return end
        for _, stack in ipairs(recipe.inputs) do
            InventoryManager:remove(inventory, stack.item.id, stack.count)
        end
        line.running, line.progress = true, 0
    end

    line.progress = line.progress + dt
    if line.progress < recipe.duration then return end
    -- An output that no longer fits waits: the cycle ends when there is room.
    if not hasRoomForOutputs(inventory, recipe) then return end

    for _, stack in ipairs(recipe.outputs) do
        InventoryManager:add(inventory, stack.item.id, stack.count)
    end
    if recipe.income then
        AccountManager:deposit(AccountManager:accountOf(entity), recipe.income)
    end
    line.running, line.progress = false, 0
    line.cycles = line.cycles + 1
end

---@param yield YieldComponent
---@param quantity QuantityComponent
---@param dt number
function ProductionSystem:regrow(yield, quantity, dt)
    local current, max = quantity:getQuantity(), yield:getMaxQuantity()
    if current >= max or yield:getRegrowRate() <= 0 then
        yield:setGrowth(0)
        return
    end
    local growth = yield:getGrowth() + yield:getRegrowRate() * dt
    local whole = math.floor(growth)
    quantity:setQuantity(math.min(max, current + whole))
    yield:setGrowth(growth - whole)
end

return ProductionSystem()
