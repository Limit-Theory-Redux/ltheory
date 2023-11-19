local Player = require('GameObjects.Entities.Player')
local System = require('GameObjects.Entities.Test.System')
local DebugControl = require('Systems.Controls.Controls.DebugControl')
local Actions = requireAll('GameObjects.Actions')

local ShipTest = require('States.Application')
local rng = RNG.FromTime()

function ShipTest:spawnShip()
    local ship
    do -- Player Ship
        local currentShip = self.currentShip or self.player:getControlling()
        if currentShip then currentShip:delete() end
        ship = self.system:spawnShip(Enums.ShipHulls.Solo, self.player)
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
    Log.Debug('Seed: %s', self.seed)

    if self.system then self.system:delete() end
    self.system = System(self.seed)
    GameState.world.currentSystem = self.system
    GameState.gen.uniqueShips = true
    GameState:SetState(Enums.GameStates.InGame)

    self:spawnShip()
end

function ShipTest:generate()
    self:newSystem()
end

function ShipTest:onInit()
    self.player = Player()
    GameState.player.humanPlayer = self.player

    self:generate()

    DebugControl.ltheory = self
    self.gameView = Systems.Overlay.GameView(GameState.player.humanPlayer, self.audio)
    self.canvas = UI.Canvas()
    self.canvas
        :add(self.gameView
            :add(Systems.Controls.Controls.GenTestControl(self.gameView, GameState.player.humanPlayer)))
end

function ShipTest:onInput()
    self.canvas:input()

    if InputInstance:isKeyboardShiftPressed() and InputInstance:isPressed(Button.KeyboardB) then
        self:newSystem()
    elseif InputInstance:isPressed(Button.KeyboardB) then
        self:spawnShip()
    end
end

function ShipTest:onUpdate(dt)
    self.player:getRoot():update(dt)
    self.canvas:update(dt)
    HmGui.Begin(self.resX, self.resY, InputInstance)
    HmGui.End(InputInstance)
end

function ShipTest:onDraw()
    self.canvas:draw(self.resX, self.resY)
    HmGui.Draw()
end

return ShipTest
