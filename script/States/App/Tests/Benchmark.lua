local Player = require('GameObjects.Entities.Player')
local System = require('GameObjects.Entities.StarSystem')
local DebugControl = require('Systems.Controls.Controls.DebugControl')
local Actions = requireAll('GameObjects.Actions')
local Objects = requireAll('GameObjects.Entities.Objects')

local Benchmark = require('States.Application')

local BENCHMARK_ASTEROIDS = 1000
local BENCHMARK_ASTEROIDS_DISTANCE = 50
local BENCHMARK_STATIONS = 50
local BENCHMARK_STATIONS_DISTANCE = 2500
local BENCHMARK_SHIPS = 300
local BENCHMARK_SHIPS_HULL = Enums.ShipHulls.Solo
local BENCHMARK_SHIPS_DISTANCE = 50

function Benchmark:spawnPlanet()
    self.planet = self.system:spawnPlanet(false)
    self.planet:setPos(Config.gen.origin)
end

function Benchmark:spawnAsteroids()
    local asteroidsStart = self.planet:getRadius() + 10000

    local sideLength = math.floor(math.sqrt(BENCHMARK_ASTEROIDS))
    local rows = sideLength
    local columns = sideLength
    if rows * columns < BENCHMARK_ASTEROIDS then
        columns = columns + 1
    end

    local counter = 0
    local x = 0
    local y = -1000
    local z = asteroidsStart

    for i = 1, BENCHMARK_ASTEROIDS do
        local scale = self.rng:getInt(100, 300)

        local asteroid = Objects.Asteroid(self.rng:get31(), scale)
        asteroid:setType(Config:getObjectTypeByName("object_types", "Asteroid"))
        asteroid:setSubType(Config:getObjectTypeByName("asteroid_subtypes", "Silicaceous"))

        local asteroidName = System:getAsteroidName(self, self.rng)
        asteroid:setName(format("%s", asteroidName))

        asteroid:setScale(scale)

        counter = counter + 1
        if counter < columns then
            x = x + BENCHMARK_ASTEROIDS_DISTANCE
        else
            x = 0
            z = z + BENCHMARK_ASTEROIDS_DISTANCE
            counter = 0
        end

        asteroid:setPos(Config.gen.origin + Position(x, y, z))
        asteroid:setRot(self.rng:getQuat())

        self.system:addChild(asteroid)
    end
end

function Benchmark:spawnShips()
    local shipStart = self.planet:getRadius() + 10000

    local sideLength = math.floor(math.sqrt(BENCHMARK_SHIPS))
    local rows = sideLength
    local columns = sideLength
    if rows * columns < BENCHMARK_SHIPS then
        columns = columns + 1
    end

    local counter = 0
    local x = 0
    local y = 0
    local z = shipStart

    for i = 1, BENCHMARK_SHIPS do
        local ship = self.system:spawnShip(BENCHMARK_SHIPS_HULL, self.player)
        ship:setPos(Config.gen.origin + Position(x, y, z))
        ship:setFriction(0)
        ship:setSleepThreshold(0, 0)
        ship:setOwner(self.player, true)

        if counter == 0 then
            self.player:setControlling(ship)
        end

        counter = counter + 1
        if counter < columns then
            x = x + BENCHMARK_SHIPS_DISTANCE
        else
            x = 0
            z = z + BENCHMARK_SHIPS_DISTANCE
            counter = 0
        end
    end
end

function Benchmark:spawnStations()
    local stationStart = self.planet:getRadius() + 5000

    local sideLength = math.floor(math.sqrt(BENCHMARK_STATIONS))
    local rows = sideLength
    local columns = sideLength
    if rows * columns < BENCHMARK_STATIONS then
        columns = columns + 1
    end

    local counter = 0
    local x = 0
    local y = 2500
    local z = stationStart

    for i = 1, BENCHMARK_STATIONS do
        local station = self.system:spawnStation(Enums.StationHulls.Small, self.player)
        station:setPos(Config.gen.origin + Position(x, y, z))
        station:setFriction(0)
        station:setSleepThreshold(0, 0)
        station:setOwner(self.player, true)

        counter = counter + 1
        if counter < columns then
            x = x + BENCHMARK_STATIONS_DISTANCE
        else
            x = 0
            z = z + BENCHMARK_STATIONS_DISTANCE
            counter = 0
        end
    end
end

function Benchmark:newSystem()
    self.seed = 1
    self.rng = RNG.Create(self.seed):managed()
    self.planet = nil
    Log.Debug('Seed: %s', self.seed)

    if self.system then self.system:delete() end
    self.system = System(self.seed)
    GameState.world.currentSystem = self.system
    GameState:SetState(Enums.GameStates.InGame)

    self:spawnPlanet()
    self:spawnShips()
    self:spawnStations()
    self:spawnAsteroids()
end

function Benchmark:generate()
    self:newSystem()
end

function Benchmark:onInit()
    self.player = Player()
    GameState.player.humanPlayer = self.player

    self:generate()

    GameState.debug.metricsEnabled = true

    DebugControl.ltheory = self
    self.gameView = Systems.Overlay.GameView(GameState.player.humanPlayer, self.audio)
    self.canvas = UI.Canvas()
    self.canvas
        :add(self.gameView
            :add(Systems.Controls.Controls.MasterControl(self.gameView, GameState.player.humanPlayer)))
end

function Benchmark:onInput()
    self.canvas:input()

    if InputInstance:isPressed(Button.KeyboardEscape) then
        self:quit()
    end
end

function Benchmark:onUpdate(dt)
    self.player:getRoot():update(dt)
    self.canvas:update(dt)
    Gui:beginGui(self.resX, self.resY)
    Gui:endGui(InputInstance)
end

function Benchmark:onDraw()
    self.canvas:draw(self.resX, self.resY)
    Gui:draw()
end

return Benchmark
