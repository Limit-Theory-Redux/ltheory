local Player = require('GameObjects.Entities.Player')
local System = require('GameObjects.Entities.StarSystem')
local DebugControl = require('Systems.Controls.Controls.DebugControl')
local Actions = requireAll('GameObjects.Actions')

local PlanetTest = require('States.Application')
local rng = RNG.FromTime()

function PlanetTest:spawnPlanet()
    do -- Player Ship
        if self.currentPlanet then self.currentPlanet:delete() end
        self.currentPlanet = self.system:spawnPlanet(false)
        self.currentPlanet:setPos(Config.gen.origin)
        self.player:setControlling(self.currentPlanet)
        --self.system:addChild(ship)
    end
end

function PlanetTest:newSystem()
    self.seed = rng:get64()
    self.currentPlanet = nil
    Log.Debug('Seed: %s', self.seed)

    if self.system then self.system:delete() end
    self.system = System(self.seed)
    GameState.world.currentSystem = self.system
    GameState:SetState(Enums.GameStates.InGame)

    self:spawnPlanet()
end

function PlanetTest:generate()
    self:newSystem()
end

function PlanetTest:onInit()
    self.player = Player()
    GameState.player.humanPlayer = self.player

    self:generate()

    DebugControl.ltheory = self
    self.gameView = Systems.Overlay.GameView(GameState.player.humanPlayer, self.audio)
    self.canvas = UI.Canvas()
    self.canvas
        :add(self.gameView
            :add(Systems.Controls.Controls.GenTestControl(self.gameView, self.player)))
end

function PlanetTest:onInput()
    self.canvas:input()

    if Input:isKeyboardShiftPressed() and Input:isPressed(Button.KeyboardB) then
        self:newSystem()
    elseif Input:isPressed(Button.KeyboardB) then
        self:spawnPlanet()
    end
end

function PlanetTest:onUpdate(dt)
    self.player:getRoot():update(dt)
    self.canvas:update(dt)
    Gui:beginGui(self.resX, self.resY)
    Gui:endGui()
end

function PlanetTest:onDraw()
    self.canvas:draw(self.resX, self.resY)
    Gui:draw()
end

return PlanetTest
