local Player = require('GameObjects.Entities.Player')
local System = require('GameObjects.Entities.Test.System')
local DebugControl = require('Systems.Controls.Controls.DebugControl')
local Actions = requireAll('GameObjects.Actions')

local CooperativeTest = require('States.Application')
local rng = RNG.FromTime()

function CooperativeTest:spawnShip()
    CooperativeGroup("spawnShip", function()
        local currentShip = self.currentShip or self.player:getControlling()
        if currentShip then currentShip:delete() end

        local task = Cooperative(function()
            return self.system:spawnShip(Enums.ShipHulls.Solo, self.player)
        end)

        while not CooperativeCompleted(task) do
            coroutine.yield()
        end

        if CooperativeCompleted(task) then
            local ship = CooperativeResult(task)
            ship:setPos(Config.gen.origin)
            ship:setFriction(0)
            ship:setSleepThreshold(0, 0)
            ship:setOwner(self.player, true)
            self.player:setControlling(ship)
            self.currentShip = ship
        end
    end)
end

function CooperativeTest:asyncTest()
    CooperativeGroup("asyncTest", function()
        local task = CooperativeAsync(function()
            Log.Debug("do something 1")
        end, self, self.onAsyncTest)

        local task2 = CooperativeAsync(function(taskId)
            local counter = 0
            local loops_per_iteration = 25
            local iterations = 500
            for i = 1, iterations do
                Log.Debug(format("Loop iteration: %d", i))

                if i == (counter + loops_per_iteration) then
                    counter = i
                    CooperativeUpdateProgress(taskId, i / iterations)
                    Log.Debug(format("Progress: %d", CooperativeProgress(taskId)))
                    Log.Debug(format("yielded after %d loops", loops_per_iteration))
                    coroutine.yield()
                end
            end
            Log.Debug("do something 2")
        end, self, self.onAsyncTest)

        local task3 = CooperativeAsync(function()
            Log.Debug("do something 3")
        end, self, self.onAsyncTest)

        while not CooperativeCompleted(task) or not CooperativeCompleted(task2) or not CooperativeCompleted(task3) do
            coroutine.yield()
        end

        Log.Debug("ASYNC TEST DONE")
    end)
end

function CooperativeTest:onAsyncTest()
    print("finished")
end

function CooperativeTest:newSystem()
    CooperativeGroup("spawnSystem", function()
        self.seed = rng:get64()
        Log.Debug('Seed: %s', self.seed)

        if self.system then self.system:delete() end
        self.system = System(self.seed)

        GameState.world.currentSystem = self.system
        GameState.gen.uniqueShips = true
        GameState:SetState(Enums.GameStates.InGame)

        self:spawnShip()

        while not CooperativeGroupCompleted("spawnShip") do
            coroutine.yield()
        end

        DebugControl.ltheory = self
        self.gameView = Systems.Overlay.GameView(GameState.player.humanPlayer, self.audio)
        self.canvas = UI.Canvas()
        self.canvas
            :add(self.gameView
                :add(Systems.Controls.Controls.GenTestControl(self.gameView, GameState.player.humanPlayer)))
    end)
end

function CooperativeTest:generate()
    self:newSystem()
end

function CooperativeTest:onInit()
    self.logo                    = Tex2D.Load("./res/images/LTR_logo2.png") -- load the full LTR logo
    self.logoname                = Tex2D.Load("./res/images/LTR-logo-name.png")
    self.logoicon                = Tex2D.Load("./res/images/LTR-logo-icon.png")

    self.player                  = Player()
    GameState.player.humanPlayer = self.player

    self:generate()
end

function CooperativeTest:onInput()
    if self.canvas then
        self.canvas:input()
    end

    if InputInstance:isPressed(Button.KeyboardB) then
        self:asyncTest()
    end
end

function CooperativeTest:onUpdate(dt)
    if self.player and self.player:getRoot() and self.player:getRoot():update(dt) then
        self.player:getRoot():update(dt)
    end

    if self.canvas then
        self.canvas:update(dt)
    end

    -- update event loop
    CooperativeEventLoop:update(dt)

    Gui:beginGui(self.resX, self.resY, InputInstance)
    Gui:endGui(InputInstance)
end

function CooperativeTest:onDraw()
    if self.canvas then
        self.canvas:draw(self.resX, self.resY)
    end

    Gui:draw()
end

function CooperativeTest:showGameLogo()
    -- Draw the LTR game logo on top of the background star system
    local scaleFactor = ((self.resX * self.resY) / (1600 * 900)) ^ 0.5
    local scaleFactorX = self.resX / 1600
    local scaleFactorY = self.resY / 900

    Gui:image(self.logo) -- draw the LTR logo on top of the canvas
    Gui:setPercentSize(76.0 * scaleFactor / scaleFactorX, 24.3 * scaleFactor / scaleFactorY)
    Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)
end

return CooperativeTest
