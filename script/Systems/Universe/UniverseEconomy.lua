local Production = require('Systems.Economy.Production')
local Item = require('Systems.Economy.Item')
local Actions = requireAll('GameObjects.Actions')
local Words = require('Systems.Gen.Words')
local rng = RNG.FromTime()

local UniverseEconomy = class(function(self) end)

function UniverseEconomy:Init()
    self.systems = {
        highAttention = {},
        lowAttention = {}
    }

    self.econDelta = 0
    self.nextUpdate = 0
end

local function addSystemGenerics(system)
    -- Add system-wide AI director
    system.tradeAI = Entities.Player("AI Trade Player")
    local tradeAi = system.tradeAI
    tradeAi:addCredits(1e10)

    -- Add a generic ship-like entity to serve as the imaginary player ship
    system.tradeShip = Entity()
    system.tradeShip:setOwner(tradeAi, true)

    -- Every inhabited star system gets one "free" solar plant
    -- TODO: Don't do this step for star systems that are not inhabited
    system:spawnStation(Enums.StationHulls.Small, tradeAi, Production.EnergySolar)

    -- Every inhabited star system gets one "free" silicon refinery
    system:spawnStation(Enums.StationHulls.Small, tradeAi, Production.Silicon)

    -- Add a free Waste Recycler station
    system:spawnStation(Enums.StationHulls.Small, tradeAi, Production.Recycler)

    -- Spawn space stations (start count at *2* for inhabited star systems -- see above)
    for i = 3, GameState.gen.nStations do
        -- Create Stations within randomly selected AsteroidField Zones
        system:spawnStation(Enums.StationHulls.Small, tradeAi, nil)
    end
    Log.Debug("Spawned %d Stations for AI Player '%s'", GameState.gen.nStations, tradeAi:getName())

    -- Possibly add some additional factory stations based on which ones were randomly created and their inputs
    system:addExtraFactories(system, GameState.gen.nPlanets, tradeAi)
end

local function addMarket(system)
    -- create table for aiPlayers
    system.aiPlayers = {}
    local aiPlayerCount

    if GameState.randomizeAIPlayers then
        if GameState.gen.nAIPlayers <= 0 then
            aiPlayerCount = 0
        else
            aiPlayerCount = rng:getInt(1, GameState.gen.nAIPlayers)
        end
    else
        aiPlayerCount = GameState.gen.nAIPlayers
    end

    for i = 1, aiPlayerCount do
        -- temp name until we have rnd names
        local aiPlayer = Entities.Player("AI Trade Player " .. i)
        aiPlayer:addCredits(Config.econ.eStartCredits)

        local factionType = math.random(1, #Enums.FactionTypeNames)
        local factionName

        -- TODO: TRAITS & CHANCES -> BASE FACTION TYPE ON OWNER TRAITS / OWNER BIRTHPLACE TRAITS

        if factionType == Enums.FactionType.Corporation or
            factionType == Enums.FactionType.TradingGuild or
            factionType == Enums.FactionType.Empire then
            do
                factionName = Words.getCoolName(rng) .. " " .. Enums.FactionTypeNames[factionType]
            end
        else
            factionName = Enums.FactionTypeNames[factionType] .. " " .. Words.getCoolName(rng)
        end

        local playerFaction = Entities.Faction({
            name = factionName,
            type = factionType,
            owner = aiPlayer
        })

        -- Create assets (ships)
        local aiAssetCount

        if GameState.gen.randomizeEconNPCs then
            if GameState.gen.nEconNPCs <= 0 then
                aiAssetCount = 0
            else
                aiAssetCount = rng:getInt(1, GameState.gen.nEconNPCs)
            end
        else
            aiAssetCount = GameState.gen.nEconNPCs
        end

        system:spawnAI(aiAssetCount, Actions.Wait(1), aiPlayer)
        printf("%d assets added to %s", aiAssetCount, aiPlayer:getName())
        -- Configure assets
        for asset in aiPlayer:iterAssets() do
            asset:setFaction(playerFaction)
            system:place(asset)
        end

        -- Tell AI player to start using the Think action
        aiPlayer:pushAction(Actions.Think())
        -- Spawn space stations (start count at *2* for inhabited star systems -- see above)
        for i = 2, GameState.gen.nStations do
            -- Create Stations within randomly selected AsteroidField Zones
            system:spawnStation(Enums.StationHulls.Small, aiPlayer)
        end
        print("Spawned %d Stations for AI Player %s", GameState.gen.nStations, aiPlayer:getName())
        -- Add AI Player to the system
        table.insert(system.aiPlayers, aiPlayer)
    end
end

local function addBlackMarket(system)
    local aiPirateCount

    if GameState.gen.randomizePirateNPCs then
        if GameState.gen.nPirateNPCs <= 0 then
            aiPirateCount = 0
        else
            aiPirateCount = rng:getInt(1, GameState.gen.nPirateNPCs)
        end
    else
        aiPirateCount = GameState.gen.nPirateNPCs
    end

    local piratePlayer = Entities.Player("Captain " .. Words.getCoolName(rng))
    piratePlayer:addCredits(Config.econ.eStartCredits)

    local factionName = Words.getCoolName(rng) .. " " .. Enums.FactionTypeNames[Enums.FactionType.Marauders]

    local playerFaction = Entities.Faction({
        name = factionName,
        type = Enums.FactionType.Marauders,
        owner = piratePlayer
    })

    system.pirateStation = system:spawnPirateStation(Enums.StationHulls.Small, piratePlayer)
    -- disp shouldn't be based on a ship tbh, also replace with faction/ai player dispo later
    system.pirateStation:setDisposition(GameState.player.currentShip, Config.game.dispoMin)
    GameState.player.currentShip:setDisposition(system.pirateStation, Config.game.dispoMin)

    system:spawnAI(aiPirateCount, Actions.Wait(1), piratePlayer)
    printf("%d assets added to %s", aiPirateCount, piratePlayer:getName())
    -- Configure assets
    for asset in piratePlayer:iterAssets() do
        asset:setDisposition(GameState.player.currentShip, Config.game.dispoMin)
        GameState.player.currentShip:setDisposition(asset, Config.game.dispoMin)

        if Config:getObjectInfo("object_types", asset:getType()) == "Ship" then
            local pirateHullInteg = asset:mgrHullGetHealthMax()
            asset:mgrHullSetHealth(pirateHullInteg, pirateHullInteg)
            asset.usesBoost = true
        end
        asset:setFaction(playerFaction)
        system:place(asset)
    end
    piratePlayer:pushAction(Actions.CriminalThink())
    table.insert(system.aiPlayers, piratePlayer)
end

function UniverseEconomy:OnUpdate(dt)
    self.econDelta = self.econDelta + dt
    -- High Attention
    for _, system in ipairs(self.systems.highAttention) do
        -- generate aiPlayers
        if not system.aiPlayers then
            addMarket(system)
            addBlackMarket(system)
            addSystemGenerics(system)
            Log.Debug("System: " .. system:getName() .. " has " .. #system.ships .. " ships.")
        end

        -- Handle High Attention Systems
        self:HandleHighAttention(dt, system)
    end

    if self.econDelta > self.nextUpdate then
        -- Low Attention
        for _, system in ipairs(self.systems.lowAttention) do
            -- Handle Low Attention Systems
            self:HandleLowAttention(dt, system)
        end
        self.nextUpdate = self.econDelta + Config.econ.lowAttentionUpdateRate
    end
end

function UniverseEconomy:AddSystem(system)
    Log.Debug("Adding a new system to universe economy: " .. system:getName())

    table.insert(self.systems.highAttention, system)
end

function UniverseEconomy:HandleHighAttention(dt, system)
    system:updateEconomy(dt) --> script\GameObjects\Elements\Economy\Economy.lua
end

function UniverseEconomy:HandleLowAttention(dt, system)

end

return UniverseEconomy
