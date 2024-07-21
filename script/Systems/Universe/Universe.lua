local UniverseEconomy = require('Systems.Universe.UniverseEconomy')
local StarSystem = require('GameObjects.Entities.StarSystem')
local Actions = requireAll('GameObjects.Actions')
local Jobs = requireAll('GameObjects.Jobs')

---@class Universe
local Universe = class(function(self) end)

local firstRun = true

---@param seed integer
function Universe:init(seed)
    if not firstRun then
        self:_clean()
    end

    -- Player
    GameState.player.humanPlayer = Entities.Player(GameState.player.humanPlayerName)

    self.universeSeed = seed
    self.universeRng = RNG.Create(seed):managed()
    self.systems = {}
    self.players = {}   --* system or universe layer?
    self.aiPlayers = {} --* system or universe layer?
    self.factions = {}  --* system or universe layer?
    self.economy = UniverseEconomy:init()
    firstRun = false
end

---@private
function Universe:_clean() --! this needs a fix: this stays in memory instead of being freed
    Log.Debug("--------------------- Cleaning Universe ---------------------")

    -- destroy systems
    if self.systems and #self.systems > 0 then
        for _, system in ipairs(self.systems) do
            Log.Debug("- Destroying System: %s", system:getName())
            -- destroy assets
            if system.players and #system.players > 0 then
                for _, player in ipairs(system.players) do
                    for _, asset in ipairs(player:getAssets()) do
                        Log.Debug("-- Destroying Asset: %s, Owner: %s", asset:getName(), player:getName())
                        asset:delete()
                    end
                end
            end

            if system.aiPlayers and #system.aiPlayers > 0 then
                for _, player in ipairs(system.aiPlayers) do
                    for _, asset in ipairs(player:getAssets()) do
                        Log.Debug("-- Destroying Asset: %s, Owner: %s", asset:getName(), player:getName())
                        asset:delete()
                    end
                end
            end

            system:delete()
            Log.Debug("--- Destroyed System: %s", system:getName())
        end
    end
    Log.Debug("-------------------------------------------------------------")
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

    local ship = system:spawnShip(shipSize, shipObject.owner)
    ship:setName(shipObject.shipName)

    -- Insert ship into this star system
    local spawnPosition = pos or Config.gen.origin + Position(20000, 20000, 20000)

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

function Universe:systemHasPlayer(system, player, type) --* system or universe layer?
    assert(system)
    assert(player)

    if not type then type = Enums.PlayerTypes.Human end

    if type == Enums.PlayerTypes.Human then
        for _, systemPlayer in ipairs(system.players) do
            if systemPlayer == player then
                return true
            end
        end
    elseif type == Enums.PlayerTypes.AI then
        for _, systemPlayer in ipairs(system.aiPlayers) do
            if systemPlayer == player then
                return true
            end
        end
    end
    return false
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
