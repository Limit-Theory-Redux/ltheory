local Test = require('States.Application')
local SoundManager = require('Systems.SFX.SoundManager')
local MusicPlayer = require('Systems.SFX.MusicPlayer')
local InitFiles = require('Systems.Files.InitFiles')
local UIRouter = require('UI.HmGui.UICore.UIRouter')
local UIPageExample = require('script.UI.HmGui.Pages.Example')
local UIPageMainMenu = require('script.UI.HmGui.Pages.MainMenu')

local rng = RNG.FromTime()
local Universe = require("Systems.Universe.Universe")
local System = require('GameObjects.Entities.Test.System')
local DebugControl = require('Systems.Controls.Controls.DebugControl')

local useRenderer = false

function Test:onInit()
    self.renderer = Renderer()

    DebugControl.ltheory = self

    SoundManager:init()
    MusicPlayer:Init() --todo: fix all casing errors

    -- Read user-defined values and update game variables
    InitFiles:readUserInits()

    self.player = Entities.Player(GameState.player.humanPlayerName)
    GameState.player.humanPlayer = self.player

    GameState:SetState(Enums.GameStates.MainMenu)

    Universe:Init()
    self.seed = rng:get64()
    self:createStarSystem()

    -- set initial view
    UIPageExample:setView("Main")
    UIPageMainMenu:setView("Title")

    -- add page
    UIRouter:addPage(UIPageExample)
    UIRouter:addPage(UIPageMainMenu)
    UIRouter:setCurrentPage("Example")
end

function Test:onInput(dt)
    self.canvas:input()
    UIRouter:input(dt)
end

function Test:onUpdate(dt)
    GameState.player.humanPlayer:getRoot():update(dt)
    self.canvas:update(dt)
    Universe:OnUpdate(dt)
    SoundManager:clean(dt)
    MusicPlayer:OnUpdate(dt) --todo fix casing

    Gui:beginGui(self.resX, self.resY, InputInstance)
    UIRouter:update(dt)
    Gui:endGui(InputInstance)
end

function Test:onDraw()
    self.canvas:draw(self.resX, self.resY)

    if useRenderer then
        self.renderer:start(self.resX, self.resY)
        Viewport.Push(0, 0, self.resX, self.resY, true)
        Gui:draw()
        Viewport.Pop()
        self.renderer:stop()
        self.renderer:present(0, 0, self.resX, self.resY)
    else
        Gui:draw()
    end
end

function Test:createStarSystem()
    if self.backgroundSystem then self.backgroundSystem:delete() end

    Log.Debug("------------------------")
    if GameState:GetCurrentState() == Enums.GameStates.MainMenu then
        -- Use custom system generation sizes for a nice background star system
        Config.gen.scaleSystem    = Config.gen.scaleSystemBack
        Config.gen.scalePlanet    = Config.gen.scalePlanetBack
        Config.gen.scalePlanetMod = Config.gen.scalePlanetModBack
        GameState.render.zNear    = Config.gen.zNearBack
        GameState.render.zFar     = Config.gen.zFarBack
    else
        -- Use the "real" system generation sizes for a gameplay star system
        Config.gen.scaleSystem    = Config.gen.scaleSystemReal
        Config.gen.scalePlanet    = Config.gen.scalePlanetReal
        Config.gen.scalePlanetMod = Config.gen.scalePlanetModReal
        GameState.render.zNear    = Config.gen.zNearReal
        GameState.render.zFar     = Config.gen.zFarReal
    end

    if GameState:GetCurrentState() == Enums.GameStates.MainMenu or GameState:GetCurrentState() == Enums.GameStates.Splashscreen then
        -- Spawn a new star system
        self.backgroundSystem = System(self.seed)
        GameState.world.currentSystem = self.backgroundSystem -- remember the player's current star system

        -- Background Mode
        -- Generate a new star system with nebulae/dust, a planet, an asteroid field,
        -- a space station, and an invisible rotating ship
        self.backgroundSystem:spawnBackground() -- spawn a ship that can't be seen

        -- Add a planet
        for i = 1, 1 do
            local planet = self.backgroundSystem:spawnPlanet(false) -- no planetary asteroid belt
            local ppos = planet:getPos()
            ppos.x = ppos.x * 2
            ppos.y = ppos.y * 2
            planet:setPos(ppos) -- move planet away from origin for background
        end

        -- Add an asteroid field
        -- NOTE: Must always add asteroid field (a zone) BEFORE space stations
        for i = 1, rng:getInt(0, 1) do                         -- 50/50 chance of having asteroids
            -- Spawn an asteroid field (a zone)
            self.backgroundSystem:spawnAsteroidField(-1, true) -- -1 parameter is a special case meaning background

            -- Add a space station
            self.backgroundSystem:spawnStation(Enums.StationHulls.Small, GameState.player.humanPlayer, nil)
        end
    else
        -- Quickstart if forced to ingame
        if GameState:GetCurrentState() ~= Enums.GameStates.Quickstart then
            GameState:SetState(Enums.GameStates.ShipCreation)
        end
        Universe:CreateStarSystem(self.seed)
    end

    if GameState:GetCurrentState() == Enums.GameStates.ShipCreation then
        -- TODO: replace with gamestate event system
        Log.Debug("LTheoryRedux: PlayAmbient")
        MusicPlayer:PlayAmbient()

        DebugControl.ltheory = self
        self.gameView = Systems.Overlay.GameView(GameState.player.humanPlayer, self.audio)
        GameState.render.gameView = self.gameView
        self.canvas = UI.Canvas()
        self.canvas
            :add(self.gameView
                :add(Systems.Controls.Controls.GenTestControl(self.gameView, GameState.player.humanPlayer)))

        InputInstance:setCursorVisible(true)
    else
        -- Insert the game view into the application canvas to make it visible
        self.gameView = Systems.Overlay.GameView(GameState.player.humanPlayer, self.audio)
        GameState.render.gameView = self.gameView

        self.canvas = UI.Canvas()
        self.canvas
            :add(self.gameView
                :add(Systems.Controls.Controls.MasterControl(self.gameView, GameState.player.humanPlayer))
            )
        self.gameView:setCameraMode(Enums.CameraMode.FirstPerson)
    end
end

return Test
