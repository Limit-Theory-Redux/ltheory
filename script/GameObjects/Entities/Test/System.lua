local Entity = require('GameObjects.Entity')
local Player = require('GameObjects.Entities.Player')
local Zone = require('GameObjects.Entities.Zone')
local Objects = requireAll('GameObjects.Entities.Objects')
local Ship = requireAll('GameObjects.Entities.Ship')
local Effects = requireAll('GameObjects.Entities.Effects')
local Production = require('Systems.Economy.Production')
local Item = require('Systems.Economy.Item')
local Dust = require('GameObjects.Entities.Effects.Dust')
local Nebula = require('GameObjects.Entities.Objects.Nebula')
local Words = require('Systems.Gen.Words')

local System = subclass(Entity, function (self, seed)
  self.rng = RNG.Create(seed):managed()

  self:setName(Words.getCoolName(self.rng))
  self:setType(Config:getObjectTypeByName("object_types", "Star System"))

printf("Spawning new star system '%s' using seed = %s", self:getName(), seed)

  self:addChildren()

  self:addEconomy()

  self:addProjectiles()

  -- NOTE : For now, we will use a flow component on the system to represent
  --        the summed net flow of all entities in the system. Seems natural,
  --        but should keep an eye on gameplay code to ensure this does not
  --        result in unexpected behavior
  self:addFlows()

  -- TODO : Will physics be freed correctly?
  self.physics = Physics.Create():managed()
  local starAngle = self.rng:getDir2()
  self.starDir = Vec3f(starAngle.x, 0, starAngle.y)
  self.nebula = Nebula(self.rng:get64(), self.starDir)
  self.dust = Dust()

  self.zones    = {}
  self.stations = {}
  self.players  = {}

  -- When creating a new system, initialize station subtype options from all production types
  local prodType = Config:getObjectTypeIndex("station_subtypes")
  for i, prod in ipairs(Production.All()) do
    Config.objectInfo[prodType]["elems"][i+2] = {
      i + 2,
      prod:getName()
    }
  end

end)

function System:addZone (zone)
  insert(self.zones, zone)
end

function System:getZones ()
  return self.zones
end

function System:sampleZones (rng)
  return rng:choose(self.zones)
end

function System:addStation (station)
  insert(self.stations, station)
end

function System:getStations ()
  return self.stations
end

function System:getStationsByDistance (ship)
  local stationList = {}

  for _, station in ipairs(self.stations) do
    local stationStruct = {stationRef = station, stationDist = ship:getDistance(station)}
    insert(stationList, stationStruct)
  end

  table.sort(stationList, function (a, b) return a.stationDist < b.stationDist end)

  return stationList
end

function System:sampleStations (rng)
  return rng:choose(self.stations)
end

function System:place (rng, object)
  -- Set the position of an object to a random location within the extent of a randomly-selected Asteroid Field
  -- TODO: extend this to accept any kind of field, and make this function specific to Asteroid Fields for System
  local pos = Vec3f(0, 0, 0)
  local field = self:sampleZones(rng)
  if field then pos = field:getRandomPos(rng) end
  object:setPos(pos)

  return pos
end

function System:beginRender ()
  self.nebula:forceLoad()
  ShaderVar.PushFloat3('starDir', self.starDir.x, self.starDir.y, self.starDir.z)
  ShaderVar.PushTexCube('envMap', self.nebula.envMap)
  ShaderVar.PushTexCube('irMap', self.nebula.irMap)
end

function System:render (state)
  self:send(Event.Broadcast(state))
  self:renderProjectiles(state)
  self.dust:render(state)
  self.nebula:render(state)
end

function System:endRender ()
  ShaderVar.Pop('starDir')
  ShaderVar.Pop('envMap')
  ShaderVar.Pop('irMap')
end

function System:update (dt)
  if not Config.game.gamePaused then
    -- pre-physics update
    local event = Event.Update(dt)
    Profiler.Begin('AI Update')
    for _, player in ipairs(self.players) do player:send(event) end
    Profiler.End()

    self:send(event)
    Profiler.Begin('Broadcast Update')
    self:send(Event.Broadcast(event))
    Profiler.End()

    Profiler.Begin('Physics Update')
    self.physics:update(dt)
    Profiler.End()

    -- post-physics update
    event = Event.UpdatePost(dt)
    self:send(Event.Broadcast(event))
    self:send(event)
  end
end

---------------------
-- OBJECT CREATION --
---------------------

function System:spawnPlanet (bAddBelt)
  local rng = self.rng

  -- Create the new planet
  local planet = Objects.Planet(rng:get64())
  planet:setType(Config:getObjectTypeByName("object_types", "Planet"))
  planet:setSubType(Config:getObjectTypeByName("planet_subtypes", "Rocky"))

  -- Give the new planet a name
  local planetName = Words.getCoolName(self.rng)
  planet:setName(format("%s", planetName))

  -- Randomly place the planet within the system
  planet:setPos(rng:getDir3():scale(Config.gen.scaleSystem * (1.0 + rng:getExp())))

  -- Set the planet's scale
  local psbase = Config.gen.scalePlanet
  local psmod = math.floor(Config.gen.scalePlanetMod * math.abs(rng:getGaussian())) -- or rng:getErlang(2)
  local scale = psbase + psmod
  planet:setScale(scale)
--printf("planet base size = %d, psmod = %d, scale = %d", psbase, psmod, scale)

  -- Planets produce lots of plants and animals (just go with it)
  planet:addYield(Item.Biomass, rng:getInt(100000, 10000000))

  -- Planets have significant market capacity
  planet:addMarket()
--  planet:setFlow(Item.Silver, self.rng:getUniformRange(-1000, 0)) -- TEMP

  -- Planets have enormous trading capacity
  planet:addTrader()
  planet:addCredits(Config.game.eStartCredits * 1000)

  -- Let the planet bid for selected item types it wants
  -- TODO: generate better bid prices; this is just for testing the "payout" model
  for _, v in pairs(Item.T1) do
    planet.trader:addBid(v, rng:getInt(50, 200)) -- add a bid for a single unit of each item in this group
  end
  for _, v in pairs(Item.T5) do
    planet.trader:addBid(v, rng:getInt(1550, 10000)) -- add a bid for a single unit of each item in this group
  end
  for _, v in pairs(Item.T6) do
    planet.trader:addBid(v, rng:getInt(2450, 48000)) -- add a bid for a single unit of each item in this group
  end

  -- Planets have significant manufacturing capacity
  -- TODO: Move on-planet production to manufacturing colonies
--  planet:addFactory()
--  planet:addProduction(self.rng:choose(Production.All()))

  if bAddBelt then
    -- Add a planetary belt
    -- TODO: GAH! NO! Change this to a planetary rings billboard!
    local center = planet:getPos()
    local rc = 2.00 * planet:getRadius()
    local rw = 0.20 * planet:getRadius()

    for j = 1, Config.gen.nBeltSize(rng) do
      local r = rc + rng:getUniformRange(-rw, rw) * (0.5 + 0.5 * rng:getExp())
      local h = 0.1 * rw * rng:getGaussian()
      local dir = rng:getDir2()

--      local scale = Config.gen.scaleAsteroid
      local scale = Config.gen.scaleAsteroid * (1.0 + rng:getExp() ^ 2.0)

      local asteroid = Objects.Asteroid(rng:get31(), scale)
      asteroid:setType(Config:getObjectTypeByName("object_types", "Asteroid"))
      asteroid:setSubType(Config:getObjectTypeByName("asteroid_subtypes", "Silicaceous"))

      -- Give the individual asteroid a name
      local asteroidName = System:getAsteroidName(self, rng)
      asteroid:setName(format("%s", asteroidName))

      -- Possibly give the new asteroid minable Yield
      System:setAsteroidYield(rng, asteroid)

      -- Place the new asteroid in a torus around the planet
      asteroid:setPos(center + Vec3f(r * dir.x, h, r * dir.y))

      -- Let the new asteroid have a random angle
      asteroid:setRot(rng:getQuat())

      self:addChild(asteroid)
    end
  end

  self:addChild(planet)

local typeName = Config:getObjectInfo("object_types", planet:getType())
local subtypeName = Config:getObjectSubInfo("object_types", planet:getType(), planet:getSubType())
printf("Added %s (%s) '%s'", typeName, subtypeName, planet:getName())

  Config.game.currentPlanet = planet
  return planet
end

function System:spawnAsteroidField (count, reduced)
  -- Spawn a new asteroid field (a zone containing individual asteroids)
  local rng = self.rng

  -- Create the asteroid field (actually a zone)
  local AFieldName = Words.getCoolName(rng)

  -- Give the new asteroid field a name
  local zone = Zone(AFieldName)
  zone:setType(Config:getObjectTypeByName("object_types", "Zone"))
  zone:setSubType(Config:getObjectTypeByName("zone_subtypes", "Asteroid Field"))

  -- Pick a random location in the system for the center of the asteroid field
  --   (unless background, in which case pick the center of the system)
  -- If count is -1, that's the signal to create a field for background mode
  if count == -1 then
    zone.pos = Vec3f(200, 0, 200)
    count = 500
  else
    zone.pos = rng:getDir3():scale(1.0 * Config.gen.scaleSystem * (1 + rng:getExp()))
  end

  -- Set the extent (scale) of the asteroid field within a spherical (3D) volume
  zone:setExtent(Config.gen.scaleFieldAsteroid)

  for i = 1, count do
    -- Define the actual scale (size) of the new asteroid; "reduced" means make small ones only
    local scale = 7 * (1 + rng:getExp() ^ 3)
    if reduced then
      scale = 7 * (1 + rng:getExp())
    end

    -- Create the new asteroid
    local asteroid = Objects.Asteroid(rng:get31(), scale)
    asteroid:setType(Config:getObjectTypeByName("object_types", "Asteroid"))
    asteroid:setSubType(Config:getObjectTypeByName("asteroid_subtypes", "Silicaceous"))

    -- Give the individual asteroid a name
    local asteroidName = System:getAsteroidName(self, rng)
    asteroid:setName(format("%s", asteroidName))
--printf("Added %s '%s'", Config.objectInfo[1]["elems"][asteroid:getType()][2], asteroid:getName())

    -- Actually set the scale of the new asteroid
    asteroid:setScale(scale)

    -- Set asteroid position
    local pos
    if i == 1 then
      pos = zone.pos
    else
      -- We place this asteroid directly, rather than using self:place(rng, asteroid) for randomness,
      --   because we want it to go into the area around this AsteroidField (a Zone) we just created
      pos = zone.pos + rng:getDir3():scale((0.1 * zone:getExtent()) * rng:getExp() ^ rng:getExp())
    end
    asteroid:setPos(pos)

    -- Randomly rotate the asteroid from the vertical axis
    asteroid:setRot(rng:getQuat())

    -- TODO: Replace with actual system for generating minable materials in asteroids
    System:setAsteroidYield(rng, asteroid)

    -- Asteroids are added both to this new AsteroidField (Zone) and as a child of this System
    -- TODO: add asteroids only to Zones, and let Systems iterate through zones for child objects to render
    zone:add(asteroid)
    self:addChild(asteroid)
  end

  self:addZone(zone)

local typeName = Config:getObjectInfo("object_types", zone:getType())
local subtypeName = Config:getObjectInfo("zone_subtypes", zone:getSubType())
printf("Added %s - %s '%s'", typeName, subtypeName, zone:getName())

  return zone
end

function System:getAsteroidName (self, rng)
  local aName = Words.getCoolName(self.rng)
  local namernd = rng:getInt(0, 100)
  if namernd < 60 then
    aName = aName .. " " .. tostring(rng:getInt(11, 99))
  elseif namernd < 85 then
    aName = aName .. " " .. tostring(rng:getInt(101, 999))
  else
    aName = aName .. " " .. tostring(rng:getInt(1001, 9999))
  end

  return aName
end

function System:setAsteroidYield (rng, asteroid)
  -- TODO: Replace with actual system for generating minable materials in asteroids
  -- Start with a 70% chance that an asteroid will have any yield at all
  if rng:getInt(0, 100) < 70 then
    local amass = math.floor(asteroid:getMass() / 1000)
    local itemT2 = Item.T2
    table.sort(itemT2, function (a, b) return a.distribution < b.distribution end)
    local itemType = nil
    local ichance = 0.0
    local uval = rng:getUniformRange(0.00, 1.00)
    for i, item in ipairs(itemT2) do
      ichance = ichance + item.distribution
      if uval < ichance then -- pick a minable material based on distribution %
        itemType = item
        break
      end
    end
    if itemType == nil then
      itemType = Item.Silicates
    end
    asteroid:addYield(itemType, math.max(1, rng:getInt(amass / 2, amass)))
  end
end

function System:spawnStation (player, prodType)
  local rng = self.rng

  -- Spawn a new space station
  local station = Objects.Station(self.rng:get31())
  station:setType(Config:getObjectTypeByName("object_types", "Station"))

  -- Give the station a name
  station:setName(Words.getCoolName(rng))

  -- Set station location within the extent of a randomly selected asteroid field
  self:place(rng, station)

  -- Set station scale
  station:setScale(Config.gen.scaleStation)

  -- Stations have market capacity
  station:addMarket()
  for _, v in pairs(Item.T2) do
    -- TODO: generate better bid price; this is just for testing the "payout" model in Think.lua
    local flowval = self.rng:getUniformRange(-1000, 0)
    station:setFlow(v, flowval) -- TEMP
--printf("Station %s: adding flow for item %s at value %d", station:getName(), v:getName(), flowval)
  end

  -- Stations have trading capacity
  station:addTrader()

  -- Stations have manufacturing capacity for one randomly-chosen production type
  -- TODO: Assign a station's production type based on system needs (e.g., insure there's
  --       always at least one energy-generating station in each system)
  station:addFactory()
  local prod = prodType
  if not prodType then
    -- No specific production type provided, so pick one randomly
    local rint = rng:getInt(0, 100)
    if rint > 80 then
      prod = rng:choose(Production.P0) -- small chance for a powerplant
    elseif rint > 20 then
      prod = rng:choose(Production.P1) -- good chance for a refinery
    else
      prod = rng:choose(Production.P2) -- small chance for a factory
--      prod = rng:choose(Production.All()) -- if no production type is provided, choose anything randomly
    end
  end
  station:addProduction(prod)
  station:setSubType(Config:getObjectTypeByName("station_subtypes", prod:getName()))

  -- Station starts with some credits and some energy (energy Item must exist before bid is offered!)
  station:addCredits(Config.game.eStartCredits * 100)

  -- The station sets asks for selling items its facility produces as outputs,
  --     and sets bids for buying items its facility wants as inputs
  -- Ask prices (for selling) are calculated as the item's base price times a markup value
  -- Bid prices (for buying) are calculated as the item's base price times a markdown value
  for _, input in prod:iterInputs() do
    for i = 1, input.count * Config.game.inputBacklog do
      -- TODO: Change magic number 33 for "I want..." bids to a multiplier connected to this system's flows
      station.trader:addBid(input.item, math.max(1, math.floor(input.item.energy * Config.econ.markdown * 33)))
    end
  end
  for _, output in prod:iterOutputs () do
    for i = 1, output.count do
      station.trader:addAsk(output.item, math.floor(output.item.energy * Config.econ.markup))
    end
  end

  -- Assign the station to an owner
  station:setOwner(player)

  -- Add the station to this star system
  self:addChild(station)

local typeName = Config:getObjectInfo("object_types", station:getType())
local subtypeName = Config:getObjectInfo("station_subtypes", station:getSubType())
printf("Added %s %s '%s' (production = %s)", subtypeName, typeName, station:getName(), prod:getName())

  self:addStation(station)

  return station
end

function System:spawnAI (shipCount, action, player)
  -- Spawn a number of independent AI-controlled ships
  for i = 1, shipCount do
    local ship = self:spawnShip(player)
    ship:setOwner(player)
    if action then
      ship:pushAction(action)
    end
  end
  insert(self.players, player)
  return
end

function System:spawnShip (player)
  -- Spawn a new ship (with a new ship type)
  if Config.ui.uniqueShips or not self.shipType then
    self.shipType = Ship.ShipType(self.rng:get31(), Gen.Ship.ShipFighter, 4)
  end
  local ship = self.shipType:instantiate()
  ship:setType(Config:getObjectTypeByName("object_types", "Ship"))
  ship:setSubType(Config:getObjectTypeByName("ship_subtypes", "Fighter"))

  -- Give the ship a name
  local shipName = Words.getCoolName(self.rng)
  ship:setName(format("HSS %s", shipName))

  -- Give the ship a player owner if one was not provided
  local shipPlayer = nil
  if player ~= nil then
    shipPlayer = player
  else
    shipPlayer = Player(format("Ship Player for %s", ship:getName()))
    insert(self.players, shipPlayer)
  end
  ship:setOwner(shipPlayer)

  -- TODO: make sure spawn position for player ship is well outside any planetary volume
--  ship:setPos(self.rng:getDir3():scale(Config.gen.scaleSystem * 500.0))
  ship:setPos(self.rng:getDir3():scale(Config.gen.scaleSystem * (1.0 + self.rng:getExp())))

  -- TODO: replace Config.game.eInventory with actual cargo hold capacity based on ship role plug assignments
  ship:setInventoryCapacity(Config.game.eInventory)

  -- NOTE: a new ship must be added to a star system BEFORE thrusters and turrets are attached!
  self:addChild(ship)

  -- Add as many thrusters as there are thruster plugs for
  while true do
    local thruster = Ship.Thruster()
    thruster:setScale(0.5 * ship:getScale())
    -- TODO : Does this leak a Thruster/RigidBody?
--printf("ship %s: plug a thruster", ship:getName())
    if not ship:plug(thruster) then break end
  end

  -- Add as many turrets as there are turret plugs for
  while true do
    local turret = Ship.Turret()
    turret:setScale(2 * ship:getScale())
    -- TODO : Does this leak a Turret/RigidBody?
--printf("ship %s: plug a turret", ship:getName())
    if not ship:plug(turret) then
      break
    end
  end

--local subtypeName = Config:getObjectInfo("ship_subtypes", ship:getSubType())
--printf("Added Ship (%s) '%s'", subtypeName, ship:getName())

  return ship
end

function System:spawnBackground ()
  -- For a star system background only (no ship), spawn an invisible ship
  --   (because System.lua needs a thing with mass, scale, drag, and thrust
  --   in order to rotate around a camera viewpoint)
  if not self.shipType then
    self.shipType = Ship.ShipType(self.rng:get31(), Gen.Ship.ShipInvisible, 4)
  end
  local background = self.shipType:instantiate()

  local player = Player("Background Player")
  background:setOwner(player)

  self:addChild(background)

  return background
end

return System
