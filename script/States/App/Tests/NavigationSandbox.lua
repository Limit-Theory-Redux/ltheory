-- Runs navigation scenarios with nobody at the window: spawns each scenario's ships, steers
-- them to their targets, logs the metrics and ends with an exit code. `NavigationSandbox`
-- runs every scenario in turn; `NavigationSandbox/<Name>` runs one. A run fails when a ship
-- neither arrives nor deadlocks within the scenario's time limit.
-- Design: doc/script/navigation-sandbox.md
local Application           = require("States.Application")
local Registry              = require("Core.ECS.Registry")
local DrawEx                = require("UI.DrawEx")
local ScaleConfig           = require("Config.Gen.UniverseScaleConfig")

local PhysicsComponents     = require("Modules.Physics.Components")
local ConstructComponents   = require("Modules.Constructs.Components")
local ShipGenerator         = require("Modules.Constructs.Managers.Generators.ShipGenerator")
local ShipFlightSystem      = require("Modules.Constructs.Systems.ShipFlightSystem")
local RenderCoreSystem      = require("Modules.Rendering.Systems.RenderCoreSystem")

local CameraEntity          = require("Modules.Cameras.Entities").Camera
local CameraManager         = require("Modules.Cameras.Managers.CameraManager")
local OrbitCameraController = require("Modules.Cameras.Managers.CameraControllers.OrbitCameraController")
local CameraDataComponent   = require("Modules.Cameras.Components.CameraDataComponent")
-- Required for its event subscriptions: it moves the orbit camera and reads its input.
require("Modules.Cameras.Systems.CameraSystem")

local RenderingComponents   = require("Modules.Rendering.Components")
local Entity                = require("Core.ECS.Entity")

local SeekArrive            = require("States.App.Tests.NavigationSandbox.Support.SeekArrive")
local ScenarioMetrics       = require("States.App.Tests.NavigationSandbox.Support.ScenarioMetrics")
local NavDebugDraw          = require("States.App.Tests.NavigationSandbox.Support.NavDebugDraw")

---@class SandboxShipSpec
---@field start number[] { x, y, z } in metres
---@field target number[] { x, y, z } in metres
---@field facing number[]|nil Direction the ship faces at the start, not along the y axis; toward its target by default

---@class SandboxScenario
---@field name string
---@field ships SandboxShipSpec[]
---@field timeLimit number|nil Seconds of simulated time for every ship to arrive or deadlock

---@class NavigationSandbox: Application
---@field scenarios SandboxScenario[]|nil The scenarios this app runs; every scenario when nil
local NavigationSandbox = Subclass("NavigationSandbox", Application)

-- Every scenario, in the order `NavigationSandbox` runs them; each is an app of its own.
local SCENARIO_NAMES   = { "SingleShip", "HeadOn" }
local SCENARIO_MODULES = "States.App.Tests.NavigationSandbox."
local SEED             = 1
local HULL             = Enums.ShipHulls.Solo
local DEFAULT_TIME_LIMIT_S = 120
-- The camera orbits the scenario's centre at this many times its extent, from above.
local CAMERA_DISTANCE_FACTOR = 2.5
local CAMERA_PITCH     = 0.6
local CAMERA_NAME      = "NavigationSandbox"
local UP               = Vec3f(0, 1, 0)
-- The uniform environment: a small cube map of one dim grey, and the light's direction.
local ENV_MAP_SIZE     = 4
local ENV_GREY         = 0.05
local STAR_DIR         = Vec3f(1, 1, 1):normalize()
-- HUD text: font, size and layout in pixels, colour.
local HUD_FONT, HUD_FONT_SIZE = 'Unageo-Medium', 11
local HUD_X, HUD_Y, HUD_LINE = 40, 40, 25
local HUD_W, HUD_H = 40, 20
local HUD_GREY, HUD_ALPHA = 0.9, 0.9
local HUD_ALIGN_X, HUD_ALIGN_Y = 0.0, 0.5
-- Marker colours (r, g, b, a) by ship state; the target and the course use the flying colour.
local MARKER_COLORS    = {
    flying     = { 0.3, 0.7, 1.0, 1.0 },
    arrived    = { 0.3, 1.0, 0.4, 1.0 },
    deadlocked = { 1.0, 0.3, 0.2, 1.0 },
}
local COURSE_ALPHA     = 0.5

---An app that runs only `scenario`. A scenario module returns the result.
---@param scenario SandboxScenario
---@return NavigationSandbox
function NavigationSandbox.ScenarioApp(scenario)
    local app = Subclass("NavigationSandbox" .. scenario.name, NavigationSandbox)
    app.scenario = scenario
    app.scenarios = { scenario }
    return app
end

---@param v number[] { x, y, z } in metres
---@return Position
local function toPosition(v)
    return Position(ScaleConfig:toGameUnits(v[1], "starSystem"), ScaleConfig:toGameUnits(v[2], "starSystem"),
        ScaleConfig:toGameUnits(v[3], "starSystem"))
end

function NavigationSandbox:onInit()
    Window:setPresentMode(PresentMode.NoVsync)
    Window:setFullscreen(false, true)

    -- Scenario modules require this one, so they are loaded only now, after it.
    if not self.scenarios then
        self.scenarios = {}
        for _, name in ipairs(SCENARIO_NAMES) do
            table.insert(self.scenarios, require(SCENARIO_MODULES .. name).scenario)
        end
    end

    self.rng = RNG.Create(SEED)
    self.failedScenarios = 0
    self:createEnvironment()
    self:createCamera()
    self.markers = Entity.Create("NavigationSandboxMarkers",
        RenderingComponents.Render(function(_, blendMode) self:drawMarkers(blendMode) end))
    self:startScenario(1)
end

function NavigationSandbox:onExit()
    Registry:destroyEntity(self.markers, Enums.Registry.EntityDestroyMode.DestroyChildren)
end

---Ship, target and course markers: the ships are far below a pixel at the framing distance.
---Drawn in the additive pass only, so they are not covered by the opaque geometry's lighting.
---@param blendMode integer
function NavigationSandbox:drawMarkers(blendMode)
    if blendMode ~= BlendMode.Additive or not self.ships then return end
    NavDebugDraw.begin()
    for _, ship in ipairs(self.ships) do
        local color = MARKER_COLORS[ship.state]
        local flying = MARKER_COLORS.flying
        NavDebugDraw.sphere(ship.rb:getPos(), ship.rb:getBoundingRadius(), color)
        NavDebugDraw.sphere(ship.target, ScenarioMetrics.ARRIVAL_RANGE, flying, true)
        if ship.state == "flying" then
            NavDebugDraw.line(ship.rb:getPos(), ship.target,
                { flying[1], flying[2], flying[3], COURSE_ALPHA })
        end
    end
    NavDebugDraw.finish()
end

---A uniform environment instead of a generated skybox: the lighting shaders read the
---`envMap` and `irMap` cube maps, and the irradiance of a uniform map is the map itself.
function NavigationSandbox:createEnvironment()
    self.envMap = TexCube.Create(ENV_MAP_SIZE, TexFormat.RGBA16F)
    self.envMap:clear(ENV_GREY, ENV_GREY, ENV_GREY, 1)
    ShaderVar.PushTexCube('envMap', self.envMap)
    ShaderVar.PushTexCube('irMap', self.envMap)
    CameraManager:setStarDir(STAR_DIR)
end

function NavigationSandbox:createCamera()
    local camera = CameraEntity()
    CameraManager:registerCamera(CAMERA_NAME, camera)
    self.camera = OrbitCameraController(camera, { initialPitch = CAMERA_PITCH, minDistance = 0, maxDistance = math.huge })
    camera:get(CameraDataComponent):setController(self.camera)
    CameraManager:setActiveCamera(CAMERA_NAME)
    -- An entity without a rigid body: the camera orbits its target offset.
    self.camera:setTarget(Registry:createEntity())
end

---Point the camera at the middle of every start and target of the scenario.
---@param scenario SandboxScenario
function NavigationSandbox:frameCamera(scenario)
    local points = {}
    for _, spec in ipairs(scenario.ships) do
        table.insert(points, toPosition(spec.start))
        table.insert(points, toPosition(spec.target))
    end
    local cx, cy, cz = 0, 0, 0
    for _, p in ipairs(points) do cx, cy, cz = cx + p.x, cy + p.y, cz + p.z end
    local centre = Position(cx / #points, cy / #points, cz / #points)
    local extent = 0
    for _, p in ipairs(points) do extent = math.max(extent, p:distance(centre)) end

    self.camera:setTargetOffset(Vec3f(centre.x, centre.y, centre.z))
    self.camera:setDistance(extent * CAMERA_DISTANCE_FACTOR)
end

---@param spec SandboxShipSpec
---@param index integer
---@return SandboxShip
function NavigationSandbox:spawnShip(spec, index)
    local start, target = toPosition(spec.start), toPosition(spec.target)
    local facing = spec.facing and Vec3f(spec.facing[1], spec.facing[2], spec.facing[3])
        or Vec3f(target.x - start.x, target.y - start.y, target.z - start.z)
    local entity = ShipGenerator:createFighter(self.rng:get31(), {
        position = start,
        rotation = Quat.FromLook(facing:normalize(), UP),
        scale = Config.game.shipHulls.scale[HULL],
        isKinematic = false,
        flightProfile = Enums.FlightProfile.Solo,
    })
    entity:add(ConstructComponents.ControlInput())
    local rb = entity:get(PhysicsComponents.RigidBody):getRigidBody()
    self.world:addRigidBody(rb)
    return { entity = entity, rb = rb, target = target, label = string.format("ship %d", index) }
end

---@param index integer Index into `self.scenarios`
function NavigationSandbox:startScenario(index)
    local scenario = self.scenarios[index]
    self.scenarioIndex = index
    self.simTime = 0
    self.timeLimit = scenario.timeLimit or DEFAULT_TIME_LIMIT_S

    local tuning = Config.game.physics
    self.world = Physics.Create()
    self.world:setLengthUnit(tuning.lengthUnit)
    self.world:setContactNaturalFrequency(tuning.contactNaturalFrequency)

    self.ships = {}
    for i, spec in ipairs(scenario.ships) do
        table.insert(self.ships, self:spawnShip(spec, i))
    end
    self.metrics = ScenarioMetrics(scenario.name, self.ships)
    self:frameCamera(scenario)
    Log.Info("NavigationSandbox: %s: started, %d ships", scenario.name, #self.ships)
end

function NavigationSandbox:endScenario()
    local scenario = self.scenarios[self.scenarioIndex]
    self.metrics:summarize(self.simTime)
    if self.metrics:unfinished() > 0 then
        self.failedScenarios = self.failedScenarios + 1
        -- Log.Error raises, which would skip the remaining scenarios; the exit code reports the failure.
        Log.Warn("NavigationSandbox: %s: failed, %d ships still flying after the %.0f s limit", scenario.name,
            self.metrics:unfinished(), self.timeLimit)
    end

    for _, ship in ipairs(self.ships) do
        self.world:removeRigidBody(ship.rb)
        Registry:destroyEntity(ship.entity, Enums.Registry.EntityDestroyMode.DestroyChildren)
    end
    self.ships, self.metrics, self.world = nil, nil, nil

    if self.scenarioIndex < #self.scenarios then
        self:startScenario(self.scenarioIndex + 1)
    else
        Log.Info("NavigationSandbox: %d scenarios run, %d failed", #self.scenarios, self.failedScenarios)
        self:quit(self.failedScenarios == 0 and Core.ExitCode.Success or Core.ExitCode.Failure)
    end
end

--- PreSim: the control input writers.
---@param data EventData
function NavigationSandbox:onPreSim(data)
    if not self.metrics then return end
    self.metrics:recordFrame(data:deltaTime())
    for _, ship in ipairs(self.ships) do
        if ship.state == "flying" then
            SeekArrive.steer(ship.entity, ship.target)
        else
            SeekArrive.release(ship.entity)
        end
    end
end

--- Sim: actuation from the control input, then the physics step.
---@param data EventData
function NavigationSandbox:onSim(data)
    if not self.metrics then return end
    local dt = data:deltaTime()
    ShipFlightSystem:update()
    self.world:update(dt)
    self.simTime = self.simTime + dt
end

--- PostSim: the metrics of the step, and the end of the scenario.
---@param data EventData
function NavigationSandbox:onPostSim(data)
    if not self.metrics then return end
    self.metrics:update(self.world, self.simTime)
    if self.metrics:isDone() or self.simTime >= self.timeLimit then
        self:endScenario()
    end
end

---@param data EventData
function NavigationSandbox:onRender(data)
    RenderCoreSystem:render(data)
    if not self.metrics then return end

    local metrics = self.metrics
    local lines = {
        string.format("Scenario %d/%d: %s", self.scenarioIndex, #self.scenarios, metrics.name),
        string.format("Simulated: %.1f s of %.0f s", self.simTime, self.timeLimit),
        string.format("Arrived %d, deadlocked %d, collisions %d, of %d ships", metrics.arrivals,
            metrics.deadlocks, metrics.collisions, #self.ships),
    }
    self:immediateUI(function()
        for i, line in ipairs(lines) do
            DrawEx.TextAdditive(HUD_FONT, line, HUD_FONT_SIZE, HUD_X, HUD_Y + (i - 1) * HUD_LINE, HUD_W, HUD_H,
                HUD_GREY, HUD_GREY, HUD_GREY, HUD_ALPHA, HUD_ALIGN_X, HUD_ALIGN_Y)
        end
    end)
end

return NavigationSandbox
