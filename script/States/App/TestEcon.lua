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
local kStations = 2
local kFields = 5
local kFieldSize = 200
local lastPlayer = nil

function TestEcon:getWindowMode ()
  return Bit.Or32(WindowMode.Shown, WindowMode.Resizable)
end

function TestEcon:getTitle ()
  return 'Economy Simulation'
end

function TestEcon:onInit ()
  self.canvas = UI.Canvas()
  self.system = Entities.Test.System(rng:get64())

  self.tradeAI = Entities.Player("AI Trade Player")
  self.tradeAI:addItem(Item.Credit, 1e10)
  self.tradeShip = Entity()
  self.tradeShip:addInventory(Config.game.eStartCredits)
  self.tradeShip:setOwner(self.tradeAI)

  for i = 1, kPlayers do
    local tradePlayerName = format("AI Trade Player %d", i)
    local tradePlayer = Entities.Player(tradePlayerName)
    self.system:spawnAI(kAssets, Actions.Wait(100), tradePlayer)
    printf("%d assets added to %s", kAssets, tradePlayerName)
    lastPlayer = tradePlayer
  end

  for i = 1, kStations do
    local newStation = self.system:spawnStation(self.tradeAI) -- provide a default owner

    local ownerNum = rng:getInt(1, kPlayers)
    for i, v in ipairs(self.system.players) do
      if i == ownerNum then
printf("New station %s will have owner %s", newStation:getName(), v:getName())
--        newStation:setOwner(v)
        break
      end
    end
  printf("New station %s has owner %s", newStation:getName(), newStation:getOwner():getName())
  end

  for i = 1, kFields do self.system:spawnAsteroidField(kFieldSize, 1) end

  self.canvas:add(SystemMap(self.system))

  self.system:register(Event.Debug, function (system, state)
    local ctx = state.context
    ctx:text('AI Players')
    ctx:indent()
    for i, v in ipairs(system.players) do
      ctx:text('[%d] %s : %d credits', i, v:getName(), v:getItemCount(Item.Credit))
    end
    ctx:undent()
  end)
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

return TestEcon
