local UniverseEconomy = require('Systems.Universe.UniverseEconomy')
local System = require('GameObjects.Entities.Test.System')
local Actions = requireAll('GameObjects.Actions')

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
    for i = 1, GameState.gen.nPlanets do
      planet = system:spawnPlanet(false)
    end

    -- Add asteroid fields
    -- Must add BEFORE space stations
    for i = 1, GameState.gen.nFields do
      afield = system:spawnAsteroidField(GameState.gen.nAsteroids, false)
      printf("Added %s asteroids to %s", GameState.gen.nAsteroids, afield:getName())
    end

    local shipObject = {
      owner = GameState.player.humanPlayer,
      shipName = GameState.player.humanPlayerShipName,
      health = {
        [1] = 500,
        [2] = 500,
        [3] = 1
      },
      friction = 0,
      sleepThreshold = {
        [1] = 0,
        [2] = 0
      }
    }

    local playerShip = self:CreateShip(system, nil, shipObject)

    GameState.player.currentShip = playerShip

    printf("Added our ship, the '%s', at pos %s", playerShip:getName(), playerShip:getPos())

    -- Escort Ships for Testing
    local escortShips = {}
    if GameState.gen.nEscortNPCs > 0 then
      for i = 1, GameState.gen.nEscortNPCs do
        local escort = system:spawnShip(nil)
        local offset = system.rng:getSphere():scale(100)
        escort:setPos(playerShip:getPos() + offset)
        escort:pushAction(Actions.Escort(playerShip, offset))

        -- TEMP: a few NPC escort ships get to be "aces" with extra health and maneuverability
        --       These will be dogfighting challenges!
        if rng:getInt(0, 100) < 20 then
          escort:setHealth(100, 100, 0.2)
          escort.usesBoost = true
        end

        insert(escortShips, escort)
      end
      -- TESTING: MAKE SHIPS CHASE EACH OTHER!
      for i = 1, #escortShips - 1 do
        escortShips[i]:pushAction(Actions.Attack(escortShips[i+1]))
      end
      printf("Added %d escort ships", GameState.gen.nEscortNPCs)
    end

    -- Add System to the Universe
    table.insert(self.systems, system)
    printf("Added System: " .. system:getName() .. " to the Universe.")
  end
  self:AddSystemEconomy(system)
end

function Universe:CreateShip(system, pos, shipObject)
    -- Add the player's ship
    local ship = system:spawnShip(shipObject.owner)
    ship:setName(shipObject.shipName)
    ship:setHealth(shipObject.health[1], shipObject.health[2], shipObject.health[3]) -- make the player's ship healthier than the default NPC ship

    -- Insert ship into this star system
    local spawnPosition = pos or Config.gen.origin
    ship:setPos(spawnPosition)
    ship:setFriction(shipObject.friction)
    ship:setSleepThreshold(shipObject.sleepThreshold[1], shipObject.sleepThreshold[2])
    ship:setOwner(shipObject.owner)
    system:addChild(ship)
    shipObject.owner:setControlling(ship)

    return ship
end

function Universe:AddFaction(name, type, players)

end

function Universe:AddSystemEconomy(system)
  -- do other stuff here too
  UniverseEconomy:AddSystem(system)
end

return Universe