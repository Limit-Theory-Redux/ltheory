local Production = require('Systems.Economy.Production')
local Item = require('Systems.Economy.Item')
local Actions = requireAll('GameObjects.Actions')
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

local function AddSystemGenerics(system)
    -- Add system-wide AI director
    system.tradeAI = Entities.Player("AI Trade Player")
    local tradeAi = system.tradeAI
    tradeAi:addCredits(1e10)

    -- Add a generic ship-like entity to serve as the imaginary player ship
    system.tradeShip = Entity()
    system.tradeShip:setOwner(tradeAi)

    -- Every inhabited star system gets one "free" solar plant
    -- TODO: Don't do this step for star systems that are not inhabited
    system:spawnStation(Enums.StationHulls.Small, tradeAi, Production.EnergySolar)

    -- Add a free Waste Recycler station
    system:spawnStation(Enums.StationHulls.Small, tradeAi, Production.Recycler)

    -- Spawn space stations (start count at *2* for inhabited star systems -- see above)
    for i = 3, GameState.gen.nStations do
        -- Create Stations within randomly selected AsteroidField Zones
        system:spawnStation(Enums.StationHulls.Small, tradeAi, nil)
    end
    printf("Spawned %d Stations for AI Player '%s'", GameState.gen.nStations, tradeAi:getName())

    -- Possibly add some additional factory stations based on which ones were randomly created and their inputs
    system:addExtraFactories(system, GameState.gen.nPlanets, tradeAi)
end

function UniverseEconomy:OnUpdate(dt)
    self.econDelta = self.econDelta + dt
    -- High Attention
    for _, system in ipairs(self.systems.highAttention) do
        -- generate aiPlayers
        if not system.aiPlayers then
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
                    system:place(asset)
                end

                -- Tell AI player to start using the Think action
                aiPlayer:pushAction(Actions.Think())

                -- Add AI Player to the system
                table.insert(system.aiPlayers, aiPlayer)
            end
            print("System: " .. system:getName() .. " has " .. #system.ships .. " ships.")
        end
        -- Handle High Attention Systems
        self:HandleHighAttention(system)
    end

    if self.econDelta > self.nextUpdate then
        -- Low Attention
        for _, system in ipairs(self.systems.lowAttention) do
            -- Handle Low Attention Systems
            self:HandleLowAttention(system)
        end
        self.nextUpdate = self.econDelta + Config.econ.lowAttentionUpdateRate
    end
end

function UniverseEconomy:AddSystem(system)
    print("Adding a new system to universe economy: " .. system:getName())

    -- Add star system's AI Director and space stations only once per system
    AddSystemGenerics(system)

    table.insert(self.systems.highAttention, system)
end

function UniverseEconomy:HandleHighAttention(system)

end

function UniverseEconomy:HandleLowAttention(system)

end

return UniverseEconomy
