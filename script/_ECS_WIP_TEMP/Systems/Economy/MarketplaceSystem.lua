-- Systems
local GlobalStorage = require("_ECS_WIP_TEMP.Systems.GlobalStorage") --!temp path

-- Utilities
local QuickProfiler = require("_ECS_WIP_TEMP.Shared.Tools.QuickProfiler")

---@class MarketplaceSystem
---@overload fun(self: MarketplaceSystem): MarketplaceSystem class internal
---@overload fun(): MarketplaceSystem class external
local MarketplaceSystem = Class(function(self)
    ---@diagnostic disable-next-line: invisible
    self:registerVars()
    ---@diagnostic disable-next-line: invisible
    self:registerEvents()
end)

---@private
function MarketplaceSystem:registerVars()
    ---@private
    self.profiler = QuickProfiler("MarketplaceSystem", false, false)

    self.rng = RNG.FromTime()
    self.maxUpdateRateDeviation = 0.5
    self.updateRate = 2
end

---@private
function MarketplaceSystem:registerEvents()
    EventBus:subscribe(Event.PreRender, self, self.onPreRender)
end

function MarketplaceSystem:onPreRender()
    self.profiler:start()
    --[[ Todo
    - Get marketplaces (x)
    - Check if marketplace has trader assigned (x)
    - Update offers
    - Update item flow
    ]]
    local marketplaces = GlobalStorage:getComponentsFromArchetype(Enums.ComponentArchetype.MarketplaceComponent)
    ---@cast marketplaces table<MarketplaceComponent>

    local now = TimeStamp.Now()

    if marketplaces and #marketplaces > 0 then
        ---@param marketplace MarketplaceComponent
        for index, marketplace in ipairs(marketplaces) do
            if not marketplace:getTrader() then
                goto skipMarketplace
            end

            local nextUpdate = marketplace:getNextUpdate()

            if not nextUpdate then
                nextUpdate = TimeStamp.GetFuture(self.updateRate + self.rng:getUniformRange(0, self.maxUpdateRateDeviation))
                marketplace:setNextUpdate(nextUpdate)
                goto skipMarketplace
            end

            -- update
            if now:getDifference(nextUpdate) > 0 then
                -- trader handles transaction
            end

            :: skipMarketplace ::
        end
    end
    self.profiler:stop()
end

return MarketplaceSystem()
