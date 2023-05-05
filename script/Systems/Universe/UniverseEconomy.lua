local Production = require('Systems.Economy.Production')
local Item = require('Systems.Economy.Item')
local Actions = requireAll('GameObjects.Actions')
local rng = RNG.FromTime()

local UniverseEconomy = class(function (self) end)

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
  -- Every systen gets one "free" solar plant
  local newStation = system:spawnStation(tradeAi, Production.EnergySolar)
  system:place(newStation)

  if Config.gen.nAIPlayers > 0 and Config.gen.nEconNPCs > 0 then
      -- Add the "extra" stations only if there are economic ships to use them
      -- Add a free Waste Recycler station
      system:spawnStation(tradeAi, Production.Recycler)
      system:place(newStation)
  end

  local aiStationCount = rng:getInt(1, Config.gen.nStations)

  print("Spawn " .. aiStationCount .. " Stations for: " .. tradeAi:getName())
  for i=1, aiStationCount do
    -- Create Stations within randomly selected AsteroidField Zones
    system:spawnStation(tradeAi, nil)
  end
  -- Possibly add some additional factory stations based on which ones were randomly created and their inputs
  system:addExtraFactories(system, Config.gen.nPlanets, tradeAi)
end

function UniverseEconomy:OnUpdate(dt)
  self.econDelta = self.econDelta + dt
  -- High Attention
  for _, system in ipairs(self.systems.highAttention) do
    -- generate aiPlayers
    if not system.aiPlayers then
      -- create table for aiPlayers
      system.aiPlayers = {}
      for i=1, rng:getInt(1, 3) do
        -- temp name until we have rnd names
        local aiPlayer = Entities.Player("AI Trade Player " .. i)
        aiPlayer:addCredits(Config.econ.eStartCredits)
        -- Create assets (ships)
        local aiAssetCount = rng:getInt(10, Config.gen.nEconNPCs)
        system:spawnAI(aiAssetCount, Actions.Wait(1), aiPlayer)
        printf("%d assets added to %s", aiAssetCount, aiPlayer:getName())
        -- Configure assets
        for asset in aiPlayer:iterAssets() do
          system:place(asset)
        end

        -- Tell AI player to start using the Think action
        aiPlayer:pushAction(Actions.Think())

        -- Temporary
        AddSystemGenerics(system)

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
  table.insert(self.systems.highAttention, system)
end

function UniverseEconomy:HandleHighAttention(system)

end

function UniverseEconomy:HandleLowAttention(system)

end

return UniverseEconomy