local Registry = require("Core.ECS.Registry")
local QuickProfiler = require("Shared.Tools.QuickProfiler")
local Economy = require("Modules.Economy.Components")
local InventoryManager = require("Modules.Economy.Managers.InventoryManager")
local AccountManager = require("Modules.Economy.Managers.AccountManager")
local MarketplaceSystem = require("Modules.Economy.Systems.MarketplaceSystem")

---Keeps a producing station's orders on its own market (`MarketMakerComponent`): every interval it
---takes back its orders and posts a bid for each input of its recipes, up to its stock target, and
---an ask for all it holds of each output. A price is the market's shifting equilibrium for the item
---(which follows the trades made there), raised for an input the station is short of and an output
---it has little of, and lowered for a glut. The stock targets are the station's consumption or
---making of the item over `stockCycles` cycles; an input's target also covers its consumption over
---`resupplyS` seconds, so a slow supply (ore that miners take minutes to bring) is bid for early.
---@class MarketMakerSystem
local MarketMakerSystem = Class("MarketMakerSystem", function(self)
    self:registerVars()
    self:registerEvents()
end)

function MarketMakerSystem:registerVars()
    self.profiler = QuickProfiler("MarketMakerSystem", false, false)
    self.rng = RNG.FromTime()
    -- A bid's price over the reference: at an empty stock, and at the target.
    self.bidEmptyFactor = 1.5
    self.bidFullFactor = 0.8
    -- An ask's price over the reference: at an empty stock, and at twice the target or more.
    self.askEmptyFactor = 1.3
    self.askGlutFactor = 0.75
    -- The reference stays within these multiples of the item's start price.
    self.minReferenceFactor = 0.25
    self.maxReferenceFactor = 4
    -- The share of its balance a station commits to its bids.
    self.bidBudgetShare = 0.8
end

function MarketMakerSystem:registerEvents()
    EventBus:subscribe(Event.PreSim, self, self.onPreSim)
end

---@param e EventData
function MarketMakerSystem:onPreSim(e)
    self:update()
end

function MarketMakerSystem:update()
    local now = MarketplaceSystem:now()
    self.profiler:start()
    for entity, maker in Registry:iterEntities(Economy.MarketMaker) do
        ---@cast maker MarketMakerComponent
        local due = maker:getNextRepostAt()
        if not due then
            -- Spread the stations' reposts over the interval from the start.
            maker:setNextRepostAt(now + self.rng:getUniformRange(0, maker:getInterval()))
        elseif now >= due then
            self:repost(entity, maker)
            maker:setNextRepostAt(now + maker:getInterval() * self.rng:getUniformRange(0.8, 1.2))
        end
    end
    self.profiler:stop()
end

---The price an item is reckoned at on a market.
---@param marketplace MarketplaceComponent
---@param itemId integer
---@return number
function MarketMakerSystem:referencePrice(marketplace, itemId)
    local start = MarketplaceSystem:getStartPrice(itemId)
    local reference = marketplace:getShiftingEquilibrium(itemId) or start
    return math.max(start * self.minReferenceFactor, math.min(start * self.maxReferenceFactor, reference))
end

---@param reference number
---@param fill number Stock over target
---@return integer
function MarketMakerSystem:bidPrice(reference, fill)
    local t = math.min(1, math.max(0, fill))
    return math.max(1, math.floor(reference * (self.bidEmptyFactor + (self.bidFullFactor - self.bidEmptyFactor) * t)))
end

---@param reference number
---@param fill number Stock over target
---@return integer
function MarketMakerSystem:askPrice(reference, fill)
    local t = math.min(1, math.max(0, fill / 2))
    return math.max(1, math.ceil(reference * (self.askEmptyFactor + (self.askGlutFactor - self.askEmptyFactor) * t)))
end

---Takes back the station's orders and posts its bids and asks again.
---@param entity Entity
---@param maker MarketMakerComponent
function MarketMakerSystem:repost(entity, maker)
    local marketplace = entity:get(Economy.Marketplace)
    local production = entity:get(Economy.Production)
    local inventory = entity:get(Economy.Inventory)
    if not marketplace or not production or not inventory then return end

    -- Taking the orders back first returns their escrow, so the stock and balance below are whole.
    MarketplaceSystem:cancelOrdersOf(entity, entity)
    local inputs, outputs = production:getFlows()
    local inputRates = production:getRates()
    local ttl = maker:getInterval() * 2

    local wanted, cost = {}, 0
    for itemId, perCycle in pairs(inputs) do
        local target = math.max(perCycle * maker:getStockCycles(), math.ceil(inputRates[itemId] * maker:getResupplyS()))
        local stock = InventoryManager:count(inventory, itemId)
        if stock < target then
            local price = self:bidPrice(self:referencePrice(marketplace, itemId), stock / target)
            wanted[#wanted + 1] = { itemId = itemId, quantity = target - stock, price = price }
            cost = cost + (target - stock) * price
        end
    end
    -- A station that can't pay for every bid scales them all down alike.
    local budget = AccountManager:balanceOf(entity) * self.bidBudgetShare
    local scale = cost > budget and budget / cost or 1
    for _, bid in ipairs(wanted) do
        local quantity = math.floor(bid.quantity * scale)
        if quantity > 0 then
            MarketplaceSystem:postBid(entity, entity, bid.itemId, quantity, bid.price, ttl)
        end
    end

    for itemId, perCycle in pairs(outputs) do
        -- An item the station also consumes is kept for its own lines.
        if not inputs[itemId] then
            local stock = InventoryManager:count(inventory, itemId)
            if stock > 0 then
                local target = perCycle * maker:getStockCycles()
                local price = self:askPrice(self:referencePrice(marketplace, itemId), stock / target)
                MarketplaceSystem:postAsk(entity, entity, itemId, stock, price, ttl)
            end
        end
    end
end

return MarketMakerSystem()
