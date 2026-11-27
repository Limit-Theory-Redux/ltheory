local Registry = require("Core.ECS.Registry")
local Items = require("Shared.Registries.Items")
local Economy = require("Modules.Economy.Components")
local InventoryManager = require("Modules.Economy.Managers.InventoryManager")
local AccountManager = require("Modules.Economy.Managers.AccountManager")
local MarketplaceSystem = require("Modules.Economy.Systems.MarketplaceSystem")
local SpawnerSystem = require("Modules.Spawning.Systems.SpawnerSystem")
local PeopleManager = require("Modules.People.Managers.PeopleManager")
local Factions = require("Modules.Factions.Components")

-- Design: doc/script/economy-sim.md

---Watches an economy over time: every `intervalS` simulated seconds it takes what the economy is
---(each station's credits, stock and prices; the agents by state and kind; what they earned and spent;
---the factions' treasuries; every credit there is) and keeps the series the charts draw, the last
---`keep` samples of each. With a CSV path it writes every sample as rows of `time,kind,name,key,value`,
---and a summary at `finish`.
---@class EconomySampler
---@overload fun(self: EconomySampler, options: table|nil): EconomySampler class internal
---@overload fun(options: table|nil): EconomySampler class external
local EconomySampler = Class("EconomySampler", function(self, options)
    options = options or {}
    self.intervalS = options.intervalS or 60
    self.keep = options.keep or 240
    self.nextAt = 0
    self.samples = 0
    ---@type table<string, number[]> Series name -> values, oldest first
    self.series = {}
    ---@type number[] The times of the kept samples
    self.times = {}
    self.first = nil
    self.last = nil
    if options.csvPath then
        self.file = io.open(options.csvPath, "w")
        if self.file then
            self.file:write("time,kind,name,key,value\n")
        else
            Log.Warn("EconomySampler: can't write %s", options.csvPath)
        end
    end
end)

local stateNames = nil

---@param state integer
---@return string
local function stateName(state)
    if not stateNames then
        stateNames = {}
        for name, value in pairs(Enums.EconomyAgentState) do stateNames[value] = name end
    end
    return stateNames[state] or tostring(state)
end

---@param name string
---@param value number
function EconomySampler:push(name, value)
    local list = self.series[name]
    if not list then
        list = {}
        -- A series that starts late is padded, so all line up with `times`.
        for _ = 1, #self.times - 1 do list[#list + 1] = value end
        self.series[name] = list
    end
    list[#list + 1] = value
    if #list > self.keep then table.remove(list, 1) end
end

---@param now number
---@param kind string
---@param name string
---@param key string
---@param value number
function EconomySampler:row(now, kind, name, key, value)
    if self.file then self.file:write(string.format("%.0f,%s,%s,%s,%.3f\n", now, kind, name, key, value)) end
end

---Takes a sample when one is due.
---@param now number Simulated seconds
function EconomySampler:update(now)
    if now < self.nextAt then return end
    self.nextAt = now + self.intervalS
    self:sample(now)
end

---Takes a sample now.
---@param now number Simulated seconds
---@return table sample The totals (also kept as `last`)
function EconomySampler:sample(now)
    self.samples = self.samples + 1
    self.times[#self.times + 1] = now
    if #self.times > self.keep then table.remove(self.times, 1) end

    local totals = { time = now, stationCredits = 0, agentCredits = 0, treasuries = 0, earned = 0, spent = 0, trips = 0,
        agents = 0, stock = {}, priceSum = {}, priceCount = {}, states = {} }

    for _, station in ipairs(SpawnerSystem:getStations()) do
        local entity = station.entity
        if Registry:hasEntity(entity) then
            local credits = AccountManager:balanceOf(entity) + MarketplaceSystem:creditsHeldBy(entity, entity)
            totals.stationCredits = totals.stationCredits + credits
            self:row(now, "station", station.name, "credits", credits)
            local inventory = entity:get(Economy.Inventory)
            local marketplace = entity:get(Economy.Marketplace)
            local items = {}
            for itemId in pairs(inventory and inventory:getInventory() or {}) do items[itemId] = true end
            for itemId in pairs(marketplace and marketplace:getMarketPrices() or {}) do items[itemId] = true end
            for itemId in pairs(items) do
                local name = Items:getDefinition(itemId).name
                local stock = (inventory and InventoryManager:count(inventory, itemId) or 0)
                    + MarketplaceSystem:goodsHeldBy(entity, entity, itemId)
                totals.stock[name] = (totals.stock[name] or 0) + stock
                self:row(now, "stock", station.name, name, stock)
                local price = marketplace and marketplace:getMarketPrice(itemId)
                if price and price > 0 then
                    totals.priceSum[name] = (totals.priceSum[name] or 0) + price
                    totals.priceCount[name] = (totals.priceCount[name] or 0) + 1
                    self:row(now, "price", station.name, name, price)
                end
            end
        end
    end

    local seenAccounts = {}
    for _, ship in ipairs(SpawnerSystem:getAgents()) do
        local agent = Registry:hasEntity(ship.entity) and PeopleManager:commandOf(ship.entity, Economy.EconomyAgent)
        if agent then
            totals.agents = totals.agents + 1
            local state = stateName(agent:getState())
            totals.states[state] = (totals.states[state] or 0) + 1
            local trips, earned, spent = agent:getStats()
            totals.trips, totals.earned, totals.spent = totals.trips + trips, totals.earned + earned, totals.spent + spent
            local account = AccountManager:accountOf(ship.entity)
            if account and not seenAccounts[account] then
                seenAccounts[account] = true
                totals.agentCredits = totals.agentCredits + account:getAccountBalance()
            end
        end
    end
    for _, faction in ipairs(SpawnerSystem:getFactions()) do
        if Registry:hasEntity(faction.entity) then
            local treasury = AccountManager:balanceOf(faction.leader)
            totals.treasuries = totals.treasuries + treasury
            self:row(now, "faction", faction.entity:get(Factions.Faction):getName(), "treasury", treasury)
        end
    end
    local allCredits = 0
    for _, account in Registry:view(Economy.PlayerBankAccount) do allCredits = allCredits + account:getAccountBalance() end
    totals.allCredits = allCredits

    for state, count in pairs(totals.states) do self:row(now, "agents", "state", state, count) end
    self:row(now, "total", "economy", "agents", totals.agents)
    self:row(now, "total", "economy", "trips", totals.trips)
    self:row(now, "total", "economy", "earned", totals.earned)
    self:row(now, "total", "economy", "spent", totals.spent)
    self:row(now, "total", "economy", "stationCredits", totals.stationCredits)
    self:row(now, "total", "economy", "allCredits", allCredits)

    -- The series the charts draw.
    for name, sum in pairs(totals.priceSum) do self:push("price:" .. name, sum / totals.priceCount[name]) end
    for name, stock in pairs(totals.stock) do self:push("stock:" .. name, stock) end
    for state, count in pairs(totals.states) do self:push("state:" .. state, count) end
    self:push("credits:stations", totals.stationCredits)
    self:push("credits:all", allCredits)
    self:push("trips", totals.trips)
    if self.file and self.samples % 10 == 0 then self.file:flush() end

    self.first = self.first or totals
    self.last = totals
    return totals
end

---What changed from the first sample to the last, as lines of text.
---@return string[]
function EconomySampler:summary()
    local first, last = self.first, self.last
    if not first or not last then return { "no samples" } end
    local hours = (last.time - first.time) / 3600
    local lines = {
        string.format("%.2f simulated hours, %d samples, %d agents", hours, self.samples, last.agents),
        string.format("trips %d, earned %.0f, spent %.0f", last.trips, last.earned, last.spent),
        string.format("station credits %.0f -> %.0f, all credits %.0f -> %.0f",
            first.stationCredits, last.stationCredits, first.allCredits, last.allCredits),
    }
    local names = {}
    for name in pairs(last.priceSum) do names[#names + 1] = name end
    table.sort(names)
    for _, name in ipairs(names) do
        local before = first.priceSum[name] and first.priceSum[name] / first.priceCount[name]
        local after = last.priceSum[name] / last.priceCount[name]
        lines[#lines + 1] = string.format("%s: price %s -> %.1f, stock %d -> %d", name,
            before and string.format("%.1f", before) or "-", after, first.stock[name] or 0, last.stock[name] or 0)
    end
    return lines
end

---Writes the summary to the log and the CSV, and closes the CSV.
function EconomySampler:finish()
    for _, line in ipairs(self:summary()) do
        Log.Info("EconomySampler: %s", line)
        if self.file then self.file:write("# " .. line .. "\n") end
    end
    if self.file then
        self.file:close()
        self.file = nil
    end
end

return EconomySampler
