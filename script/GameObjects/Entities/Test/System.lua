local Entity = require('GameObjects.Entity')
local Player = require('GameObjects.Entities.Player')
local Zone = require('GameObjects.Entities.Zone')
local Objects = requireAll('GameObjects.Entities.Objects')
local Components = requireAll('GameObjects.Elements.Components')
local Ship = requireAll('GameObjects.Entities.Ship')
local Effects = requireAll('GameObjects.Entities.Effects')
local Production = require('Systems.Economy.Production')
local Item = require('Systems.Economy.Item')
local Dust = require('GameObjects.Entities.Effects.Dust')
local Nebula = require('GameObjects.Entities.Objects.Nebula')
local Words = require('Systems.Gen.Words')
local HUD = require('Systems.Overlay.HUD')

local System = subclass(Entity, function(self, seed)
    self.rng = RNG.Create(seed):managed()

    self:setName(Words.getCoolName(self.rng))
    self:setType(Config:getObjectTypeByName("object_types", "Star System"))

    Log.Debug("Spawning new star system '%s' using seed = %s", self:getName(), seed)

    self:addChildren()

    self:addEconomy()

    self:addProjectiles()

    -- NOTE : For now, we will use a flow component on the system to represent
    -- the summed net flow of all entities in the system. Seems natural,
    -- but should keep an eye on gameplay code to ensure this does not
    -- result in unexpected behavior
    self:addFlows()

    -- TODO : Will physics be freed correctly?
    self.physics         = Physics.Create():managed()
    local starAngle      = self.rng:getDir2()
    self.starDir         = Vec3f(starAngle.x, 0, starAngle.y)
    self.nebula          = Nebula(self.rng:get64(), self.starDir)
    self.dust            = Dust()

    self.players         = {}
    self.aiPlayers       = nil
    self.stars           = {}
    self.planets         = {}
    self.zones           = {}
    self.stations        = {}
    self.ships           = {}
    self.lightList       = {}

    -- When creating a new system, initialize station subtype options from all production types
    local prodType       = Config:getObjectTypeIndex("station_subtypes")
    local originalLength = #Config.objectInfo[prodType]["elems"]

    for i, prod in ipairs(Production.All()) do
        Config.objectInfo[prodType]["elems"][i + originalLength] =
        {
            i + originalLength,
            prod:getName()
        }
    end
end)

function System:addStar(star)
    insert(self.stars, star)
end

function System:getStars()
    return self.stars
end

function System:addPlanet(planet)
    insert(self.planets, planet)
end

function System:getPlanets()
    return self.planets
end

function System:addZone(zone)
    insert(self.zones, zone)
end

function System:getZones()
    return self.zones
end

function System:sampleZones(rng)
    return rng:choose(self.zones)
end

function System:addStation(station)
    insert(self.stations, station)
end

function System:getStations()
    return self.stations
end

function System:getStationsByDistance(ship)
    -- Return a table of stations sorted by nearest first
    local stationList = {}
    for _, station in ipairs(self.stations) do
        local stationStruct = { stationRef = station, stationDist = ship:getDistance(station) }
        if station:hasDockable() and station:isDockable() and not station:isBanned(ship) then
            insert(stationList, stationStruct)
        end
    end

    table.sort(stationList, function(a, b) return a.stationDist < b.stationDist end)

    return stationList
end

function System:sampleStations(rng)
    return rng:choose(self.stations)
end

function System:addShip(ship)
    insert(self.ships, ship)
end

function System:getShips()
    return self.ships
end

function System:hasProdType(prodtype)
    -- Scan the production types of all factories in this system to see if one has the specified production type
    local hasProdType = false
    for _, station in ipairs(self.stations) do
        if station:hasFactory() then
            if station:getFactory():hasProductionType(prodtype) then
                hasProdType = true
                break
            end
        end
    end

    return hasProdType
end

function System:countProdType(prodtype)
    -- Scan the production types of all factories in this system to see how many of the specified production type exist
    local numProdType = 0
    for _, station in ipairs(self.stations) do
        if station:hasFactory() then
            if station:getFactory():hasProductionType(prodtype) then
                numProdType = numProdType + 1
            end
        end
    end

    return numProdType
end

function System:addExtraFactories(system, planetCount, aiPlayer)
    -- Based on what factories were added randomly to stations, a system may need some
    -- additional factories to provide the necessary Input items
    if Config.gen.nEconNPCs > 0 then
        local newStation = nil
        local prodTypeCount = 0

        prodTypeCount = prodTypeCount + system:countProdType(Production.Silver)
        prodTypeCount = prodTypeCount + system:countProdType(Production.Gold)
        prodTypeCount = prodTypeCount + system:countProdType(Production.Platinum)
        for i = 1, prodTypeCount do
            -- Add a Copper Refinery station (to create Item.AnodeSludge)
            newStation = system:spawnStation(Enums.StationHulls.Small, aiPlayer, Production.Copper)
            newStation.zone = system:place(newStation)
        end

        prodTypeCount = system:countProdType(Production.EnergyNuclear)
        for i = 1, prodTypeCount do
            -- Add an Isotope Factory station (to create Item.Isotopes)
            newStation = system:spawnStation(Enums.StationHulls.Small, aiPlayer, Production.Isotopes)
            newStation.zone = system:place(newStation)
        end

        prodTypeCount = system:countProdType(Production.Isotopes)
        for i = 1, prodTypeCount do
            -- Add a Thorium Refinery station (to create Item.Thorium)
            newStation = system:spawnStation(Enums.StationHulls.Small, aiPlayer, Production.Thorium)
            newStation.zone = system:place(newStation)
        end

        prodTypeCount = system:countProdType(Production.EnergyFusion)
        for i = 1, prodTypeCount do
            -- Add 2 Water Melter stations (to create Item.WaterLiquid)
            newStation = system:spawnStation(Enums.StationHulls.Small, aiPlayer, Production.WaterMelter)
            newStation.zone = system:place(newStation)
            newStation = system:spawnStation(Enums.StationHulls.Small, aiPlayer, Production.WaterMelter)
            newStation.zone = system:place(newStation)
        end

        for i = 1, planetCount do
            -- Add a Petroleum Refinery station
            -- TODO: only add refineries for each planet that has a Trader
            newStation = system:spawnStation(Enums.StationHulls.Small, aiPlayer, Production.Petroleum)
            newStation.zone = system:place(newStation)
        end

        prodTypeCount = system:countProdType(Production.Petroleum)
        for i = 1, prodTypeCount do
            -- Add a Plastics Factory station (to create Item.Plastic)
            newStation = system:spawnStation(Enums.StationHulls.Small, aiPlayer, Production.WaterMelter)
            newStation.zone = system:place(newStation)
        end
    end
end

function System:place(object, spawnOutOfAsteroidZone)
    -- Set the position of an object to a random location within the extent of a randomly-selected Asteroid Field
    -- TODO: extend this to accept any kind of field, and make this function specific to Asteroid Fields for System
    local typeName = Config:getObjectInfo("object_types", object:getType())

    local pos = Config.gen.origin
    local field = self:sampleZones(self.rng)
    local counter = 1

    if field and not spawnOutOfAsteroidZone then
        pos = field:getRandomPos(self.rng) -- place new object within a random field
        -- Stations
        if typeName == "Station" then
            -- TODO: inefficient way of doing this. replace later.
            local validSpawn = false
            while not validSpawn do
                local stations = self.stations

                local function checkDistanceToAllStations(pos)
                    for _, station in ipairs(stations) do
                        if pos:distance(station:getPos()) < Config.gen.stationMinimumDistance then
                            Log.Debug("New Station closer than " ..
                                Config.gen.stationMinimumDistance .. " (" ..
                                math.floor(pos:distance(station:getPos())) ..
                                ") to station: '" .. station:getName() .. "'. Finding New Position.")
                            return false
                        end
                    end
                    return true
                end

                local function checkIfInSystem(pos)
                    if Config.gen.scaleSystem < 5e4 then
                        local distanceFromOrigin = pos:distance(Config.gen.origin)
                        -- TODO: replace later with actual system size
                        if distanceFromOrigin > 200000 then
                            Log.Debug("New Station too far away from system core: " ..
                                math.floor(distanceFromOrigin) .. ". Finding New Position.")
                            return false
                        end
                        return true
                    end
                end

                do
                    if counter >= Config.gen.minimumDistancePlacementMaxTries then
                        Log.Debug("Exceeded max placement tries, placing at last random position: %s", pos)
                        validSpawn = true
                    elseif not checkIfInSystem(pos) or not checkDistanceToAllStations(pos) then
                        pos = field:getRandomPos(self.rng)
                        counter = counter + 1
                    else
                        Log.Debug("Found Position to Spawn: %s", pos)
                        validSpawn = true
                    end
                end
            end
        end

        -- Ships
        if typeName == "Ship" then

        end
    elseif spawnOutOfAsteroidZone then
        local minPirateStationSpawnPositionScale = math.floor(Config.gen.scaleSystem * 0.9)
        pos = Vec3f(self.rng:getInt(minPirateStationSpawnPositionScale, Config.gen.scaleSystem), 0,
            self.rng:getInt(minPirateStationSpawnPositionScale, Config.gen.scaleSystem)) -- place new object _near_ the origin
    else
        pos = Vec3f(self.rng:getInt(5000, 8000), 0, self.rng:getInt(5000, 8000))         -- place new object _near_ the origin
    end
    object:setPos(pos)
    -- Return the Asteroid Field zone in which the object is being placed
    return field
end

function System:beginRender()
    self.nebula:forceLoad()
    ShaderVar.PushFloat3('starDir', self.starDir.x, self.starDir.y, self.starDir.z)
    ShaderVar.PushTexCube('envMap', self.nebula.envMap)
    ShaderVar.PushTexCube('irMap', self.nebula.irMap)
end

function System:render(state)
    self:send(Event.Broadcast(state))
    self:renderProjectiles(state)
    self.dust:render(state)
    self.nebula:render(state)
end

function System:endRender()
    ShaderVar.Pop('starDir')
    ShaderVar.Pop('envMap')
    ShaderVar.Pop('irMap')
end

function System:update(dt)
    if not GameState.paused then
        -- pre-physics update
        local event = Event.Update(dt)
        Profiler.Begin('AI Update')
        if self.aiPlayers and #self.aiPlayers > 0 then
            for _, player in ipairs(self.aiPlayers) do player:send(event) end
        end
        for _, player in ipairs(self.players) do player:send(event) end
        Profiler.End()

        -- self:send(event) -- unnecessary extra event?
        Profiler.Begin('Broadcast Update')
        self:send(Event.Broadcast(event))
        Profiler.End()

        Profiler.Begin('Physics Update')
        self.physics:update(dt)
        local collision = Collision()
        while (self.physics:getNextCollision(collision)) do
            local entity1 = Entity.fromRigidBody(collision.body0)
            local entity2 = Entity.fromRigidBody(collision.body1)

            if entity1 and entity2 then
                entity1:send(Event.Collision(collision, entity2))
                entity2:send(Event.Collision(collision, entity1))
            end
            --print('', collision.index, collision.body0, collision.body1)
        end
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

function System:spawnPlanet(bAddBelt)
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
    --Log.Debug("planet base size = %d, psmod = %d, scale = %d", psbase, psmod, scale)

    self:addChild(planet)

    -- Add all applicable components to this planet
    -- TODO: For now, every socket gets one of the appropriate components. Later, this must be replaced by:
    -- 1) default components (for planets created when a new star system is generated)
    -- 2) loaded components (for planets recreated when the player loads a saved game)
    -- NOTE: Components must be instantiated AFTER their parent is added as a child to the star system!

    -- Add as many communicators as there are communicator plugs for
    for i = 1, planet.countCommo do
        local commo = Components.Communicator()
        commo:setName(planet:getName() .. ": " .. "Morse's " .. commo:getName() .. " " .. tostring(i))
        insert(planet.components.commo, commo)
        planet:plug(commo)
    end

    -- Add as many computers as there are computer plugs for
    for i = 1, planet.countComputer do
        local computer = Components.Computer()
        computer:setName(planet:getName() .. ": " .. "Babbage's " .. computer:getName() .. " " .. tostring(i))
        insert(planet.components.computer, computer)
        planet:plug(computer)
    end

    -- Add as many sensors as there are sensor plugs for
    for i = 1, planet.countSensor do
        local sensor = Components.Sensor()
        sensor:setName(planet:getName() .. ": " .. "April's " .. sensor:getName() .. " " .. tostring(i))
        insert(planet.components.sensor, sensor)
        -- planet:plug(sensor)
    end

    -- Add transport pods to every inventory plug
    for i = 1, planet.countInventory do
        local inventory = Components.Inventory()
        inventory:setName(planet:getName() ..
            ": " .. inventory:getName() .. "(" .. Config.gen.planetInventorySize .. ") " .. tostring(i))
        inventory:setInventoryCapacity(Config.gen.planetInventorySize)
        insert(planet.components.inventory, inventory)
        planet:plug(inventory)
    end

    -- Add as many shield generators as there are shield plugs for
    for i = 1, planet.countShield do
        local shield = Components.Shield()
        shield:setName(planet:getName() .. ": " .. "Magma " .. shield:getName() .. " " .. tostring(i))
        local shieldStrength = shield:getStrengthMax() * 20 -- planetary shields are stronger than station shields
        shield:setStrength(shieldStrength, shieldStrength, shield:getReviveRate() * 2)
        insert(planet.components.shield, shield)
        planet:plug(shield)
    end

    -- Planets produce lots of plants and animals (just go with it)
    planet:addYield(Item.Biomass, rng:getInt(100000, 10000000))

    -- Planets have significant market capacity
    planet:addMarket()
    -- planet:setFlow(Item.Silver, self.rng:getUniformRange(-1000, 0)) -- TEMP

    -- Planets have enormous trading capacity
    planet:addTrader()
    planet:addCredits(Config.econ.eStartCredits * 1000)

    -- Let the planet bid for selected item types it wants
    -- TODO: generate better bid prices; this is just for testing the "payout" model
    local price = 0    -- base price
    local dprice = 0   -- desire price
    local bidCount = 0 -- number of bids to offer
    -- NOTE: bid prices are being generated higher than all of the ask prices for these items when they're
    -- produced. This is temporary to insure there's always a profit in trading factory-produced goods.
    -- TODO: generate prices based on the item's "energy," but enable random "high demand" bids and/or
    -- locally higher-than-normal bid prices.
    -- TODO: Add AI to station/trader/factory owners to let them set the prices for their bids and asks (bidding wars!)
    for _, v in pairs(Item.T1) do
        bidCount = rng:getInt(1000, 10000)
        for i = 1, bidCount do
            dprice = v.energy * Config.econ.markup * 2
            price = dprice + rng:getInt(math.max(1, math.floor(dprice / 10)), math.max(1, math.floor(dprice / 2)))
            planet.trader:addBid(v, price) -- add a bid for a single unit of each item in the Information group
        end
    end
    for _, v in pairs(Item.T5) do
        bidCount = rng:getInt(300, 2000)
        for i = 1, bidCount do
            dprice = v.energy * Config.econ.markup * 2
            price = dprice + rng:getInt(math.max(1, math.floor(dprice / 10)), math.max(1, math.floor(dprice / 2)))
            planet.trader:addBid(v, price) -- add a bid for a single unit of each item in the General Products group
        end
    end
    for _, v in pairs(Item.T6) do
        bidCount = rng:getInt(150, 800)
        for i = 1, bidCount do
            dprice = v.energy * Config.econ.markup * 2
            price = dprice + rng:getInt(math.max(1, math.floor(dprice / 10)), math.max(1, math.floor(dprice / 2)))
            planet.trader:addBid(v, price) -- add a bid for a single unit of each item in the General Products group
        end
    end

    -- Planets have significant manufacturing capacity
    -- TODO: Move on-planet production to manufacturing colonies
    -- planet:addFactory()
    -- planet:addProduction(self.rng:choose(Production.All()))

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

    self:addPlanet(planet)

    local typeName = Config:getObjectInfo("object_types", planet:getType())
    local subtypeName = Config:getObjectSubInfo("object_types", planet:getType(), planet:getSubType())
    Log.Debug("Added %s (%s) '%s'", typeName, subtypeName, planet:getName())

    return planet
end

function System:spawnAsteroidField(count, reduced)
    -- Spawn a new asteroid field (a zone containing individual asteroids)
    local rng = self.rng

    -- Create the asteroid field (actually a zone)
    local AFieldName = Words.getCoolName(rng)

    -- Give the new asteroid field a name
    local zone = Zone(AFieldName)
    zone:setType(Config:getObjectTypeByName("object_types", "Zone"))
    zone:setSubType(Config:getObjectTypeByName("zone_subtypes", "Asteroid Field"))

    -- Pick a random location in the system for the center of the asteroid field
    -- (unless background, in which case pick the center of the system)
    -- If count is -1, that's the signal to create a field for background mode
    if count == -1 then
        zone.pos = Vec3f(200, 0, 200)
        count = 500
    else
        zone.pos = rng:getDir3():scale(1.0 * Config.gen.scaleSystem * (2 + rng:getExp()))
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
        --Log.Debug("Added %s '%s'", Config.objectInfo[1]["elems"][asteroid:getType()][2], asteroid:getName())

        -- Actually set the scale of the new asteroid
        asteroid:setScale(scale)

        -- Set asteroid position
        local pos
        if i == 1 then
            pos = zone.pos -- place first object at zone's center (for non-asteroid field zones)
        else
            -- We place this asteroid directly, rather than using self:place(asteroid) for randomness,
            -- because we want it to go into the area around this AsteroidField (a Zone) we just created
            pos = zone.pos + rng:getDir3():scale((0.1 * zone:getExtent()) * rng:getExp() ^ rng:getExp())
            if Config.gen.scaleSystem < 5e4 then
                while pos:distance(Config.gen.origin) > 200000 do -- constrain max extent of small star systems for performance
                    pos = zone.pos + rng:getDir3():scale((0.1 * zone:getExtent()) * rng:getExp() ^ rng:getExp())
                end
            end
        end
        asteroid:setPos(pos)

        -- Randomly rotate the asteroid from the vertical axis
        asteroid:setRot(rng:getQuat())

        -- TODO: Replace with actual system for generating minable materials in asteroids
        System:setAsteroidYield(rng, asteroid)

        -- Asteroids are added both to this new AsteroidField (Zone) and as a child of this System
        -- TODO: add asteroids only to Zones, and let Systems iterate through zones for child objects to render
        zone:addChild(asteroid)
        asteroid.zone = zone
        self:addChild(asteroid)
    end

    self:addZone(zone)
    -- TODO: Event update should be sent to zones and their children aswell instead of only the system
    self:addChild(zone)

    local typeName = Config:getObjectInfo("object_types", zone:getType())
    local subtypeName = Config:getObjectInfo("zone_subtypes", zone:getSubType())
    Log.Debug("Added %s - %s '%s'", typeName, subtypeName, zone:getName())

    return zone
end

function System:getAsteroidName(self, rng)
    local aNum = ""
    local namernd = rng:getInt(0, 100)
    if namernd < 60 then
        aNum = tostring(rng:getInt(11, 99)) .. " "
    elseif namernd < 85 then
        aNum = tostring(rng:getInt(101, 999)) .. " "
    else
        aNum = tostring(rng:getInt(1001, 9999)) .. " "
    end

    return (aNum .. Words.getCoolName(self.rng))
end

function System:setAsteroidYield(rng, asteroid)
    -- TODO: Replace with actual system for generating minable materials in asteroids
    -- Start with a 70% chance that an asteroid will have any yield at all
    if rng:getInt(0, 100) < 70 then
        local amass = math.floor(asteroid:getMass() / 1000)
        local itemT2 = Item.T2
        table.sort(itemT2, function(a, b) return a.distribution < b.distribution end)
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
        asteroid:addYield(itemType, math.max(1, math.floor(rng:getUniformRange(amass / 2, amass))))
    end
end

local function addStationComponents(station, hullSize)
    -- Add all components to this station
    -- TODO: For now, every socket gets one of the appropriate components. Later, this must be replaced by:
    -- 1) default components (for stations magically spawned when a new star system is generated)
    -- 2) no components (for stations newly built at a factory)
    -- 3) loaded components (for stations recreated when the player loads a saved game)
    -- NOTE: Components must be instantiated AFTER their parent is added as a child to the star system!

    -- Add as many armor plates as there are armor plugs for
    -- station:addArmor(Config.gen.compArmorStats.healthMax * station.countArmor) -- TEMP
    for i = 1, station.countArmor do
        local armor = Components.Armor()
        armor:setName(station:getName() .. ": " .. "Glassiron " .. armor:getName() .. " " .. tostring(i))
        insert(station.components.armor, armor)
        -- station:plug(armor)
    end

    -- Add as many bays as there are bay plugs for
    for i = 1, station.countBay do
        local bay = Ship.Bay()
        bay.name = station:getName() .. ": " .. bay.name .. " " .. tostring(i)
        insert(station.components.bay, bay)
        -- TODO : Does this leak a Bay/RigidBody?
        station:plug(bay)
    end

    -- Add as many capacitors as there are capacitor plugs for
    for i = 1, station.countCapacitor do
        local capacitor = Components.Capacitor()
        capacitor:setName(station:getName() .. ": " .. "Cauchy's " .. capacitor:getName() .. " " .. tostring(i))
        insert(station.components.capacitor, capacitor)
        station:plug(capacitor)
    end

    -- Add as many communicators as there are communicator plugs for
    for i = 1, station.countCommo do
        local commo = Components.Communicator()
        commo:setName(station:getName() .. ": " .. "Morse's " .. commo:getName() .. " " .. tostring(i))
        insert(station.components.commo, commo)
        station:plug(commo)
    end

    -- Add as many computers as there are computer plugs for
    for i = 1, station.countComputer do
        local computer = Components.Computer()
        computer:setName(station:getName() .. ": " .. "Babbage's " .. computer:getName() .. " " .. tostring(i))
        insert(station.components.computer, computer)
        station:plug(computer)
    end

    -- Add as many drone racks as there are drone plugs for
    for i = 1, station.countDrone do
        local drone = Ship.Drone()
        drone:setName(station:getName() .. ": " .. "Wilson's " .. drone:getName() .. " " .. tostring(i))
        insert(station.components.drone, drone)
        -- station:plug(drone)
    end

    -- Add a Hull component as a unitary object
    local hull = Components.Hull()
    hull:setName(station:getName() .. ": " .. "Superdense " .. hull:getName())
    local hullHealth = hull:getHealth() * Config.gen.stationComponents[Enums.StationComponents.Hull][hullSize]
    hull:setHealth(hullHealth, hullHealth)
    insert(station.components.hull, hull)
    -- station:plug(hull)

    -- Add transport pods to every inventory plug
    for i = 1, station.countInventory do
        local inventory = Components.Inventory()
        inventory:setName(station:getName() ..
            ": " .. inventory:getName() .. "(" .. Config.gen.stationInventorySize .. ") " .. tostring(i))
        inventory:setInventoryCapacity(Config.gen.stationInventorySize)
        insert(station.components.inventory, inventory)
        station:plug(inventory)
    end
    if station.countInventory > 0 then
        --Log.Debug("SYSTEM(station): registering Inventory Event.Debug, handler = %s", Entity.mgrInventoryDebug)
        station:register(Event.Debug, Entity.mgrInventoryDebug)
    end

    -- Add as many sensors as there are sensor plugs for
    for i = 1, station.countSensor do
        local sensor = Components.Sensor()
        sensor:setName(station:getName() .. ": " .. "April's " .. sensor:getName() .. " " .. tostring(i))
        insert(station.components.sensor, sensor)
        -- station:plug(sensor)
    end

    -- Add as many shield generators as there are shield plugs for
    for i = 1, station.countShield do
        local shield = Components.Shield()
        shield:setName(station:getName() .. ": " .. "Magma " .. shield:getName() .. " " .. tostring(i))
        local shieldStrength = shield:getStrengthMax() * 2 -- station shields are stronger than ship shields
        shield:setStrength(shieldStrength, shieldStrength, shield:getReviveRate() * 2)
        insert(station.components.shield, shield)
        station:plug(shield)
    end

    -- Add as many turrets as there are turret plugs for
    for i = 1, station.countTurret do
        local turret = Ship.Turret()
        turret.name = station:getName() .. ": " .. turret.name .. " " .. tostring(i)
        turret:setScale(0.6 * station:getScale())
        insert(station.components.turret, turret)
        -- TODO : Does this leak a Turret/RigidBody?
        station:plug(turret)
    end
end

function System:spawnStation(hullSize, player, prodType)
    local rng = self.rng

    -- Spawn a new space station
    local station = Objects.Station(self.rng:get31(), hullSize)
    station:setType(Config:getObjectTypeByName("object_types", "Station"))

    -- Give the station a name
    station:setName(Words.getCoolName(rng))

    -- Set station location within the extent of a randomly selected asteroid field
    station.zone = self:place(station)

    -- Assign the station to an owner
    station:setOwner(player)

    -- Stations have market capacity
    station:addMarket()
    for _, v in pairs(Item.T2) do
        -- TODO: generate better bid price; this is just for testing the flow-based "payout" model in Think.lua
        local flowval = self.rng:getUniformRange(-1000, 0)
        station:setFlow(v, flowval) -- TEMP
        --Log.Debug("Station %s: adding flow for item %s at value %d", station:getName(), v:getName(), flowval)
    end

    -- Stations have trading capacity
    station:addTrader()

    -- Stations have manufacturing capacity for one randomly-chosen production type
    -- TODO: Assign a station's production type based on system needs (e.g., insure there's
    -- always at least one energy-generating station in each system)
    station:addFactory()
    local prod = prodType
    if not prodType then
        -- No specific production type provided, so pick one randomly
        -- prod = rng:choose(Production.All()) -- if no production type is provided, choose anything randomly
        local rint = rng:getInt(0, 100)
        if rint > 80 then
            prod = rng:choose(Production.P0) -- small chance for a powerplant
        elseif rint > 25 then
            prod = rng:choose(Production.P1) -- good chance for a refinery
        else
            prod = rng:choose(Production.P2) -- good chance for a factory
        end
    end
    station:addProduction(prod)
    station:setSubType(Config:getObjectTypeByName("station_subtypes", prod:getName()))

    -- Station starts with some credits and some energy (energy Item must exist before bid is offered!)
    station:addCredits(Config.econ.eStartCredits * 100)

    -- The station sets asks for selling items its facility produces as outputs,
    -- and sets bids for buying items its facility wants as inputs
    -- Ask prices (for selling) are calculated as the item's base price times a markup value
    -- Bid prices (for buying) are calculated as the item's base price times a markdown value
    for _, input in prod:iterInputs() do
        for i = 1, input.count * 3 do -- multiply initial bids to stimulate early star system production
            -- TODO: Change magic number 33 for "I want..." bids to a multiplier connected to this system's flows
            if input.item == Item.Energy then
                station.trader:addBid(input.item, 100 + rng:getInt(25, 100)) -- make sure Energy-requiring factories bid well
            else
                station.trader:addBid(input.item, math.max(1, math.floor(input.item.energy * Config.econ.markdown * 33)))
            end
        end
    end
    for _, output in prod:iterOutputs() do
        for i = 1, output.count do
            if output.item == Item.Energy then
                station.trader:addAsk(output.item, 1) -- Energy starts out cheap!
            else
                station.trader:addAsk(output.item, math.floor(output.item.energy * Config.econ.markup))
            end
        end
    end

    -- TODO: The weapon installed in each turret/bay should dictate its base emission or thruster color
    -- For now, every weapon per station gets the same pulse weapon color effect
    station.projColorR = self.rng:getUniformRange(0.1, 1.2)
    station.projColorG = self.rng:getUniformRange(0.1, 1.2)
    station.projColorB = self.rng:getUniformRange(0.1, 1.2)

    local typeName = Config:getObjectInfo("object_types", station:getType())
    local subtypeName = Config:getObjectInfo("station_subtypes", station:getSubType())
    Log.Debug("SYSTEM(station) - Added %s %s '%s' (production = %s)", subtypeName, typeName, station:getName(),
        prod:getName())

    -- Add the station to this star system
    self:addChild(station)
    self:addStation(station)

    -- Add station components
    --! NEED TO BE ADDED AFTER ADDING IT TO THE SYSTEM AS CHILD
    addStationComponents(station, hullSize)

    return station
end

function System:spawnPirateStation(hullSize, player)
    local rng = self.rng
    -- Spawn a new space station
    local station = Objects.Station(self.rng:get31(), hullSize)
    station:setType(Config:getObjectTypeByName("object_types", "Station"))
    station:setSubType(Config:getObjectTypeByName("station_subtypes", "Pirate")) -- pirate station

    -- Give the station a name
    station:setName(Words.getCoolName(rng) .. " Marauders")

    -- Set station location within the extent of a randomly selected asteroid field
    station.zone = self:place(station)

    -- Assign the station to an owner
    station:setOwner(player)

    -- Add the black market
    station:addBlackMarket()
    station:addBlackMarketTrader()

    station:addFactory()
    local prod = Production.Piracy
    station:addProduction(prod)

    -- Station starts with some credits and some energy (energy Item must exist before bid is offered!)
    station:addCredits(Config.econ.eStartCredits * 100)

    -- The station sets asks for selling items its facility produces as outputs,
    -- and sets bids for buying items its facility wants as inputs
    -- Ask prices (for selling) are calculated as the item's base price times a markup value
    -- Bid prices (for buying) are calculated as the item's base price times a markdown value
    for _, input in prod:iterInputs() do
        for i = 1, input.count * 3 do -- multiply initial bids to stimulate early star system production
            -- TODO: Change magic number 33 for "I want..." bids to a multiplier connected to this system's flows
            if input.item == Item.Energy then
                station.blackMarketTrader:addBid(input.item, 100 + rng:getInt(25, 100)) -- make sure Energy-requiring factories bid well
            else
                station.blackMarketTrader:addBid(input.item,
                    math.max(1, math.floor(input.item.energy * Config.econ.markdown * 33)))
            end
        end
    end

    -- Add the station to this star system
    self:addChild(station)
    self:addStation(station)

    -- Add station components !NEED TO BE ADDED AFTER ADDING IT TO THE SYSTEM AS CHILD
    --! NEED TO BE ADDED AFTER ADDING IT TO THE SYSTEM AS CHILD
    addStationComponents(station, hullSize)

    return station
end

function System:spawnAI(shipCount, action, player)
    -- Spawn a number of independent AI-controlled ships
    local rng = self.rng
    for i = 1, shipCount do
        local ship = self:spawnShip(rng:choose({ 1, 2, 3, 4, 5, 6 }), player)
        ship:setOwner(player)
        if action then
            ship:pushAction(action)
        end
    end
end

function System:spawnShip(hullSize, player)
    -- Spawn a new ship (with a new ship type)
    local shipHull = hullSize
    local shipRole = Enums.ShipRoles.Combat
    if GameState.gen.uniqueShips or not self.shipType then
        if hullSize == Enums.ShipHulls.Solo then
            self.shipType = Ship.ShipType(self.rng:get31(), Gen.Ship.ShipFighter, shipHull)
        elseif hullSize == Enums.ShipHulls.VeryLarge then
            self.shipType = Ship.ShipType(self.rng:get31(), Gen.Ship.ShipBasic, shipHull)
        else
            self.shipType = Ship.ShipType(self.rng:get31(), Gen.Ship.ShipCapital, shipHull)
        end
    end
    local ship = self.shipType:instantiate(shipHull)
    ship:setType(Config:getObjectTypeByName("object_types", "Ship"))
    ship:setSubType(3 + (shipRole * 6) + (shipHull - 1))
    ship:setHull(shipHull)
    ship:setRole(shipRole)

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

    -- TODO: make sure spawn position for ship is well outside any planetary volume
    local shipPos = self.rng:getDir3():scale(Config.gen.scaleSystem * (1.0 + self.rng:getExp()))
    if Config.gen.scaleSystem < 5e4 then
        while shipPos:distance(Config.gen.origin) > 200000 do -- constrain max extent of small star systems for performance
            shipPos = self.rng:getDir3():scale(Config.gen.scaleSystem * (1.0 + self.rng:getExp()))
        end
    end
    ship:setPos(shipPos)

    -- Add the ship to this star system
    self:addChild(ship)

    -- Add all components to this ship
    -- TODO: For now, every socket gets one of the appropriate components. Later, this must be replaced by:
    -- 1) default components (for ships magically spawned when a new star system is generated)
    -- 2) no components (for ships newly built at a factory)
    -- 3) loaded components (for ships recreated when the player loads a saved game)
    -- NOTE: Components must be instantiated AFTER their parent is added as a child to the star system!

    -- Add as many armor plates as there are armor plugs for
    -- ship:addArmor(Config.gen.compArmorStats.healthMax * ship.countArmor) -- TEMP
    for i = 1, ship.countArmor do
        local armor = Components.Armor()
        armor:setName(ship:getName() .. ": " .. "Glassiron " .. armor:getName() .. " " .. tostring(i))
        insert(ship.components.armor, armor)
        -- ship:plug(armor)
    end

    -- Add as many bays as there are bay plugs for
    for i = 1, ship.countBay do
        local bay = Ship.Bay()
        bay.name = ship:getName() .. ": " .. bay:getName() .. " " .. tostring(i)
        insert(ship.components.bay, bay)
        -- TODO : Does this leak a Bay/RigidBody?
        ship:plug(bay)
    end

    -- Add as many capacitors as there are capacitor plugs for
    for i = 1, ship.countCapacitor do
        local capacitor = Components.Capacitor()
        capacitor:setName(ship:getName() .. ": " .. "Cauchy's " .. capacitor:getName() .. " " .. tostring(i))
        insert(ship.components.capacitor, capacitor)
        ship:plug(capacitor)
    end

    -- Add as many cloaks as there are plugs for
    for i = 1, ship.countCloak do
        local cloak = Components.Cloak()
        cloak:setName(ship:getName() .. ": " .. "Valeria's " .. cloak:getName() .. " " .. tostring(i))
        insert(ship.components.cloak, cloak)
        -- ship:plug(cloak)
    end

    -- Add as many communicators as there are communicator plugs for
    for i = 1, ship.countCommo do
        local commo = Components.Communicator()
        commo:setName(ship:getName() .. ": " .. "Morse's " .. commo:getName() .. " " .. tostring(i))
        insert(ship.components.commo, commo)
        ship:plug(commo)
    end

    -- Add as many computers as there are computer plugs for
    for i = 1, ship.countComputer do
        local computer = Components.Computer()
        computer:setName(ship:getName() .. ": " .. "Babbage's " .. computer:getName() .. " " .. tostring(i))
        insert(ship.components.computer, computer)
        ship:plug(computer)
    end

    -- Add a Hull component as a unitary object
    local hull = Components.Hull()
    hull:setName(ship:getName() .. ": " .. "Bonded " .. hull:getName())
    local hullHealth = hull:getHealth() * Config.gen.shipComponents[Enums.ShipComponents.Hull][hullSize]
    hull:setHealth(hullHealth, hullHealth)
    insert(ship.components.hull, hull)
    -- ship:plug(hull)

    -- Add as many drone racks as there are drone plugs for
    for i = 1, ship.countDrone do
        local drone = Ship.Drone()
        drone:setName(ship:getName() .. ": " .. "Wilson's " .. drone:getName() .. " " .. tostring(i))
        insert(ship.components.drone, drone)
        -- ship:plug(drone)
    end

    -- Add transport pods to every inventory plug
    for i = 1, ship.countInventory do
        local inventory = Components.Inventory()
        inventory:setName(ship:getName() ..
            ": " .. inventory:getName() .. "(" .. Config.gen.shipInventorySize .. ") " .. tostring(i))
        inventory:setInventoryCapacity(Config.gen.shipInventorySize)
        insert(ship.components.inventory, inventory)
        ship:plug(inventory)
    end
    if ship.countInventory > 0 then
        --Log.Debug("SYSTEM(ship): registering Inventory Event.Debug, handler = %s", Entity.mgrInventoryDebug)
        ship:register(Event.Debug, Entity.mgrInventoryDebug)
    end

    -- Add as many sensors as there are sensor plugs for
    for i = 1, ship.countSensor do
        local sensor = Components.Sensor()
        sensor:setName(ship:getName() .. ": " .. "April's " .. sensor:getName() .. " " .. tostring(i))
        insert(ship.components.sensor, sensor)
        -- ship:plug(sensor)
    end

    -- Add as many shield generators as there are shield plugs for
    for i = 1, ship.countShield do
        local shield = Components.Shield()
        shield:setName(ship:getName() .. ": " .. "Magma " .. shield:getName() .. " " .. tostring(i))
        insert(ship.components.shield, shield)
        ship:plug(shield)
    end

    -- Add as many thrusters as there are thruster plugs for
    while true do
        local thruster = Ship.Thruster(ship)
        thruster:setScale(0.5 * ship:getScale())
        insert(ship.components.thruster, thruster)
        -- TODO : Does this leak a Thruster/RigidBody?
        if not ship:plug(thruster) then break end
    end

    -- Add as many turrets as there are turret plugs for
    for i = 1, ship.countTurret do
        local turret = Ship.Turret()
        turret.name = ship:getName() .. ": " .. turret:getName() .. " " .. tostring(i)
        if hullSize == Enums.ShipHulls.VeryLarge then
            turret:setScale(10 * ship:getScale())
        else
            turret:setScale(2 * ship:getScale())
        end
        insert(ship.components.turret, turret)
        -- TODO : Does this leak a Turret/RigidBody?
        ship:plug(turret)
    end

    -- The ship's thruster gets its own color(s)
    ship:addLight(0, 0, 0)

    -- TODO: The weapon installed in each turret/bay should dictate its base emission or thruster color
    -- For now, every weapon per ship gets the same pulse weapon color effect
    ship.projColorR = self.rng:getUniformRange(0.1, 1.2)
    ship.projColorG = self.rng:getUniformRange(0.1, 1.2)
    ship.projColorB = self.rng:getUniformRange(0.1, 1.2)

    -- Add ship to list of ships active in this star system
    self:addShip(ship)

    --local subtypeName = Config:getObjectInfo("ship_subtypes", ship:getSubType())
    --Log.Debug("SYSTEM(ship) - Added %s '%s'", subtypeName, ship:getName())

    return ship
end

function System:spawnBackground()
    -- For a star system background only (no ship), spawn an invisible ship
    -- (because System.lua needs a thing with mass, scale, drag, and thrust
    -- in order to rotate around a camera viewpoint)
    local player = Player("Background Player")
    local hullType = Enums.ShipHulls.Solo
    if not self.shipType then
        self.shipType = Ship.ShipType(self.rng:get31(), Gen.Ship.ShipFighter, hullType)
    end
    local backgroundShip = self.shipType:instantiate(hullType)
    backgroundShip:setHull(hullType)

    self:addChild(backgroundShip)
    insert(self.players, player)

    -- Insert ship into this star system
    backgroundShip:setPos(Config.gen.origin)
    backgroundShip:setFriction(0)
    backgroundShip:setSleepThreshold(0, 0)
    backgroundShip:setOwner(player)
    self:addChild(backgroundShip)
    GameState.player.currentShip = backgroundShip
    GameState.player.humanPlayer:setControlling(backgroundShip)
end

return System
