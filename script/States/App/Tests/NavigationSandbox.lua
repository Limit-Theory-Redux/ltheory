-- Runs navigation scenarios with nobody at the window: spawns each scenario's ships, steers
-- them to their targets through a `NavWorld`, logs the metrics and ends with an exit code.
-- `NavigationSandbox` runs every scenario in turn; `NavigationSandbox/<Name>` runs one. A run
-- fails on a ship-obstacle collision, a deadlock, or a ship that doesn't finish in time.
-- Design: doc/script/navigation-sandbox.md
local Application           = require("States.Application")
local Registry              = require("Core.ECS.Registry")
local DrawEx                = require("UI.DrawEx")

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
local NavAgentHelper        = require("Shared.Helpers.NavAgentHelper")

local ScenarioMetrics      = require("States.App.Tests.NavigationSandbox.Support.ScenarioMetrics")
local NavDebugDraw          = require("States.App.Tests.NavigationSandbox.Support.NavDebugDraw")
local ShipBehaviour         = require("States.App.Tests.NavigationSandbox.Support.ShipBehaviour")
local Metres                = require("States.App.Tests.NavigationSandbox.Support.Metres")

---@class SandboxShipSpec
---@field start number[] { x, y, z } in metres
---@field target number[]|nil { x, y, z } in metres; the ship stops there. Not used with a route, nor with a Pursue, Follow or Orbit behaviour, which have their own point
---@field route number[][]|nil Waypoints { x, y, z } in metres, flown in a loop from the first
---@field behaviour SandboxBehaviourSpec|nil What the ship does besides flying to `target`; see `ShipBehaviour`
---@field facing number[]|nil Direction the ship faces at the start, not along the y axis; toward its first target by default
---@field hull integer|nil An `Enums.ShipHulls` value, which sets the ship's size; Solo by default

---@class SandboxObstacleCircleSpec
---@field centre number[] { x, y, z } in metres
---@field radius number In metres
---@field periodS number Seconds for one turn; the sphere starts on the +x side and moves toward +z

---@class SandboxObstacleSpec
---@field position number[]|nil { x, y, z } in metres; not used with a circle
---@field radius number In metres
---@field circle SandboxObstacleCircleSpec|nil Moves the sphere on a circle in the xz plane

---@class SandboxScenario
---@field name string
---@field ships SandboxShipSpec[]
---@field obstacles SandboxObstacleSpec[]|nil
---@field timeLimit number|nil Seconds of simulated time for every ship to arrive or deadlock; route ships fly until it ends
---@field avoidance SandboxAvoidanceSpec|nil The avoidance tuning; the navigation world's defaults when nil

---@class SandboxAvoidanceSpec
---@field horizonS number Look-ahead, in seconds
---@field clearanceRadii number Distance kept from an obstacle's surface, in ship radii
---@field aimFactor number The detour passes the obstacle's reach at this multiple of it

---@class NavigationSandbox: Application
---@field scenarios SandboxScenario[]|nil The scenarios this app runs; every scenario when nil
local NavigationSandbox = Subclass("NavigationSandbox", Application)

-- Every scenario, in the order `NavigationSandbox` runs them; each is an app of its own.
local SCENARIO_NAMES   = { "SingleShip", "HeadOn", "CrossingStreams", "CircleSwap", "ObstacleField",
    "SeekThrough", "HoldAfterCruise", "PursueRock", "FollowLeader", "OrbitRock" }
local SCENARIO_MODULES = "States.App.Tests.NavigationSandbox."
local SEED             = 1
local DEFAULT_HULL     = Enums.ShipHulls.Solo
local DEFAULT_TIME_LIMIT_S = 120
-- Sensing mask with every collision group bit set.
local SENSING_MASK     = 0xFFFFFFFF
-- The longest physics step; a longer frame advances the simulation by this much only.
local MAX_STEP_S       = 0.05
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
    obstacle   = { 0.5, 0.5, 0.5, 1.0 },
}
-- Filled spheres are drawn at this fraction of their colour: the additive pass is exposed for
-- much dimmer content, and a full-strength fill saturates to white.
local FILL_BRIGHTNESS  = 0.12
local FILL_COLORS      = {}
for state, c in pairs(MARKER_COLORS) do
    FILL_COLORS[state] = { c[1] * FILL_BRIGHTNESS, c[2] * FILL_BRIGHTNESS, c[3] * FILL_BRIGHTNESS, c[4] }
end
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

local toPosition = Metres.position

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
    if blendMode ~= BlendMode.Additive or not self.obstacles then return end
    NavDebugDraw.begin()
    for _, obstacle in ipairs(self.obstacles) do
        NavDebugDraw.sphere(obstacle.rb:getPos(), obstacle.radius, FILL_COLORS.obstacle)
    end
    for _, ship in ipairs(self.ships) do
        local flying = MARKER_COLORS.flying
        NavDebugDraw.sphere(ship.rb:getPos(), ship.rb:getBoundingRadius(), FILL_COLORS[ship.state])
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
    Renderer:setEnvironment(self.envMap, self.envMap)
    CameraManager:setStarDir(STAR_DIR)
end

function NavigationSandbox:createCamera()
    local camera = CameraEntity()
    CameraManager:registerCamera(CAMERA_NAME, camera)
    self.camera = OrbitCameraController(camera,
        { initialPitch = CAMERA_PITCH, minDistance = 0, maxDistance = math.huge, zoomSpeed = 0.6, mouseSensitivity = 0.01 })
    camera:get(CameraDataComponent):setController(self.camera)
    CameraManager:setActiveCamera(CAMERA_NAME)
    -- An entity without a rigid body: the camera orbits its target offset.
    self.camera:setTarget(Registry:createEntity())
end

---Point the camera at the middle of every start and target of the scenario.
---@param scenario SandboxScenario
function NavigationSandbox:frameCamera(scenario)
    -- Each point has a padding: the obstacle's radius, so that its whole sphere is in view.
    local points, padding = {}, {}
    for _, spec in ipairs(scenario.ships) do
        table.insert(points, toPosition(spec.start))
        table.insert(padding, 0)
        local waypoints = spec.route or (spec.target and { spec.target }) or {}
        for _, waypoint in ipairs(waypoints) do
            table.insert(points, toPosition(waypoint))
            table.insert(padding, 0)
        end
        for _, point in ipairs(spec.behaviour and spec.behaviour.through or {}) do
            table.insert(points, toPosition(point))
            table.insert(padding, 0)
        end
    end
    -- A moving obstacle is framed by its whole circle.
    for _, obstacle in ipairs(self.obstacles) do
        table.insert(points, obstacle.circle and obstacle.circle.centre or obstacle.rb:getPos())
        table.insert(padding, obstacle.radius + (obstacle.circle and obstacle.circle.radius or 0))
    end
    local cx, cy, cz = 0, 0, 0
    for _, p in ipairs(points) do cx, cy, cz = cx + p.x, cy + p.y, cz + p.z end
    local centre = Position(cx / #points, cy / #points, cz / #points)
    local extent = 0
    for i, p in ipairs(points) do extent = math.max(extent, p:distance(centre) + padding[i]) end

    self.camera:setTargetOffset(Vec3f(centre.x, centre.y, centre.z))
    self.camera:setDistance(extent * CAMERA_DISTANCE_FACTOR)
end

---@param spec SandboxShipSpec
---@param index integer
---@return SandboxShip
function NavigationSandbox:spawnShip(spec, index)
    local route
    if spec.route then
        route = {}
        for _, waypoint in ipairs(spec.route) do table.insert(route, toPosition(waypoint)) end
    end
    local start = toPosition(spec.start)
    -- A ship that tracks a body has no target until its behaviour gives it one.
    local target = route and route[1] or spec.target and toPosition(spec.target) or start
    local facing
    if spec.facing then
        facing = Vec3f(spec.facing[1], spec.facing[2], spec.facing[3])
    else
        assert(spec.target or route, "a ship without a target or a route needs a facing")
        facing = Vec3f(target.x - start.x, target.y - start.y, target.z - start.z)
    end
    local hull = spec.hull or DEFAULT_HULL
    local entity = ShipGenerator:createFighter(self.rng:get31(), {
        hull = hull,
        position = start,
        rotation = Quat.FromLook(facing:normalize(), UP),
        scale = Config.game.shipHulls.scale[hull],
        isKinematic = false,
        flightProfile = Enums.FlightProfile.Solo,
    })
    entity:add(ConstructComponents.ControlInput())
    local rb = entity:get(PhysicsComponents.RigidBody):getRigidBody()
    self.world:addRigidBody(rb)
    return {
        entity = entity,
        rb = rb,
        spec = spec,
        behaviour = spec.behaviour and ShipBehaviour.create(spec.behaviour),
        target = target,
        route = route,
        waypointIndex = route and 1 or nil,
        label = string.format("ship %d", index),
    }
end

---Where a circling obstacle is `t` seconds into the scenario.
---@param circle SandboxObstacleCircle
---@param t number
---@return Position
local function circlePosition(circle, t)
    local phase = 2 * math.pi * t / circle.periodS
    local c = circle.centre
    return Position(c.x + circle.radius * math.cos(phase), c.y, c.z + circle.radius * math.sin(phase))
end

---A sphere: a kinematic unit-radius body scaled to the obstacle's radius. A static one sits at
---its position; one with a circle starts on it and is moved by `moveObstacles`.
---@param spec SandboxObstacleSpec
---@param index integer
---@return SandboxObstacle
function NavigationSandbox:spawnObstacle(spec, index)
    local radius = Metres.length(spec.radius)
    local circle
    if spec.circle then
        circle = {
            centre = toPosition(spec.circle.centre),
            radius = Metres.length(spec.circle.radius),
            periodS = spec.circle.periodS,
        }
    end
    local rb = RigidBody.CreateSphere()
    rb:setKinematic(true)
    rb:setPos(circle and circlePosition(circle, 0) or toPosition(spec.position))
    rb:setScale(radius)
    local entity = Entity.Create("NavigationSandboxObstacle", PhysicsComponents.RigidBody(rb))
    self.world:addRigidBody(rb)
    return { entity = entity, rb = rb, radius = radius, circle = circle, label = string.format("obstacle %d", index) }
end

---Set the kinematic targets of the circling obstacles to where they are at `t`, so physics sees
---their velocity over the step that ends there.
---@param t number Seconds into the scenario
function NavigationSandbox:moveObstacles(t)
    for _, obstacle in ipairs(self.obstacles) do
        if obstacle.circle then
            obstacle.rb:setKinematicTarget(circlePosition(obstacle.circle, t), obstacle.rb:getRot())
        end
    end
end

---Register the ships as agents in a `NavWorld` over the physics world, turn sensing on so the
---obstacles are found from physics, send each ship to its target, and allocate the buffers its
---results are copied into.
function NavigationSandbox:createNavigation()
    self.nav = NavWorld.Create(self.world)
    for _, ship in ipairs(self.ships) do
        ship.input = ship.entity:get(ConstructComponents.ControlInput)
        ship.agent = NavAgentHelper.addShip(self.nav, ship.entity, ship.rb)
    end
    -- The obstacles are kinematic bodies in the physics world, so sensing finds them in every
    -- collision group; the ships are agents and are never sensed.
    self.nav:setSensing(SENSING_MASK)
    for _, ship in ipairs(self.ships) do
        if ship.behaviour then
            ShipBehaviour.start(self.nav, ship, self.ships, self.obstacles, ScenarioMetrics.RULES)
        else
            self.nav:setArrive(ship.agent, ship.target, ScenarioMetrics.ARRIVAL_RANGE)
        end
    end
    local avoidance = self.scenarios[self.scenarioIndex].avoidance
    if avoidance then
        self.nav:setAvoidanceTuning(avoidance.horizonS, avoidance.clearanceRadii, avoidance.aimFactor)
    end
    self.axes = ffi.new("double[?]", #self.ships * NavWorld.AxisCount())
    self.arrived = ffi.new("bool[?]", #self.ships)
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

    self.ships, self.obstacles = {}, {}
    for i, spec in ipairs(scenario.ships) do
        table.insert(self.ships, self:spawnShip(spec, i))
    end
    for i, spec in ipairs(scenario.obstacles or {}) do
        table.insert(self.obstacles, self:spawnObstacle(spec, i))
    end
    self.metrics = ScenarioMetrics(scenario.name, self.ships, self.obstacles, scenario.avoidance)
    self:createNavigation()
    self:frameCamera(scenario)
    Log.Info("NavigationSandbox: %s: started, %d ships, %d obstacles", scenario.name, #self.ships, #self.obstacles)
end

function NavigationSandbox:endScenario()
    local scenario = self.scenarios[self.scenarioIndex]
    self.metrics:summarize(self.simTime)
    local failures = self.metrics:failures()
    if #failures > 0 then
        self.failedScenarios = self.failedScenarios + 1
        -- Log.Error raises, which would skip the remaining scenarios; the exit code reports the failure.
        Log.Warn("NavigationSandbox: %s: failed: %s (time limit %.0f s)", scenario.name,
            table.concat(failures, ", "), self.timeLimit)
    end

    -- Registered bodies must stay in physics while the navigation world exists.
    self.nav = nil
    for _, ship in ipairs(self.ships) do
        self.world:removeRigidBody(ship.rb)
        Registry:destroyEntity(ship.entity, Enums.Registry.EntityDestroyMode.DestroyChildren)
    end
    for _, obstacle in ipairs(self.obstacles) do
        self.world:removeRigidBody(obstacle.rb)
        Registry:destroyEntity(obstacle.entity, Enums.Registry.EntityDestroyMode.DestroyChildren)
    end
    self.ships, self.obstacles, self.metrics, self.world = nil, nil, nil, nil
    self.axes, self.arrived = nil, nil

    if self.scenarioIndex < #self.scenarios then
        self:startScenario(self.scenarioIndex + 1)
    else
        Log.Info("NavigationSandbox: %d scenarios run, %d failed", #self.scenarios, self.failedScenarios)
        self:quit(self.failedScenarios == 0 and Core.ExitCode.Success or Core.ExitCode.Failure)
    end
end

---Milliseconds since `start`, from a monotonic clock.
---@param start InstantTime
---@return number
local function msSince(start)
    return InstantTime.Now():durationSince(start)
end

---The glue between the navigation step and the ships' control inputs: advance route ships whose
---waypoint is reached and ships on to the next phase of their behaviour, stop the agents of
---finished ships (except those that keep tracking a body), then copy the step's axes into the
---control inputs.
function NavigationSandbox:applyNavigation()
    local count, stride = #self.ships, NavWorld.AxisCount()
    self.nav:copyAxes(self.axes, count * stride)
    self.nav:copyArrived(self.arrived, count)
    for _, ship in ipairs(self.ships) do
        if ship.state ~= "flying" then
            local keeps = ship.behaviour and ShipBehaviour.keepsBehaviour(ship)
            if not ship.released and not keeps then
                self.nav:clearBehaviour(ship.agent)
                ship.released = true
            end
        elseif ship.behaviour then
            ShipBehaviour.advance(self.nav, ship, self.arrived[ship.agent], self.simTime,
                ScenarioMetrics.RULES, self.metrics)
        elseif ship.route and self.arrived[ship.agent] then
            self.metrics:waypointReached(ship, self.simTime)
            self.nav:setArrive(ship.agent, ship.target, ScenarioMetrics.ARRIVAL_RANGE)
        end
        NavAgentHelper.writeAxes(self.axes, ship.agent, ship.input)
    end
end

--- PreSim: the navigation step, then the control input writers; both are timed.
---@param data EventData
function NavigationSandbox:onPreSim(data)
    if not self.metrics then return end
    self.metrics:recordFrame(data:deltaTime())
    local start = InstantTime.Now()
    self.nav:step()
    self.stepMs = msSince(start)
    start = InstantTime.Now()
    self:applyNavigation()
    self.glueMs = msSince(start)
end

--- Sim: actuation from the control input, then the physics step.
---@param data EventData
function NavigationSandbox:onSim(data)
    if not self.metrics then return end
    -- A long frame (the first one loads shaders and assets) must not become one long physics
    -- step: ships at full speed would cross kilometres in it and tunnel through each other.
    local dt = math.min(data:deltaTime(), MAX_STEP_S)
    local start = InstantTime.Now()
    ShipFlightSystem:update()
    local actuationMs = msSince(start)
    self:moveObstacles(self.simTime + dt)
    self.world:update(dt)
    self.simTime = self.simTime + dt
    self.metrics:recordTimings(self.stepMs, self.glueMs, actuationMs)
end

--- PostSim: the metrics of the step, and the end of the scenario.
---@param data EventData
function NavigationSandbox:onPostSim(data)
    if not self.metrics then return end
    -- The points that tracking ships are to be at move with their bodies: judge against where
    -- they are now.
    for _, ship in ipairs(self.ships) do ShipBehaviour.refreshTarget(ship) end
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
        string.format("Arrived %d, waypoints %d, seek points %d, deadlocked %d, of %d ships", metrics.arrivals,
            metrics.waypoints, metrics.passes, metrics.deadlocks, #self.ships),
        string.format("Collisions: %d with obstacles, %d between ships", metrics.obstacleCollisions,
            metrics.shipCollisions),
        string.format("Navigation per frame (mean ms): step %.3f, glue %.3f, actuation %.3f",
            metrics:meanMs(metrics.timings.step), metrics:meanMs(metrics.timings.glue),
            metrics:meanMs(metrics.timings.actuation)),
    }
    self:immediateUI(function()
        for i, line in ipairs(lines) do
            DrawEx.TextAdditive(HUD_FONT, line, HUD_FONT_SIZE, HUD_X, HUD_Y + (i - 1) * HUD_LINE, HUD_W, HUD_H,
                HUD_GREY, HUD_GREY, HUD_GREY, HUD_ALPHA, HUD_ALIGN_X, HUD_ALIGN_Y)
        end
    end)
end

return NavigationSandbox
