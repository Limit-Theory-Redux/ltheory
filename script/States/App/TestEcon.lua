-- TODO: Using requireAll locally like this is not recommended.
-- Implementation of requireEach needs to happen.
local Entities = requireAll('GameObjects.Entities')
local Actions = requireAll('GameObjects.Actions')
local Item = require('Systems.Economy.Item')
local TestEcon = require('States.Application')
local SystemMap = require('Systems.CommandView.SystemMap')

local rng = RNG.FromTime()
--local rng = RNG.Create(10)

local kAssets = 4
local kPlayers = 3
local kStations = 4
local kFields = 5
local kFieldCount = 200
local lastPlayer = nil

function TestEcon:getWindowMode ()
  return Bit.Or32(WindowMode.Shown, WindowMode.Resizable)
end

function TestEcon:getTitle ()
  return 'Economy Simulation'
end

function TestEcon:onInit ()
  -- Generate new universe for economic testing
  self.canvas = UI.Canvas()
  self.system = Entities.Test.System(rng:get64())

  -- Add system-wide AI director
  self.tradeAI = Entities.Player("AI Trade Player")
  self.tradeAI:addItem(Item.Credit, 1e10)

  -- Add a generic ship-like entity to serve as the imaginary player ship
  self.tradeShip = Entity()
  self.tradeShip:addInventory(Config.game.eStartCredits)
  self.tradeShip:setOwner(self.tradeAI)

  -- Add Player objects
  for i = 1, kPlayers do
    local tradePlayerName = format("AI Trade Player %d", i)
    local tradePlayer = Entities.Player(tradePlayerName)
    self.system:spawnAI(kAssets, Actions.Wait(100), tradePlayer)
    printf("%d assets added to %s", kAssets, tradePlayerName)
    lastPlayer = tradePlayer
  end

  -- Add Asteroid Field (and Asteroid) objects
  for i = 1, kFields do self.system:spawnAsteroidField(kFieldCount, false) end

  -- Add Station objects
  for i = 1, kStations do
    -- Randomly select an AsteroidField Zone and get its position and extent
    local szone   = rng:choose(self.system:getZones())
    local spos    = nil
    local sextent = 0
    if szone ~= nil then
      spos    = szone:getPos()
      sextent = szone:getExtent()
    end

    -- Create a station, owned by this system's AI player, within a random AsteroidField Zone
    local newStation = self.system:spawnStation(self.tradeAI, spos, sextent)

    -- Assign the new Station to an AI owner
    local ownerNum = rng:getInt(1, kPlayers)
    for i, v in ipairs(self.system.players) do
      if i == ownerNum then
        printf("New station %s should have owner %s", newStation:getName(), v:getName())
--        newStation:setOwner(v) -- causes an infinite loop somewhere
        printf("New station %s actually has owner %s", newStation:getName(), newStation:getOwner():getName())
        break
      end
    end
  end

  self.canvas:add(SystemMap(self.system))

  TestEcon:showStatus()
end

function TestEcon:onInput ()
  self.canvas:input()
end

function TestEcon:onUpdate (dt)
  self.system:update(dt)
  self.canvas:update(dt)
end

function TestEcon:onDraw ()
  self.canvas:draw(self.resX, self.resY)
end

function TestEcon:showStatus ()
  -- Display statuses of players and assets
  self.system:register(Event.Debug, function (system, state)
    local ctx = state.context
    ctx:text("AI Players:")
    for i, v in ipairs(system.players) do
      ctx:indent()
      ctx:text("[%d] %s : %d credits", i, v:getName(), v:getItemCount(Item.Credit))
      ctx:indent()
      ctx:text("Actions:")
      ctx:indent()
      for j, a in ipairs(v.actions) do
        ctx:text("%d : %s", j, a:getName())
      end
      ctx:undent()
      ctx:undent()
      ctx:indent()
      ctx:text("Assets:")
      ctx:indent()
      for asset in v:iterAssets() do
        ctx:text("%s", asset:getName())
        ctx:indent()
        ctx:text("Actions:")
        ctx:indent()
        for j, a in ipairs(asset.actions) do
          ctx:text("%d : %s", j, a:getName())
        end
        ctx:undent()
        ctx:undent()
      end
      ctx:undent()
      ctx:undent()
      ctx:undent()
    end
  end)
end

return TestEcon
