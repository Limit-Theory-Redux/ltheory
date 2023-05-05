local UniverseEconomy = require('Systems.Universe.UniverseEconomy')
local System = require('GameObjects.Entities.Test.System')

local rng = RNG.FromTime()

local Universe = class(function (self) end)

function Universe:Init()
  self.systems = {}
  self.players = {}
  self.aiPlayers = {}
  self.factions = {}
  self.economy = UniverseEconomy:Init()
end

function Universe:OnUpdate(dt)
  UniverseEconomy:OnUpdate(dt)
end

function Universe:CreateStarSystem(seed)
  -- Spawn a new star system
  local system = System(seed)
  GameState.world.currentSystem = system -- remember the player's current star system

  do
    -- Flight Mode
    -- Reset variables used between star systems
    GameState.gamePaused          = false
    GameState.panelActive         = false
    GameState.player.playerMoving = false
    GameState.player.weaponGroup  = 1

    -- Generate a new star system with nebulae/dust, a planet, an asteroid field,
    --   a space station, a visible pilotable ship, and possibly some NPC ships
    local afield = nil

    -- Add planets
    local planet = nil -- remember the last planet created (TODO: remember ALL the planets)
    for i = 1, Config.gen.nPlanets do
      planet = system:spawnPlanet(false)
    end

    -- Add asteroid fields
    -- Must add BEFORE space stations
    for i = 1, Config.gen.nFields do
      afield = system:spawnAsteroidField(Config.gen.nAsteroids, false)
      printf("Added %s asteroids to %s", Config.gen.nAsteroids, afield:getName())
    end

    -- Add the player's ship
    local playerShip = system:spawnShip(GameState.player.humanPlayer)
    playerShip:setName(GameState.player.humanPlayerShipName)
    playerShip:setHealth(500, 500, 10) -- make the player's ship healthier than the default NPC ship

    -- Insert ship into this star system
    playerShip:setPos(Config.gen.origin)
    playerShip:setFriction(0)
    playerShip:setSleepThreshold(0, 0)
    playerShip:setOwner(GameState.player.humanPlayer)
    system:addChild(playerShip)
    GameState.player.humanPlayer:setControlling(playerShip)

    GameState.player.currentShip = playerShip

    -- Set our ship's starting location within the extent of a random asteroid field
    system:place(playerShip)
    printf("Added our ship, the '%s', at pos %s", playerShip:getName(), playerShip:getPos())

    -- Add System to the Universe
    table.insert(self.systems, system)
    printf("Added System: " .. system:getName() .. " to the Universe.")
  end
  self:AddSystemEconomy(system)
end

function Universe:AddFaction(name, type, players)

end

function Universe:AddSystemEconomy(system)

end

return Universe