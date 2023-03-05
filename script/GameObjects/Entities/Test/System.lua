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

  self.players = {}
  self.zones = {}
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

  -- Randomly place the planet within the system, but not within 200 x or y units of the system origin
  local pos = rng:getDir3():scale(Config.gen.scaleSystem * (1.0 + rng:getExp()))
  pos.x = pos.x + 200
  pos.y = pos.y + 200
  planet:setPos(pos)

  -- Set the planet's scale
  local psbase = Config.gen.scalePlanet
  local psmod = math.floor(Config.gen.scalePlanetMod * math.abs(rng:getGaussian())) -- or rng:getErlang(2)
  local scale = psbase + psmod
printf("planet base size = %d, psmod = %d, scale = %d", psbase, psmod, scale)
  planet:setScale(scale)

  -- Planets have significant market capacity
--  planet:setFlow(Item.Silver, self.rng:getUniformRange(-1000, 0)) -- temporary!
  planet:addMarket()

  -- Planets have enormous trading capacity
  planet:addTrader()
  planet:addCredits(Config.game.eStartCredits * 1000)
  -- Let the planet bid on all items
  -- TODO: add iterating through each item type
  for _, v in pairs(Item.T1) do
    -- TODO: generate better bid price (ask price?); this is just for testing the "payout" model in Think.lua
    planet.trader:addBid(v, rng:getInt(50, 200))
  end
  for _, v in pairs(Item.T2) do
    -- TODO: generate better bid price (ask price?); this is just for testing the "payout" model in Think.lua
    planet.trader:addBid(v, rng:getInt(20, 100))
  end
  for _, v in pairs(Item.T3) do
    -- TODO: generate better bid price (ask price?); this is just for testing the "payout" model in Think.lua
    planet.trader:addBid(v, rng:getInt(150, 350))
  end
  for _, v in pairs(Item.T5) do
    -- TODO: generate better bid price (ask price?); this is just for testing the "payout" model in Think.lua
    planet.trader:addBid(v, rng:getInt(2550, 40000))
  end

  -- Planets have significant manufacturing capacity
  local prod = self.rng:choose(Production.All())
  planet:addFactory()
  -- TODO: Move on-planet production to manufacturing colonies
  planet:addProduction(prod)

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

      local scale = Config.gen.scaleAsteroid
--      local scale = Config.gen.scaleAsteroid * (1.0 + rng:getExp() ^ 2.0)

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
  --   (unless background, in which pick the center of the system)
  if count == 0 then
    zone.pos = Vec3f(200, 0, 200)
    count = 500
  else
    zone.pos = rng:getDir3():scale(1.0 * Config.gen.scaleSystem * (1 + rng:getExp()))
  end

  -- Set the extent (scale) of the asteroid field within a spherical (3D) volume
  zone:setExtent(Config.gen.scaleFieldAsteroid)

  for i = 1, count do
    -- Define the scale (size) of the new asteroid; "reduced" means make small ones only
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
  if rng:getInt(0, 100) > 50 then
    asteroid:addYield(rng:choose(Item.T2), rng:getInt(1, 100))
  end
end

function System:spawnStation (player, fieldPos, fieldExtent)
  local rng = self.rng

  -- Spawn a new space station
  local station = Objects.Station(self.rng:get31())
  station:setType(Config:getObjectTypeByName("object_types", "Station"))

  -- Give the station a name
  station:setName(Words.getCoolName(rng))

  -- Set station location within the extent of a randomly selected asteroid field
  self:place(rng, station)
--  pos = rng:getDisc():scale(Config.gen.scaleSystem) -- old placement style

  -- Set station scale
  station:setScale(Config.gen.scaleStation)

  -- Stations have inventory
  station:setInventoryCapacity(Config.game.eInventory)

  -- Stations have market capacity
  station:addMarket()
  station:setFlow(Item.Silver, self.rng:getUniformRange(-1000, 0))

  -- Stations have trading capacity
  station:addTrader()
  station:addCredits(Config.game.eStartCredits)
  station.trader.credits = Config.game.eStartCredits
  -- Let the station bid on all items that can be mined (T2)
  for _, v in pairs(Item.T2) do
    -- TODO: generate better bid price (ask price?); this is just for testing the "payout" model in Think.lua
    station.trader:addBid(v, 100)
  end

  -- Stations have manufacturing capacity
  local prod = rng:choose(Production.All())
  station:addFactory()
  station:addProduction(prod)
  station:setSubType(Config:getObjectTypeByName("station_subtypes", prod:getName()))

  -- Assign the station to an owner
  station:setOwner(player)

  -- Add the station to this star system
  self:addChild(station)

local typeName = Config:getObjectInfo("object_types", station:getType())
local subtypeName = Config:getObjectInfo("station_subtypes", station:getSubType())
printf("Added %s %s '%s'", subtypeName, typeName, station:getName())

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
  ship:setPos(self.rng:getDir3():scale(Config.gen.scaleSystem * 500.0))
--  ship:setPos(self.rng:getDir3():scale(Config.gen.scaleSystem * (1.0 + self.rng:getExp())))

  ship:setInventoryCapacity(Config.game.eInventory)

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
