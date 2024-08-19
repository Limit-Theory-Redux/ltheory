-- Entities
local PlayerEntity = require("Entities.Player")

-- Systems
local GlobalStorage = require("Systems.GlobalStorage")

---@class UniverseEconomy
local UniverseEconomy = Class(function(self, seed)
    ---@cast self UniverseEconomy
    self:init(seed)
end)

---@param seed integer
function UniverseEconomy:init(seed)
    self.systems = {
        highAttention = {},
        lowAttention = {}
    }

    self.econDelta = 0
    self.nextUpdate = 0

    self:registerEvents()
end

function UniverseEconomy:registerEvents()
    EventBus:subscribe(FrameStage.ToString(FrameStage.PreRender), self, self.onPreRender)
end

---@param data EventData
function UniverseEconomy:onPreRender(data)
    self.econDelta = self.econDelta + data:getDeltaTime()
    -- High Attention
    for _, system in ipairs(self.systems.highAttention) do
        -- generate aiPlayers
        if not system.aiPlayers then
            addMarket(system)
            addBlackMarket(system)
            addEscorts(system) -- TEMP: for feature testing (do not remove until NPC ship spawning is implemented)
            self:addSystemGenerics(system)
            Log.Debug("System: " .. system:getName() .. " has " .. #system.ships .. " ships.")
        end

        -- Handle High Attention Systems
        self:handleHighAttention(dt, system)
    end

    if self.econDelta > self.nextUpdate then
        -- Low Attention
        for _, system in ipairs(self.systems.lowAttention) do
            -- Handle Low Attention Systems
            self:handleLowAttention(dt, system)
        end
        self.nextUpdate = self.econDelta + Config.econ.lowAttentionUpdateRate
    end
end

function UniverseEconomy:addSystemGenerics(system)
    local solarStationEntity = SpaceStationEntity(Enums.StationHulls.Small, ProductionType.EnergySolar)
    GlobalStorage:storeEntity(solarStationEntity)

    local siliconStationEntity = SpaceStationEntity(Enums.StationHulls.Small, ProductionType.Silicon)
    GlobalStorage:storeEntity(siliconStationEntity)

    local recyclerStationEntity = SpaceStationEntity(Enums.StationHulls.Small, ProductionType.Recycler)
    GlobalStorage:storeEntity(recyclerStationEntity)
end
