local Production = require('Systems.Economy.Production')
local Item = require('Systems.Economy.Item')
local Actions = requireAll('GameObjects.Actions')
local Words = require('Systems.Gen.Words')
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

local function addSystemGenerics(system)
  -- Market
  -- Add system-wide AI director
  system.tradeAI = Entities.Player("AI Trade Player")
  local tradeAi = system.tradeAI
  tradeAi:addCredits(1e10)

  -- Add a generic ship-like entity to serve as the imaginary player ship
  system.tradeShip = Entity()
  system.tradeShip:setOwner(tradeAi)

  -- Every inhabited star system gets one "free" solar plant
  -- TODO: Don't do this step for star systems that are not inhabited
  system:spawnStation(tradeAi, Production.EnergySolar)

  if GameState.gen.nAIPlayers > 0 and GameState.gen.nEconNPCs > 0 then
    -- Add the "extra" stations only if there are economic ships to use them
    -- Add a free Waste Recycler station
    system:spawnStation(tradeAi, Production.Recycler)
  end
  system:spawnStation(tradeAi, Production.Silicon) -- temp to boost economy
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

  for i=1, aiPlayerCount do
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
    -- Spawn space stations (start count at *2* for inhabited star systems -- see above)
    for i = 2, GameState.gen.nStations do
      -- Create Stations within randomly selected AsteroidField Zones
      system:spawnStation(aiPlayer, nil)
    end
    print("Spawned %d Stations for AI Player %s", GameState.gen.nStations, aiPlayer:getName())
    -- Add AI Player to the system
    table.insert(system.aiPlayers, aiPlayer)
  end
end

local function addBlackMarket(system)
  local piratesCount = 24
  local piratePlayer = Entities.Player("Captain " .. Words.getCoolName(rng))
  piratePlayer:addCredits(Config.econ.eStartCredits)
  system.pirateStation = system:spawnPirateStation(piratePlayer)
  system.pirateStation:setDisposition(GameState.player.humanPlayer:getControlling(), Config.game.dispoMin)
  GameState.player.humanPlayer:getControlling():setDisposition(system.pirateStation, Config.game.dispoMin)

  system:spawnAI(piratesCount, Actions.Wait(1), piratePlayer)
  printf("%d assets added to %s", piratesCount, piratePlayer:getName())
  -- Configure assets
  for asset in piratePlayer:iterAssets() do
    asset:setDisposition(GameState.player.humanPlayer:getControlling(), Config.game.dispoMin)
    GameState.player.humanPlayer:getControlling():setDisposition(asset, Config.game.dispoMin)
    if Config:getObjectInfo("object_types", asset:getType()) == "Ship" then
      asset:setHealth(100, 100, 0.2)
      asset.usesBoost = true
    end
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
