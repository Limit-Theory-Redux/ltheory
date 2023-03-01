local Player = require('GameObjects.Entities.Player')
local System = require('GameObjects.Entities.Test.System')
local DebugControl = require('Systems.Controls.Controls.DebugControl')
local Actions = requireAll('GameObjects.Actions')

local ShipTest = require('States.Application')
local rng = RNG.FromTime()

function ShipTest:generate ()
  self.seed = rng:get64()
  if true then
    -- self.seed = 7035008865122330386ULL
     self.seed = 15054808765102574876ULL
    -- self.seed = 1777258448479734603ULL
    -- self.seed = 5023726954312599969ULL
  end
  printf('Seed: %s', self.seed)

  if self.system then self.system:delete() end
  self.system = System(self.seed)

  local ship
  do -- Player Ship
    ship = self.system:spawnShip(self.player)
    ship:setPos(Config.gen.origin)
    ship:setFriction(0)
    ship:setSleepThreshold(0, 0)
    ship:setOwner(self.player)
    --self.system:addChild(ship)
    self.player:setControlling(ship)
  end
end

function ShipTest:onInit ()
  self.player = Player()
  self:generate()

  DebugControl.ltheory = self
  self.gameView = Systems.Overlay.GameView(self.player)
  self.canvas = UI.Canvas()
  self.canvas
    :add(self.gameView
      :add(Systems.Controls.Controls.MasterControl(self.gameView, self.player)))
end

function ShipTest:onInput ()
  self.canvas:input()
end

function ShipTest:onUpdate (dt)
  self.player:getRoot():update(dt)
  self.canvas:update(dt)
end

function ShipTest:onDraw ()
  self.canvas:draw(self.resX, self.resY)
end

return ShipTest
