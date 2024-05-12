local UniverseEconomy = require('Systems.Universe.UniverseEconomy')
local StarSystem = require('GameObjects.Entities.StarSystem')
local Actions = requireAll('GameObjects.Actions')
local Jobs = requireAll('GameObjects.Jobs')

---@class Universe
local Universe = class(function(self) end)

---@param seed integer
function Universe:init(seed)
    self:_clean()

    self.universeSeed = seed
    self.universeRng = RNG.Create(seed):managed()
    self.systems = {}
    self.players = {}
    self.aiPlayers = {}
    self.factions = {}
    self.economy = UniverseEconomy:init()
end

---@private
function Universe:_clean()
    Log.Debug("--- Cleaning Universe ---")

    -- destroy systems
    if self.systems and #self.systems > 0 then
        for _, system in ipairs(self.systems) do
            Log.Debug("Destroyed System: %s", system:getName())
            system:delete()
        end
    end

    -- destroy assets
    if self.players and #self.players > 0 then
        for _, player in ipairs(self.players) do
            for _, asset in ipairs(player:getAssets()) do
                Log.Debug("Destroyed Asset: %s, Owner: %s", asset:getName(), player:getName())
                asset:delete()
            end
        end
    end

    if self.aiPlayers and #self.aiPlayers > 0 then
        for _, player in ipairs(self.aiPlayers) do
            for _, asset in ipairs(player:getAssets()) do
                Log.Debug("Destroyed Asset: %s, Owner: %s", asset:getName(), player:getName())
                asset:delete()
            end
        end
    end
    Log.Debug("-------------------------")
end

---@param dt integer
function Universe:onUpdate(dt)
    UniverseEconomy:onUpdate(dt)
end

---@param withEconomy boolean
---@return StarSystem
function Universe:createStarSystem(withEconomy)
    -- Spawn a new star system
    ---@type StarSystem
    local system = StarSystem(self.universeRng:get64())
    GameState.world.currentSystem = system --! temporary: remember the player's current star system

    system:generate()

    if GameState:GetCurrentState() <= Enums.GameStates.MainMenu then
        system:spawnBackground()
        Log.Debug("Spawn Background")
    end

    -- Add System to the Universe
    if withEconomy then
        self:addSystemEconomy(system)
    end
    table.insert(self.systems, system)
    Log.Debug("Added System: %s to the Universe.", system:getName())
    return system
end

---@param system StarSystem
---@param pos Vec3f|nil
---@param shipObject table
function Universe:createShip(system, pos, shipObject)
    -- Add the player's ship
    -- TODO: Integrate this with loading a saved ship

    -- TEMP: Read player's ship hull size from user settings
    local shipSize = GameState.player.shipHull

    self:playerEnterSystem(system, shipObject.owner)
    local ship = system:spawnShip(shipSize, shipObject.owner)
    ship:setName(shipObject.shipName)

    -- Insert ship into this star system
    local spawnPosition = pos or Config.gen.origin + Vec3f(20000, 20000, 20000)

    ship:setPos(spawnPosition)
    ship:setFriction(shipObject.friction)
    ship:setSleepThreshold(shipObject.sleepThreshold[1], shipObject.sleepThreshold[2])
    shipObject.owner:setControlling(ship)

    return ship
end

function Universe:addPlayer(player)
    assert(self.players)
    table.insert(self.players, player)
    Log.Debug("Added player to Universe: %s", player:getName())
end

function Universe:addFaction(name, type, players) end

function Universe:addSystemEconomy(system)
    -- do other stuff here too
    UniverseEconomy:addSystem(system)
end

function Universe:playerEnterSystem(system, enteringPlayer, type)
    assert(system)
    assert(enteringPlayer)

    if not type then type = Enums.PlayerTypes.Human end

    if type == Enums.PlayerTypes.Human then
        table.insert(system.players, enteringPlayer)
    elseif type == Enums.PlayerTypes.AI then
        table.insert(system.aiPlayers, enteringPlayer)
    end

    Log.Debug("Player %s entered system: %s", enteringPlayer:getName(), system:getName())
end

function Universe:playerLeaveSystem(system, leavingPlayer, type)
    assert(system)
    assert(leavingPlayer)

    if not type then type = Enums.PlayerTypes.Human end

    if type == Enums.PlayerTypes.Human then
        for index, player in ipairs(system.players) do
            if player:getGuid() == leavingPlayer:getGuid() then
                table.remove(system.players, index)
            end
        end
    elseif type == Enums.PlayerTypes.AI then
        for index, player in ipairs(system.aiPlayers) do
            if player:getGuid() == leavingPlayer:getGuid() then
                table.remove(system.aiPlayers, index)
            end
        end
    end

    Log.Debug("Player %s leaving system: %s", leavingPlayer:getName(), system:getName())
end

return Universe
