local Player = require('GameObjects.Entities.Player')
local System = require('GameObjects.Entities.Test.System')
local DebugControl = require('Systems.Controls.Controls.DebugControl')
local Actions = requireAll('GameObjects.Actions')

local ShipTest = require('States.Application')
local rng = RNG.FromTime()

function ShipTest:spawnShip ()
  local ship
  do -- Player Ship
<<<<<<< HEAD:script/States/App/ShipTest.lua
    ship = self.system:spawnShip(Enums.ShipHulls.Solo, self.player)
=======
    local currentShip = self.currentShip or self.player:getControlling()
    if currentShip then currentShip:delete() end
    ship = self.system:spawnShip(self.player)
>>>>>>> main:script/States/App/Tests/ShipTest.lua
    ship:setPos(Config.gen.origin)
    ship:setFriction(0)
    ship:setSleepThreshold(0, 0)
    ship:setOwner(self.player)
    --self.system:addChild(ship)
    self.player:setControlling(ship)
    self.currentShip = ship
  end
end

function ShipTest:newSystem()
  self.seed = rng:get64()
  printf('Seed: %s', self.seed)

  if self.system then self.system:delete() end
  self.system = System(self.seed)
  GameState.world.currentSystem = self.system
  GameState.gen.uniqueShips = true
  GameState:SetState(Enums.GameStates.InGame)

  self:spawnShip()
end

function ShipTest:generate ()
  self:newSystem()
end

function ShipTest:onInit ()
  self.player = Player()
  GameState.player.humanPlayer = self.player

  --* Audio initializations *--
  Audio.Init()
  Audio.Set3DSettings(0.0, 10, 2);
  
  self:generate()

  DebugControl.ltheory = self
  self.gameView = Systems.Overlay.GameView(self.player)
  self.canvas = UI.Canvas()
  self.canvas
    :add(self.gameView
      :add(Systems.Controls.Controls.GenTestControl(self.gameView, self.player)))
end

function ShipTest:onInput ()
  self.canvas:input()

  if Input.GetKeyboardShift() and Input.GetPressed(Button.Keyboard.B) then
    self:newSystem()
  elseif Input.GetPressed(Button.Keyboard.B) then
    self:spawnShip()
  end
end

function ShipTest:onUpdate (dt)
  self.player:getRoot():update(dt)
  self.canvas:update(dt)
  HmGui.Begin(self.resX, self.resY)
  HmGui.End()
end

function ShipTest:onDraw ()
  self.canvas:draw(self.resX, self.resY)
  HmGui.Draw()
end

return ShipTest
