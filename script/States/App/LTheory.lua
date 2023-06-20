local Player = require('GameObjects.Entities.Player')
local System = require('GameObjects.Entities.Test.System')
local DebugControl = require('Systems.Controls.Controls.DebugControl')
local Actions = requireAll('GameObjects.Actions')
local Bindings = require('States.ApplicationBindings')
local MainMenu = require('Systems.Menus.MainMenu')

local LTheory = require('States.Application')
local rng = RNG.FromTime()

local ships = {}
local escortShips = 20

function LTheory:generate()
    self.seed = rng:get64()
    printf('Seed: %s', self.seed)

    if self.system then self.system:delete() end
    self.system = System(self.seed)

    GameState.world.currentSystem = self.system
    GameState:SetState(Enums.GameStates.InGame)

    -- Generate planets (no more than 1 for now)
    for i = 1, 1 do
        self.system:spawnPlanet()
    end

    -- Generate asteroid fields and asteroids
    for i = 1, 10 do
        self.system:spawnAsteroidField(200, 10)
    end

    -- Generate space stations (in asteroid fiels)
    for i = 1, 10 do
        local station = self.system:spawnStation(Enums.StationHulls.Small, self.player, nil)
    end

    -- Generate the player's ship
    local shipSize = Enums.ShipHulls.Large
    local ship
    ship = self.system:spawnShip(shipSize, self.player)
    ship:setName(GameState.player.humanPlayerShipName)
    ship:setPos(Config.gen.origin)
    ship:setFriction(0)
    ship:setSleepThreshold(0, 0)
    ship:setOwner(self.player)
    self.player:setControlling(ship)
    GameState.player.currentShip = ship

    -- Generate escort ships for testing
    for i = 1, escortShips do
        shipSize = rng:choose({ Enums.ShipHulls.Solo,
            Enums.ShipHulls.Small,
            Enums.ShipHulls.Compact,
            Enums.ShipHulls.Medium,
            Enums.ShipHulls.Large,
            Enums.ShipHulls.VeryLarge })
        local escort = self.system:spawnShip(shipSize, nil)
        local offset = rng:getSphere():scale(300)
        escort:setPos(ship:getPos() + offset)
        escort:setOwner(self.player)
        if rng:getInt(0, 100) < 20 then
            escort.usesBoost = true
        end
        escort:pushAction(Actions.Escort(ship, offset))
        insert(ships, escort)
    end

    -- Optional: make the escort ships start out attacking each other
    for i = 1, #ships - 1 do
        ships[i]:pushAction(Actions.Attack(ships[i + 1]))
    end
end

function LTheory:onInit()
    DebugControl.ltheory = self

    --* Audio initializations *--
    Audio.Init()
    Audio.Set3DSettings(0.0, 10, 2);

    self.player = Player("LTheory Player")

    self:generate()

    GameState.ui.hudStyle = Enums.HudStyles.Wide
    GameState.ui.sensorsDisplayed = true
    GameState.ui.showTrackers = true
    GameState.player.humanPlayer = self.player

    self.gameView = Systems.Overlay.GameView(self.player)
    self.canvas = UI.Canvas()
    self.canvas
        :add(self.gameView
            :add(Systems.Controls.Controls.MasterControl(self.gameView, self.player)))

    self.window:setCursor(Enums.CursorFilenames[GameState.ui.cursorStyle], GameState.ui.cursorX, GameState.ui.cursorY)
    MainMenu:SetMenuMode(Enums.MenuMode.Dialog)

end

function LTheory:onInput()
    self.canvas:input()
end

function LTheory:onUpdate(dt)
    -- If player pressed the "ToggleLights" key in Flight Mode, toggle dynamic lighting on/off
    -- NOTE: Performance is OK for just the player's ship, but adding many lit ships & pulses tanks performance
    if Input.GetPressed(Bindings.ToggleLights) then
        GameState.render.pulseLights = not GameState.render.pulseLights
    end

    self.player:getRoot():update(dt)
    self.canvas:update(dt)

    HmGui.Begin(self.resX, self.resY) -- required for HmGui.Draw() to work without crashing
    HmGui.End()
end

function LTheory:onDraw()
    self.canvas:draw(self.resX, self.resY)
    HmGui.Draw() -- post-Rust, required for game universe to be displayed
end

return LTheory
