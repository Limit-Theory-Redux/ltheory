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
  self:addProjectiles()
  self:addEconomy()

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

-- Helpers For Testing ---------------------------------------------------------

local kSystemScale = 10000

function System:spawnAI (shipCount, action, player)
  -- Spawn a number of independent AI-controlled ships
  for i = 1, shipCount do
    local ship = self:spawnShip(player)
    ship:setOwner(player)
    if action then
      ship:pushAction(action)
    end
  end
  player:addItem(Item.Credit, Config.game.eStartCredits)
  player:pushAction(Actions.Think())
  insert(self.players, player)
  return player
end

function System:spawnAsteroidField (count)
  -- Spawn a new asteroid field (a zone containing individual asteroids)
  local rng = self.rng

  -- Give the asteroid field (actually a zone) a name
  local AFieldName = Words.getCoolName(rng)
  local zone = Zone(AFieldName)
  zone:setType(Config:getObjectTypeByName("object_types", "Zone"))
  zone:setSubType(Config:getObjectTypeByName("zone_subtypes", "Asteroid Field"))

  zone.pos = rng:getDir3():scale(0.0 * kSystemScale * (1 + rng:getExp()))

  for i = 1, count do
    -- Spawn a new asteroid
    local pos
    if i == 1 then
      pos = zone.pos
    else
      pos = rng:choose(zone.children):getPos()
      pos = pos + rng:getDir3():scale((0.1 * kSystemScale) * rng:getExp() ^ rng:getExp())
    end

    local scale = 7 * (1 + rng:getExp() ^ 2)
    local asteroid = Objects.Asteroid(rng:get31(), scale)
    asteroid:setType(Config:getObjectTypeByName("object_types", "Asteroid"))
    asteroid:setSubType(Config:getObjectTypeByName("asteroid_subtypes", "Silicaceous"))

    asteroid:setPos(pos)
    asteroid:setScale(scale)
    asteroid:setRot(rng:getQuat())

    -- TODO: Replace with actual system for generating minable materials in asteroids
    if rng:getInt(0, 100) > 70 then
      asteroid:addYield(rng:choose(Item.T2), 1.0)
    end

    -- Give the individual asteroid a name
    local asteroidName = Words.getCoolName(self.rng)
    local namernd = rng:getInt(0, 100)
    if namernd < 60 then
      asteroidName = asteroidName .. " " .. tostring(rng:getInt(11, 99))
    elseif namernd < 85 then
      asteroidName = asteroidName .. " " .. tostring(rng:getInt(101, 999))
    else
      asteroidName = asteroidName .. " " .. tostring(rng:getInt(1001, 9999))
    end
    asteroid:setName(format("%s", asteroidName))
--printf("Added %s '%s'", Config.objectInfo[1]["elems"][asteroid:getType()][2], asteroid:getName())

    zone:add(asteroid)
    self:addChild(asteroid) -- adding each asteroid to both the system and an owning zone (itself not a child of system)
  end

  self:addZone(zone)

local typeName = Config:getObjectInfo("object_types", zone:getType())
local subtypeName = Config:getObjectInfo("zone_subtypes", zone:getSubType())
printf("Added %s - %s '%s'", typeName, subtypeName, zone:getName())

  Config.game.currentZone = zone
  return zone
end

function System:spawnPlanet ()
  -- Spawn a new planet
  local rng = self.rng
  local planet = Objects.Planet(rng:get64())
  planet:setType(Config:getObjectTypeByName("object_types", "Planet"))
  planet:setSubType(Config:getObjectTypeByName("planet_subtypes", "Rocky"))

  local pos = rng:getDir3():scale(kSystemScale * (1.0 + rng:getExp()))
  local scale = 1e5 * rng:getErlang(2)
  planet:setPos(pos)
  planet:setScale(scale)

  -- Planets have significant market capacity
--  planet:setFlow(Item.Silver, self.rng:getUniformRange(-1000, 0)) -- temporary!
  planet:addMarket()
  planet:addTrader()

  -- Planets have significant manufacturing capacity
  local prod = self.rng:choose(Production.All())
  planet:addFactory()
  -- Adding production to a planet previously yielded an error; apparently Josh didn't
  --   think planets could have production facilities... which isn't wrong. Really, it's
  --   _colonies_ on planets that should have production facility children.
  planet:addProduction(prod)

  -- Give the planet a name
  local planetName = Words.getCoolName(self.rng)
  planet:setName(format("%s", planetName))

  self:addChild(planet)

local typeName = Config:getObjectInfo("object_types", planet:getType())
local subtypeName = Config:getObjectSubInfo("object_types", planet:getType(), planet:getSubType())
printf("Added %s (%s) '%s'", typeName, subtypeName, planet:getName())

  Config.game.currentPlanet = planet
  return planet
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

  ship:setInventoryCapacity(Config.game.eInventory)
  ship:setPos(self.rng:getDir3():scale(kSystemScale * (1.0 + self.rng:getExp())))
  self:addChild(ship)

  if true then
    while true do
      local thruster = Ship.Thruster()
      thruster:setScale(0.5 * ship:getScale())
      -- TODO : Does this leak a Thruster/RigidBody?
      if not ship:plug(thruster) then break end
    end
  end
  if true then
    while true do
      local turret = Ship.Turret()
      turret:setScale(2 * ship:getScale())
      -- TODO : Does this leak a Turret/RigidBody?
--printf("ship %s: plug a turret", ship:getName())
      if not ship:plug(turret) then
        break
      end
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

function System:spawnStation (player)
  -- Spawn a new space station
  local station = Objects.Station(self.rng:get31())
  station:setType(Config:getObjectTypeByName("object_types", "Station"))

  local p = self.rng:getDisc():scale(kSystemScale)
  station:setPos(Vec3f(p.x, 0, p.y))
  station:setScale(100)

  -- Stations have market capacity
  --  station:setFlow(Item.Silver, self.rng:getUniformRange(-1000, 0))
  station:addMarket()
  station:addTrader()

  -- Stations have manufacturing capacity
  local prod = self.rng:choose(Production.All())
  station:addFactory()
  station:addProduction(prod)
  station:setSubType(Config:getObjectTypeByName("station_subtypes", prod:getName()))

  -- Give the station a name
  station:setName(Words.getCoolName(self.rng))

  station:setOwner(player)

  self:addChild(station)

local typeName = Config:getObjectInfo("object_types", station:getType())
local subtypeName = Config:getObjectInfo("station_subtypes", station:getSubType())
printf("Added %s %s '%s'", subtypeName, typeName, station:getName())

  return station
end

return System
