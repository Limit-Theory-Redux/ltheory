local Player = require('Legacy.GameObjects.Entities.Player')
local System = require('Legacy.GameObjects.Entities.StarSystem')
local DebugControl = require('Legacy.Systems.Controls.Controls.DebugControl')
local Actions = requireAll('Legacy.GameObjects.Actions')

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
        ship:setOwner(self.player, true)
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
    self.gameView = Legacy.Systems.Overlay.GameView(GameState.player.humanPlayer, self.audio)
    self.canvas = UI.Canvas()
    self.canvas
        :add(self.gameView
            :add(Legacy.Systems.Controls.Controls.GenTestControl(self.gameView, GameState.player.humanPlayer)))
end

function ShipTest:onInput()
    self.canvas:input()

    if Input:isKeyboardShiftPressed() and Input:isPressed(Button.KeyboardB) then
        self:newSystem()
    elseif Input:isPressed(Button.KeyboardB) then
        self:spawnShip()
    end
end

function ShipTest:onUpdate(dt)
    self.player:getRoot():update(dt)
    self.canvas:update(dt)
    Gui:beginGui(self.resX, self.resY)
    Gui:endGui()
end

function ShipTest:onDraw()
    self.canvas:draw(self.resX, self.resY)
    Gui:draw()
end

return ShipTest
