local Player = require('GameObjects.Entities.Player')
local System = require('GameObjects.Entities.Test.System')
local DebugControl = require('Systems.Controls.Controls.DebugControl')
local Actions = requireAll('GameObjects.Actions')
local Bindings = require('States.ApplicationBindings')

local LTheory = require('States.Application')
local rng = RNG.FromTime()

function LTheory:generate ()
  self.seed = rng:get64()
  if true then
    -- self.seed = 7035008865122330386ULL
    -- self.seed = 15054808765102574876ULL
    -- self.seed = 1777258448479734603ULL
    -- self.seed = 5023726954312599969ULL
  end
  printf('Seed: %s', self.seed)

  if self.system then self.system:delete() end
  self.system = System(self.seed)
  Config.game.currentSystem = self.system

  Config.ui.hudDisplayed = Enums.HudModes.Tight

  local ship
  do -- Player Ship
    ship = self.system:spawnShip(self.player)
    ship:setName(Config.game.humanPlayerShipName)
    ship:setPos(Config.gen.origin)
    ship:setFriction(0)
    ship:setSleepThreshold(0, 0)
    ship:setOwner(self.player)
    ship:setHealth(400, 400, 10)
    self.system:addChild(ship)
    self.player:setControlling(ship)
    Config.game.currentShip = ship

    local ships = {}
    for i = 1, 100 do
      local escort = self.system:spawnShip()
      local offset = rng:getSphere():scale(100)
      escort:setPos(ship:getPos() + offset)
      escort:setOwner(self.player)
      if rng:getInt(0, 100) < 20 then
        escort:setHealth(100, 100, 0.3)
        escort.usesBoost = true
      end
      escort:pushAction(Actions.Escort(ship, offset))
      insert(ships, escort)
    end
  end

  for i = 1, 1 do
    local station = self.system:spawnStation()
  end

  for i = 1, 0 do
    self.system:spawnAI(100, Actions.Wait(5), self.player)
  end

  for i = 1, 1 do
    self.system:spawnAsteroidField(500, 10)
  end

  for i = 1, 0 do
    self.system:spawnPlanet()
  end
end

function LTheory:onInit ()
  self.player = Player("LTheory Player")
  self:generate()

  DebugControl.ltheory = self
  self.gameView = Systems.Overlay.GameView(self.player)
  self.canvas = UI.Canvas()
  self.canvas
    :add(self.gameView
      :add(Systems.Controls.Controls.MasterControl(self.gameView, self.player)))
end

function LTheory:onInput ()
  self.canvas:input()
end

function LTheory:onUpdate (dt)
  -- If player pressed the "ToggleLights" key in Flight Mode, toggle dynamic lighting on/off
  -- NOTE: Performance is OK for just the player's ship, but adding many lit ships & pulses tanks performance
  if Input.GetPressed(Bindings.ToggleLights) then
    Config.render.pulseLights = not Config.render.pulseLights
  end

  -- Enable switching between weapon groups
  if Input.GetPressed(Button.Keyboard.N1) and Config.game.weaponGroup ~= 1 then
    Config.game.weaponGroup = 1
  elseif Input.GetPressed(Button.Keyboard.N2) and Config.game.weaponGroup ~= 2 then
    Config.game.weaponGroup = 2
  elseif Input.GetPressed(Button.Keyboard.N3) and Config.game.weaponGroup ~= 3 then
    Config.game.weaponGroup = 3
  elseif Input.GetPressed(Button.Keyboard.N4) and Config.game.weaponGroup ~= 4 then
    Config.game.weaponGroup = 4
  elseif Input.GetPressed(Button.Keyboard.N5) and Config.game.weaponGroup ~= 5 then
    Config.game.weaponGroup = 5
  elseif Input.GetPressed(Button.Keyboard.N6) and Config.game.weaponGroup ~= 6 then
    Config.game.weaponGroup = 6
  elseif Input.GetPressed(Button.Keyboard.N7) and Config.game.weaponGroup ~= 7 then
    Config.game.weaponGroup = 7
  elseif Input.GetPressed(Button.Keyboard.N8) and Config.game.weaponGroup ~= 8 then
    Config.game.weaponGroup = 8
  end

  self.player:getRoot():update(dt)
  self.canvas:update(dt)
end

function LTheory:onDraw ()
  self.canvas:draw(self.resX, self.resY)
end

return LTheory
