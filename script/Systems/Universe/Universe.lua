local UniverseEconomy = require('Systems.Universe.UniverseEconomy')
local System = require('GameObjects.Entities.Test.System')

local rng = RNG.FromTime()

local Universe = class(function (self) end)

function Universe:OnInit()
  self.systems = {}
  self.players = {}
  self.aiPlayers = {}
  self.factions = {}
  self.economy = UniverseEconomy:OnInit()
end

function Universe:AddStarSystem(seed)
  -- Spawn a new star system
  local system = System(self.seed)
  Config.game.currentSystem = self.system -- remember the player's current star system

  do
    -- Flight Mode

    -- Reset variables used between star systems
    Config.game.gamePaused   = false
    Config.game.panelActive  = false
    Config.game.playerMoving = false
    Config.game.weaponGroup  = 1

    -- Generate a new star system with nebulae/dust, a planet, an asteroid field,
    --   a space station, a visible pilotable ship, and possibly some NPC ships
    local afield = nil

    -- Add planets
    local planet = nil -- remember the last planet created (TODO: remember ALL the planets)
    for i = 1, Config.gen.nPlanets do
      planet = self.system:spawnPlanet(false)
    end

    -- Add asteroid fields
    -- Must add BEFORE space stations
    for i = 1, Config.gen.nFields do
      afield = self.system:spawnAsteroidField(Config.gen.nAsteroids, false)
      printf("Added %s asteroids to %s", Config.gen.nAsteroids, afield:getName())
    end

    -- Add the player's ship
    local playerShip = self.system:spawnShip(Config.game.humanPlayer)
    playerShip:setName(Config.game.humanPlayerShipName)
    playerShip:setHealth(500, 500, 10) -- make the player's ship healthier than the default NPC ship

    LTheoryRedux:insertShip(playerShip)

    Config.game.currentShip = playerShip

    -- Set our ship's starting location within the extent of a random asteroid field
    self.system:place(playerShip)
    printf("Added our ship, the '%s', at pos %s", playerShip:getName(), playerShip:getPos())

    -- Add System to the UniverseEconomy
    Universe:AddStarSystem(self.system)
  end
  self:AddSystemEconomy(system)
end

function Universe:AddFaction(name, type, players)

end

function Universe:AddSystemEconomy(system)

end

return Universe