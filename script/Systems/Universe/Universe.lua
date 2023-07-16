local UniverseEconomy = require('Systems.Universe.UniverseEconomy')
local System = require('GameObjects.Entities.Test.System')
local Actions = requireAll('GameObjects.Actions')
local Jobs = requireAll('GameObjects.Jobs')

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
        GameState.paused              = false
        GameState.panelActive         = false
        GameState.player.playerMoving = false
        GameState.player.weaponGroup  = 1

        -- Generate a new star system with nebulae/dust, a planet, an asteroid field,
        --   a space station, a visible pilotable ship, and possibly some NPC ships
        local afield                  = nil

        -- Add planets
        local planet                  = nil -- remember the last planet created (TODO: remember ALL the planets)
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
            friction = 0,
            sleepThreshold = {
                [1] = 0,
                [2] = 0
            }
        }

        local playerShip = self:CreateShip(system, nil, shipObject)

        GameState.player.currentShip = playerShip

        printf("Added our ship, the '%s', at pos %s", playerShip:getName(), playerShip:getPos())

        -- Escort ships for testing
        local escortShips = {}
        if GameState.gen.nEscortNPCs > 0 then
            for i = 1, GameState.gen.nEscortNPCs do
                local escort = system:spawnShip(rng:choose({ 1, 2, 3, 4, 5, 6 }), nil)
                local offset = system.rng:getSphere():scale(100)
                escort:setPos(playerShip:getPos() + offset)

                if i > GameState.gen.nEscortNPCs / 2 then
                    escort:pushAction(Actions.Orbit(playerShip, rng:getInt(4, 10) * 10, rng:getInt(10, 40)))
                else
                    escort:pushAction(Actions.Escort(playerShip, offset))
                end

                -- TEMP: a few NPC escort ships get to be "aces" with extra health and maneuverability
                --       These will be dogfighting challenges!
                if rng:getInt(0, 100) < 20 then
                    local escortHullInteg = escort:mgrHullGetHullMax()
                    escort:mgrHullSetHull(floor(escortHullInteg * 1.5), floor(escortHullInteg * 1.5))
                    escort.usesBoost = true
                end

                insert(escortShips, escort)
            end
            -- TESTING: push Attack onto action queue of escort ships
            for i = 1, #escortShips - 1 do
                escortShips[i]:pushAction(Actions.Attack(escortShips[i + 1]))
            end
            printf("Added %d escort ships", GameState.gen.nEscortNPCs)
        end

        -- Add System to the Universe
        table.insert(self.systems, system)
        printf("Added System: " .. system:getName() .. " to the Universe.")

        -- Add System to the Universe
        table.insert(self.systems, system)
        printf("Added System: " .. system:getName() .. " to the Universe.")
    end
    self:AddSystemEconomy(system)
end

function Universe:CreateShip(system, pos, shipObject)
    -- Add the player's ship
    -- TODO: Integrate this with loading a saved ship

    -- TEMP: Read player's ship hull size from user settings
    local shipSize = GameState.player.shipHull

    local ship = system:spawnShip(shipSize, shipObject.owner)
    ship:setName(shipObject.shipName)

    -- Insert ship into this star system
    local spawnPosition = pos or Config.gen.origin
    ship:setPos(spawnPosition)
    ship:setFriction(shipObject.friction)
    ship:setSleepThreshold(shipObject.sleepThreshold[1], shipObject.sleepThreshold[2])
    ship:setOwner(shipObject.owner)
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
