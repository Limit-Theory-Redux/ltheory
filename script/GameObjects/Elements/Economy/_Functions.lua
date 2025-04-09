local Jobs = requireAll('GameObjects.Jobs')
local Item = require('Legacy.Systems.Economy.Item')

local function iterateThroughFactoryInputs(self, station)
    local factory = station:getFactory()
    local prodLines = factory:getProds()

    for _, prodLine in ipairs(prodLines) do
        -- Iterate through all Inputs for this production line
        for _, input in prodLine.type:iterInputs() do
            local item = input.item
            if item:hasItem(Item.T2, item) then -- is Input item a minable resource?
                local itemBidVol = station:getTrader():getBidVolume(item)
                -- Does the Trader at the same station as this Factory have any bids for the minable Item?
                if itemBidVol > 0 then
                    -- Get no more than [considerCount] of the asteroids in the zone of this station
                    -- whose current Yield is at least the number of the trader's bids for this Input Item
                    local considerCount = 50
                    local baseYield = itemBidVol * 3

                    --[[
                    for i, asteroid in station.zone:iterChildren() do
                        if asteroid:hasYield() and asteroid:getYieldSize() > baseYield then
                            insert(self.yields, asteroid)
                        end
                        if i > considerCount then break end
                    end
                    printf("ECONOMY: Mine job test: station = %s, prod = %s, item = %s, #asteroids = %d",
                        station:getName(), prodLine.type.name, item:getName(), #self.yields)
                    ]]--

                    for i, asteroid in ipairs(self.yields) do
                        --printf("ECONOMY: src = %s, dst = %s, item = %s, itemBidVol = %d",
                        -- asteroid:getName(), station:getName(), item:getName(), itemBidVol)
                        insert(self.jobs[Enums.Jobs.Mining], Jobs.Mine(asteroid, station, item))
                        if i == considerCount then break end
                    end
                end
            end
        end
    end
end

local function cacheMiningJobs(self)
    -- Cache mining jobs
    -- Iterate through all factories at functional space stations in this star system
    self.jobs[Enums.Jobs.Mining] = {}
    for _, station in ipairs(self.factories) do
        -- Iterate through all production lines in the factory
        iterateThroughFactoryInputs(self, station)
    end
end

local function cacheTransportJobs(self)
    self.jobs[Enums.Jobs.Transport] = {}

    for _, src in ipairs(self.traders) do
        if src:hasDockable() and src:isDockable() and not src:isDestroyed() then
            for item, data in pairs(src:getTrader().elems) do
                if src:getTrader():getAskVolume(item) > 0 then
                    local buyPrice = src:getTrader():getBuyFromPrice(item, 1)
                    --printf("Buy? item %s from %s, buyPrice = %d", item:getName(), src:getName(), buyPrice)
                    if buyPrice > 0 then
                        for _, dst in ipairs(self.traders) do
                            if dst:hasDockable() and dst:isDockable() and not dst:isDestroyed() then
                                if src ~= dst then
                                    local sellPrice = dst:getTrader():getSellToPrice(item, 1)
                                    --printf("Transport test: item %s from %s @ buyPrice = %d to %s @ sellPrice = %d",
                                    -- item:getName(), src:getName(), buyPrice, dst:getName(), sellPrice)
                                    if buyPrice < sellPrice then
                                        --printf("Transport job insert: item %s from %s @ buyPrice = %d to %s @ sellPrice = %d",
                                        -- item:getName(), src:getName(), buyPrice, dst:getName(), sellPrice)
                                        insert(self.jobs[Enums.Jobs.Transport], Jobs.Transport(src, dst, item))
                                    end
                                end
                            end
                        end
                    end
                end
            end
        end
    end
end

local function cacheMaraudingJobs(self)
    self.blackMarketJobs[Enums.BlackMarketJobs.Marauding] = {}

    for _, src in ipairs(self.blackMarketTraders) do
        if src:hasDockable() and src:isDockable() and not src:isDestroyed() then
            for item, data in pairs(src:getBlackMarketTrader().elems) do
                if src:getBlackMarketTrader():getBidVolume(item) > 0 then
                    local sellPrice = src:getBlackMarketTrader():getSellToPrice(item, 1)
                    --printf("Buy? item %s from %s, buyPrice = %d", item:getName(), src:getName(), buyPrice)
                    if sellPrice > 0 then
                        --printf("Marauding job insert: item %s from %s @ buyPrice = %d to %s @ sellPrice = %d",
                        -- item:getName(), src:getName(), buyPrice, dst:getName(), sellPrice)
                        insert(self.blackMarketJobs[Enums.BlackMarketJobs.Marauding], Jobs.Marauding(src, src:getRoot())) --TODO: should also be able to extent to other systems.
                    end
                end
            end
        end
    end
end

local function cachePatrollingJobs(self)
    self.jobs[Enums.Jobs.Patrolling] = {}

    for _, src in ipairs(self.traders) do
        if src:hasDockable() and src:isDockable() and not src:isDestroyed() then
            local system = src:getRoot()
            local zone = src:getZone()
            if zone and zone.threatLevel > 5 then
                local patrolPoints = {}
                for i = 1, 10, 1 do
                    table.insert(patrolPoints, zone:getRandomPos(system.rng))
                end
                insert(self.jobs[Enums.Jobs.Patrolling], Jobs.Patrolling(src, system, patrolPoints))
            end
        end
    end
end

-- Spread economy functions for cleaner code
local returnTable = {
    cacheMiningJobs = cacheMiningJobs,
    cacheTransportJobs = cacheTransportJobs,
    cacheMaraudingJobs = cacheMaraudingJobs,
    cachePatrollingJobs = cachePatrollingJobs
}

return returnTable
