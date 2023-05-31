local Player = require('GameObjects.Entities.Player')
local System = require('GameObjects.Entities.Test.System')
local DebugControl = require('Systems.Controls.Controls.DebugControl')
local Actions = requireAll('GameObjects.Actions')

local PlanetTest = require('States.Application')
local rng = RNG.FromTime()

function PlanetTest:spawnPlanet ()
  do -- Player Ship
    if self.currentPlanet then self.currentPlanet:delete() end
    self.currentPlanet = self.system:spawnPlanet(false)
    self.currentPlanet:setPos(Config.gen.origin)
    self.player:setControlling(self.currentPlanet)
    --self.system:addChild(ship)
  end
end

function PlanetTest:generate ()
  self.seed = rng:get64()
  self.currentPlanet = nil
  if true then
    -- self.seed = 7035008865122330386ULL
     self.seed = 9356427830726706953ULL
    -- self.seed = 1777258448479734603ULL
    -- self.seed = 5023726954312599969ULL
  end
  printf('Seed: %s', self.seed)

  if self.system then self.system:delete() end
  self.system = System(self.seed)
  GameState.world.currentSystem = self.system
  GameState:SetState(Enums.GameStates.InGame)

  self:spawnPlanet()
end

function PlanetTest:onInit ()
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

function PlanetTest:onInput ()
  self.canvas:input()

  if Input.GetPressed(Button.Keyboard.B) then
    self:spawnPlanet()
  end
end

function PlanetTest:onUpdate (dt)
  self.player:getRoot():update(dt)
  self.canvas:update(dt)
  HmGui.Begin(self.resX, self.resY)
  HmGui.End()
end

function PlanetTest:onDraw ()
  self.canvas:draw(self.resX, self.resY)
  HmGui.Draw()
end

return PlanetTest
