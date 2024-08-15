-- Entities
local SpaceStationEntity = require("_ECS_WIP_TEMP.Entities.Constructs.SpaceStation") --!temp path

-- Systems
--local GlobalStorage = require("_ECS_WIP_TEMP.Systems.GlobalStorage") --!temp path

-- Utils
local Words = require("_ECS_WIP_TEMP.Systems.Generators.Words") --!temp path

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
    -- Get RNG Component
    local systemRNGComponent = assert(system:findComponentByName("RandomNumberGenerator"))
    ---@cast systemRNGComponent RandomNumberGeneratorComponent
    local systemRNG = systemRNGComponent:getRNG()

    -- Create Space Station Entity
    local solarStationEntity = SpaceStationEntity(Words.getCoolName(systemRNG), Enums.StationHulls.Small, systemRNG:get64())
    -- Store Entity
    GameState.globalStorage:storeEntity(solarStationEntity) --!temp fix

    -- Create Space Station Entity
    local siliconStationEntity = SpaceStationEntity(Words.getCoolName(systemRNG), Enums.StationHulls.Small, systemRNG:get64())
    -- Store Entity
    GameState.globalStorage:storeEntity(siliconStationEntity) --!temp fix

    -- Create Space Station Entity
    local recyclerStationEntity = SpaceStationEntity(Words.getCoolName(systemRNG), Enums.StationHulls.Small, systemRNG:get64())
    -- Store Entity
    GameState.globalStorage:storeEntity(recyclerStationEntity) --!temp fix
end

return UniverseEconomy
