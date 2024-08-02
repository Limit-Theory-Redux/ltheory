-- Entities
local SpaceStationEntity = require("Entities.Constructs.SpaceStation")

-- Systems
local GlobalStorage = require("Systems.GlobalStorage")
local Universe = require("Systems.Economy.Universe")

---@class UniverseEconomy
local UniverseEconomy = Class(function(self, seed)
    ---@cast self UniverseEconomy
    self:init(seed)
end)

---@param seed integer
function UniverseEconomy:init(seed)
    self:registerEvents()
end

function UniverseEconomy:registerEvents()
    EventBus:subscribe(FrameStage.ToString(FrameStage.PreRender), self, self.onPreRender)
end

---@param data EventData
function UniverseEconomy:onPreRender(data)

end

---@param system StarSystem
function UniverseEconomy:addSystemGenerics(system)
    local solarStationEntity = SpaceStationEntity(Enums.StationHulls.Small, ProductionType.EnergySolar)
    GlobalStorage:storeEntity(solarStationEntity)

    local siliconStationEntity = SpaceStationEntity(Enums.StationHulls.Small, ProductionType.Silicon)
    GlobalStorage:storeEntity(siliconStationEntity)

    local recyclerStationEntity = SpaceStationEntity(Enums.StationHulls.Small, ProductionType.Recycler)
    GlobalStorage:storeEntity(recyclerStationEntity)
end
