local UniverseEconomy = require('Systems.Universe.UniverseEconomy')
local StarSystem = require('GameObjects.Entities.StarSystem')
local Actions = requireAll('GameObjects.Actions')
local Jobs = requireAll('GameObjects.Jobs')

---@class Universe
local Universe = class(function(self) end)

---@param seed integer
function Universe:init(seed)
    self.universeSeed = seed
    self.universeRng = RNG.Create(seed):managed()
    self.systems = {}
    self.players = {}
    self.aiPlayers = {}
    self.factions = {}
    self.economy = UniverseEconomy:init()
end

---@param dt integer
function Universe:onUpdate(dt)
    UniverseEconomy:onUpdate(dt)
end

function Universe:createStarSystem(withEconomy)
    -- Spawn a new star system
    ---@type StarSystem
    local system = StarSystem(self.universeRng:get64())
    GameState.world.currentSystem = system --! temporary: remember the player's current star system

    system:generate()

    if not GameState.player.currentShip then
        GameState.player.currentShip = self:createShip(system, nil, {
            owner = GameState.player.humanPlayer,
            shipName = GameState.player.humanPlayerShipName,
            friction = 0,
            sleepThreshold = {
                [1] = 0,
                [2] = 0
            }
        })
    end

    Log.Debug("Added our ship, the '%s', at pos %s", GameState.player.currentShip:getName(),
        GameState.player.currentShip:getPos())

    -- Add System to the Universe
    if withEconomy then
        self:addSystemEconomy(system)
    end
    table.insert(self.systems, system)
    Log.Debug("Added System: " .. system:getName() .. " to the Universe.")
end

---@param system StarSystem
---@param pos Vec3f|nil
---@param shipObject table
function Universe:createShip(system, pos, shipObject)
    -- Add the player's ship
    -- TODO: Integrate this with loading a saved ship

    -- TEMP: Read player's ship hull size from user settings
    local shipSize = GameState.player.shipHull

    local ship = system:spawnShip(shipSize, shipObject.owner)
    ship:setName(shipObject.shipName)

    -- Insert ship into this star system
    local spawnPosition = pos or Config.gen.origin + Vec3f(20000, 20000, 20000)

    ship:setPos(spawnPosition)
    ship:setFriction(shipObject.friction)
    ship:setSleepThreshold(shipObject.sleepThreshold[1], shipObject.sleepThreshold[2])
    ship:setOwner(shipObject.owner, true)
    shipObject.owner:setControlling(ship)

    return ship
end

function Universe:addFaction(name, type, players) end

function Universe:addSystemEconomy(system)
    -- do other stuff here too
    UniverseEconomy:addSystem(system)
end

return Universe
