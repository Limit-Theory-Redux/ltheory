local Component = require("Core.ECS.Component")

---Makes an entity (a station with production) keep orders on its own market: bids for the inputs
---of its recipes, asks for their outputs. `MarketMakerSystem` posts them again every `interval`
---seconds, priced by how the stock of each item compares to its target.
---@class MarketMakerComponent: Component
---@overload fun(self: MarketMakerComponent, stockCycles: number|nil, interval: number|nil): MarketMakerComponent subclass internal
---@overload fun(stockCycles: number|nil, interval: number|nil): MarketMakerComponent subclass external
local MarketMakerComponent = Subclass("MarketMakerComponent", Component, function(self, stockCycles, interval)
    self:setComponentName("EconomyMarketMaker")

    -- The stock of an item the station aims at, in cycles of what its lines consume or make of it.
    self.stockCycles = stockCycles or 8
    self.interval = interval or 6
    self.nextRepostAt = nil
end)

---@return number cycles
function MarketMakerComponent:getStockCycles()
    return self.stockCycles
end

---@return number seconds
function MarketMakerComponent:getInterval()
    return self.interval
end

---@return number|nil time Simulated seconds
function MarketMakerComponent:getNextRepostAt()
    return self.nextRepostAt
end

---@param time number Simulated seconds
function MarketMakerComponent:setNextRepostAt(time)
    self.nextRepostAt = time
end

return MarketMakerComponent
