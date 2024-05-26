local Player = require('GameObjects.Entities.Player')
local System = require('GameObjects.Entities.Test.System')
local DebugControl = require('Systems.Controls.Controls.DebugControl')
local Actions = requireAll('GameObjects.Actions')
local Words = require('Systems.Gen.Words')
local SoundManager = require("Systems.SFX.SoundManager")

local BattleTest = require('States.Application')
local rng = RNG.FromTime()

function BattleTest:spawnFactions()
    local aiAssetCount = 30

    for i = 1, 2 do
        -- temp name until we have rnd names
        local aiPlayer = Entities.Player("AI Battle Player " .. i)

        local factionType = math.random(1, #Enums.FactionTypeNames)
        local factionName

        if factionType == Enums.FactionType.Corporation or
            factionType == Enums.FactionType.TradingGuild or
            factionType == Enums.FactionType.Empire then
            do
                factionName = Words.getCoolName(rng) .. " " .. Enums.FactionTypeNames[factionType]
            end
        else
            factionName = Enums.FactionTypeNames[factionType] .. " " .. Words.getCoolName(rng)
        end

        local playerFaction = Entities.Faction({
            name = factionName,
            type = factionType,
            owner = aiPlayer
        })

        self.system:spawnAI(aiAssetCount, Actions.Wait(1), aiPlayer)
        printf("%d assets added to %s", aiAssetCount, aiPlayer:getName())
        -- Configure assets
        for asset in aiPlayer:iterAssets() do
            asset:setFaction(playerFaction)
            self.system:place(asset)
        end

        -- Add AI Player to the system
        table.insert(self.system.aiPlayers, aiPlayer)
    end

    -- Make them hate each other
    for asset in self.system.aiPlayers[1]:iterAssets() do
        for assetP2 in self.system.aiPlayers[2]:iterAssets() do
            asset:setDisposition(assetP2, Config.game.dispoMin)
            assetP2:setDisposition(asset, Config.game.dispoMin)

            asset:pushAction(Actions.Attack(assetP2))
            assetP2:pushAction(Actions.Attack(asset))
        end
    end
end

function BattleTest:newSystem()
    self.seed = rng:get64()
    Log.Debug('Seed: %s', self.seed)

    if self.system then self.system:delete() end
    self.system = System(self.seed)
    self.system.aiPlayers = {}
    GameState.world.currentSystem = self.system
    GameState.gen.uniqueShips = true
    GameState:SetState(Enums.GameStates.InGame)

    local aField = self.system:spawnAsteroidField(400, false)
    Log.Debug("Added %s asteroids to %s", GameState.gen.nAsteroids, aField:getName())

    do -- Player Ship
        local currentShip = self.currentShip or self.player:getControlling()
        if currentShip then currentShip:delete() end
        local ship = self.system:spawnShip(Enums.ShipHulls.Solo, self.player)
        ship:setPos(aField:getPos() + Position(0, 1000, 0))
        ship:setFriction(0)
        ship:setSleepThreshold(0, 0)
        ship:setOwner(self.player, true)
        self.player:setControlling(ship)
        self.currentShip = ship
    end

    self:spawnFactions()
end

function BattleTest:generate()
    self:newSystem()
end

function BattleTest:onInit()
    self.player = Player()
    GameState.player.humanPlayer = self.player

    self:generate()

    SoundManager:init()

    DebugControl.ltheory = self
    self.gameView = Systems.Overlay.GameView(GameState.player.humanPlayer, self.audio)
    GameState.render.gameView = self.gameView
    self.canvas = UI.Canvas()
    self.canvas
        :add(self.gameView
            :add(Systems.Controls.Controls.CommandControl(self.gameView, GameState.player.humanPlayer)))
end

function BattleTest:onInput()
    self.canvas:input()
end

function BattleTest:onUpdate(dt)
    self.player:getRoot():update(dt)
    self.canvas:update(dt)
    SoundManager:clean(dt)
    Gui:beginGui(self.resX, self.resY, InputInstance)
    Gui:endGui(InputInstance)
end

function BattleTest:onDraw()
    self.canvas:draw(self.resX, self.resY)
    Gui:draw()
end

return BattleTest
