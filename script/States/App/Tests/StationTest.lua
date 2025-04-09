local Player = require('GameObjects.Entities.Player')
local System = require('GameObjects.Entities.StarSystem')
local DebugControl = require('Legacy.Systems.Controls.Controls.DebugControl')
local Actions = requireAll('GameObjects.Actions')

local StationTest = require('States.Application')
local rng = RNG.FromTime()

function StationTest:spawnStation()
    local station
    do -- Player Ship
        local currentStation = self.currentStation or self.player:getControlling()
        if currentStation then currentStation:delete() end
        station = self.system:spawnStation(Enums.StationHulls.Small, self.player)
        station:setPos(Config.gen.origin)
        station:setFriction(0)
        station:setSleepThreshold(0, 0)
        station:setOwner(self.player, true)
        --self.system:addChild(ship)
        self.player:setControlling(station)
        self.currentStation = station
    end
end

function StationTest:newSystem()
    self.seed = rng:get64()
    Log.Debug('Seed: %s', self.seed)

    if self.system then self.system:delete() end
    self.system = System(self.seed)
    GameState.world.currentSystem = self.system
    GameState:SetState(Enums.GameStates.InGame)

    self:spawnStation()
end

function StationTest:generate()
    self:newSystem()
end

function StationTest:onInit()
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

function StationTest:onInput()
    self.canvas:input()

    if Input:isKeyboardShiftPressed() and Input:isPressed(Button.KeyboardB) then
        self:newSystem()
    elseif Input:isPressed(Button.KeyboardB) then
        self:spawnStation()
    end
end

function StationTest:onUpdate(dt)
    self.player:getRoot():update(dt)
    self.canvas:update(dt)
    Gui:beginGui(self.resX, self.resY)
    Gui:endGui()
end

function StationTest:onDraw()
    self.canvas:draw(self.resX, self.resY)
    Gui:draw()
end

return StationTest
