local Production = require('Systems.Economy.Production')
local Item = require('Systems.Economy.Item')
local Actions = requireAll('GameObjects.Actions')
local Words = require('Systems.Gen.Words')
local rng = RNG.FromTime()

local UniverseEconomy = class(function(self) end)

function UniverseEconomy:init()
    self.systems = {
        highAttention = {},
        lowAttention = {}
    }

    self.econDelta = 0
    self.nextUpdate = 0
end

function UniverseEconomy:registerEvents()
    EventBus:subscribe(EventType.PostSim, self, self.onPostSim)
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
        printf("Spawned %d stations for AI Player '%s'", GameState.gen.nStations, aiPlayer:getName())
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

local function addEscorts(system)
    -- Add escort ships for testing
    -- THIS FUNCTION IS TEMPORARY
    -- IT WILL GO AWAY WHEN WE CREATE THE REAL CODE FOR SPAWNING NPC SHIPS
    -- UNTIL THEN IT IS USEFUL FOR FEATURE TESTING
    local rng = RNG.FromTime()
    local escortShips = {}
    if GameState.gen.nEscortNPCs > 0 then
        local playerShip = GameState.player.currentShip
        for i = 1, GameState.gen.nEscortNPCs do
            local escort = system:spawnShip(rng:choose({ 1, 2, 3, 4, 5, 6 }), nil)
            local offset = system.rng:getSphere():scale(100)
            escort:setPos(playerShip:getPos() + offset)
            escort:setOwner(playerShip:getOwner(), true)

            if i > GameState.gen.nEscortNPCs / 2 then
                escort:pushAction(Actions.Orbit(playerShip, rng:getInt(4, 10) * 10, rng:getInt(10, 40)))
            else
                escort:pushAction(Actions.Escort(playerShip, offset))
            end

            -- TEMP: a few NPC escort ships get to be "aces" with extra health and maneuverability
            --       These will be dogfighting challenges!
            if rng:getInt(0, 100) < 20 then
                local escortHullInteg = escort:mgrHullGetHealthMax()
                escort:mgrHullSetHealth(floor(escortHullInteg * 1.5), floor(escortHullInteg * 1.5))
                escort.usesBoost = true
            end

            insert(escortShips, escort)
        end
        -- TESTING: push Attack onto action queue of escort ships
        for i = 1, #escortShips - 1 do
            escortShips[i]:pushAction(Actions.Attack(escortShips[i + 1]))
        end
        Log.Debug("Added %d escort ships", GameState.gen.nEscortNPCs)
    end
end

function UniverseEconomy:onPostSim(dt)
    self.econDelta = self.econDelta + dt
    -- High Attention
    for _, system in ipairs(self.systems.highAttention) do
        -- generate aiPlayers
        if not system.aiPlayers then
            addMarket(system)
            addBlackMarket(system)
            addEscorts(system) -- TEMP: for feature testing (do not remove until NPC ship spawning is implemented)
            addSystemGenerics(system)
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

function UniverseEconomy:addSystem(system)
    Log.Debug("Adding a new system to universe economy: " .. system:getName())

    table.insert(self.systems.highAttention, system)
end

function UniverseEconomy:handleHighAttention(dt, system)
    system:updateEconomy(dt) --> script\GameObjects\Elements\Economy\Economy.lua
end

function UniverseEconomy:handleLowAttention(dt, system)

end

return UniverseEconomy
