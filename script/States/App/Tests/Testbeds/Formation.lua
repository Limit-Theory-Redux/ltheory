-- Formation testbed: a wing of fighters flying in formation, to watch and to measure, with no player
-- ship. The leader flies a looping patrol round a point; the wingmen hold their slots
-- (`FormationSystem`). The orbit camera follows the leader; F switches the shape. The corner text, and
-- the log every two seconds, give each wingman's distance from its slot in spacings: now, on average
-- and at most, overall and in turns.
--
-- Launch: ./bin/ltr.exe Testbeds/Formation
--         (or: cargo run -- Testbeds/Formation)
-- Env:    LTHEORY_FORMATION_SHAPE (a FormationDefs id, Wedge by default), LTHEORY_FORMATION_WINGMEN (4),
--         LTHEORY_FORMATION_SPACING (spacing in hull radii), LTHEORY_FORMATION_PACE (the leader's share of top speed)
-- Camera: right mouse turns, the wheel zooms.
-- Design: doc/script/formations.md

local Application             = require("States.Application")
local Registry                = require("Core.ECS.Registry")
local DrawEx                  = require("UI.DrawEx")
local Nav                     = require("Modules.Navigation")
local PhysicsComponents       = require("Modules.Physics.Components")
local CelestialEntities       = require("Modules.CelestialObjects.Entities")
local SolarSystemVisualizer   = require("Modules.CelestialObjects.Managers.SolarSystemVisualizer")
local CelestialLightingSystem = require("Modules.Rendering.Systems.CelestialLightingSystem")
local RenderCoreSystem        = require("Modules.Rendering.Systems.RenderCoreSystem")
local ShipFlightSystem        = require("Modules.Constructs.Systems.ShipFlightSystem")
local TravelDriveSystem       = require("Modules.Constructs.Systems.TravelDriveSystem")
local ConstructManager        = require("Modules.Constructs.Managers.ConstructManager")
local ConstructComponents     = require("Modules.Constructs.Components")
local CameraEntity            = require("Modules.Cameras.Entities").Camera
local CameraManager           = require("Modules.Cameras.Managers.CameraManager")
local OrbitCameraController   = require("Modules.Cameras.Managers.CameraControllers.OrbitCameraController")
local CameraDataComponent     = require("Modules.Cameras.Components.CameraDataComponent")
-- Required for its event subscriptions: it moves the orbit camera and reads its input.
local _                       = require("Modules.Cameras.Systems.CameraSystem")
local Skybox                  = require("Shared.Generation.Skybox")
local ScaleConfig             = require("Config.Gen.UniverseScaleConfig")
local ShipKinds               = require("Shared.Definitions.ShipKindDefs")
local FormationDefs           = require("Shared.Definitions.FormationDefs")
local SpawnerSystem           = require("Modules.Spawning.Systems.SpawnerSystem")
local FormationSystem         = require("Modules.Formations.Systems.FormationSystem")
local Formations              = require("Modules.Formations.Components")

---@class FormationTestbed: Application
local FormationTestbed = Subclass("FormationTestbed", Application)

local SEED = 20261127
local CAMERA_NAME = "FormationTestbed"
local STAR_RADIUS_M, STAR_DISTANCE_M = 7e8, 1.5e11
local SHAPES = { "Wedge", "LineAbreast", "Column", "EchelonRight", "Diamond" }
-- The leader's loop, in game units (a unit is 10 km): a patrol round a point, as a wing's patrol
-- route is made: many points on a tilted ellipse, so its turns are gentle.
local LOOP = {}
for i = 1, 16 do
    local angle = (i - 1) * 2 * math.pi / 16
    LOOP[i] = Position(math.cos(angle) * 1.6, math.sin(angle * 2) * 0.15, math.sin(angle) * 1.0)
end
-- A wingman is in a turn while the leader turns faster than this (rad/s).
local TURNING_RAD_S = 0.03
local SIM_STEP_S, MAX_ADVANCE_S = 0.05, 0.05
local HUD_FONT, HUD_FONT_SIZE = 'Unageo-Medium', 11
local HUD_X, HUD_Y, HUD_LINE = 40, 40, 22

function FormationTestbed:onInit()
    local tuning = Config.game.physics
    self.world = Physics.Create()
    self.world:setLengthUnit(tuning.lengthUnit)
    self.world:setContactNaturalFrequency(tuning.contactNaturalFrequency)
    self.skybox = Skybox.Create(SEED)

    local star = CelestialEntities.Star(SEED)
    local transform = star:get(PhysicsComponents.Transform)
    transform:setScale(ScaleConfig:toGameUnits(STAR_RADIUS_M, "star"))
    local d = ScaleConfig:toGameUnits(STAR_DISTANCE_M, "starSystem")
    transform:setPos(Position(-d * 0.6, d * 0.3, d * 0.75))
    SolarSystemVisualizer:materializeEntity(star, self.world)
    self.star = star

    Nav.attach(self.world)
    self.constructs = ConstructManager(Registry, self.world)
    SpawnerSystem:attach(self.world, self.constructs)

    local wingmen = tonumber(os.getenv("LTHEORY_FORMATION_WINGMEN")) or 4
    self.ships = {}
    for i = 0, wingmen do
        -- Bare hulls of one design, side by side: no crew or AI, the formation flies them.
        local handle = SpawnerSystem:buildHull(ShipKinds.Fighter, { position = Position(0.05 * i, 0, 0.1), seed = SEED },
            "Wing " .. i)
        local ship = handle.root
        ship:add(ConstructComponents.TravelDrive())
        ship:add(ConstructComponents.ControlInput())
        self.ships[#self.ships + 1] = ship
    end
    self.shapeIndex = 1
    local wanted = os.getenv("LTHEORY_FORMATION_SHAPE")
    for i, shape in ipairs(SHAPES) do
        if shape == wanted then self.shapeIndex = i end
    end
    self.started = false
    self.stats = {}
    self:createCamera()
end

function FormationTestbed:createCamera()
    local camera = CameraEntity()
    CameraManager:registerCamera(CAMERA_NAME, camera)
    self.camera = OrbitCameraController(camera, {
        initialPitch = 0.45, distance = 0.09, minDistance = 0.01, maxDistance = 50, zoomRatio = 0.15, mouseSensitivity = 0.01,
    })
    camera:get(CameraDataComponent):setController(self.camera)
    CameraManager:setActiveCamera(CAMERA_NAME)
    self.camera:enable()
    self.camera:setTarget(self.ships[1])
end

---The ships fly once physics has stepped (navigation reads bodies from the query index).
function FormationTestbed:start()
    self.started = true
    self.formation = FormationSystem:create(SHAPES[self.shapeIndex], self.ships, {
        spacingRadii = tonumber(os.getenv("LTHEORY_FORMATION_SPACING")),
        paceShare = tonumber(os.getenv("LTHEORY_FORMATION_PACE")),
    })
    Nav.order(self.ships[1], Nav.Orders.Patrol { points = LOOP, loop = true })
    -- For comparison: the wing with no avoidance at all.
    if os.getenv("LTHEORY_FORMATION_NO_AVOIDANCE") then
        for _, world in Registry:view(Nav.Components.NavWorld) do world:getNav():setSensing(0) end
    end
end

---@param data EventData
function FormationTestbed:onPreSim(data)
    if not self.started and (self.simSteps or 0) >= 2 then self:start() end
    if not self.started then return end
    self:measure(data:deltaTime())
    self.logClock = (self.logClock or 0) + data:deltaTime()
    if self.logClock >= 2 then
        self.logClock = 0
        for _, line in ipairs(self:lines()) do Log.Info("FormationTestbed: %s", line) end
    end
end

---Each wingman's distance from its slot, in spacings, now and summed for the averages.
---@param dt number
function FormationTestbed:measure(dt)
    local leader = self.ships[1]
    local lead = leader:get(PhysicsComponents.RigidBody):getRigidBody()
    local rot = lead:getRot()
    -- The leader's level frame, as navigation places the slots: its nose, with the system's up.
    local forward = rot:getForward()
    local right = Vec3d(-forward.z, 0, forward.x)
    if right:length() < 1e-9 then right = rot:getRight() else right = right:normalize() end
    local up = Vec3d(right.y * forward.z - right.z * forward.y, right.z * forward.x - right.x * forward.z,
        right.x * forward.y - right.y * forward.x)
    local turning = lead:getVelocityA():length() > TURNING_RAD_S
    local unitsPerM = ScaleConfig:toGameUnits(1, "starSystem")
    for i = 2, #self.ships do
        local member = self.ships[i]:get(Formations.FormationMember)
        if member and member.offset then
            local o = member.offset
            local p = lead:getPos()
            local sx = p.x + (right.x * o.x + up.x * o.y + forward.x * o.z) * unitsPerM
            local sy = p.y + (right.y * o.x + up.y * o.y + forward.y * o.z) * unitsPerM
            local sz = p.z + (right.z * o.x + up.z * o.y + forward.z * o.z) * unitsPerM
            local q = self.ships[i]:get(PhysicsComponents.RigidBody):getRigidBody():getPos()
            local spacing = math.max(1e-9, (member.spacingM or 1) * unitsPerM)
            -- LTHEORY_FORMATION_TRACE: this wing's error each frame, along the leader's axes, in metres.
            if i == 2 and os.getenv("LTHEORY_FORMATION_TRACE") then
                local ex, ey, ez = q.x - sx, q.y - sy, q.z - sz
                local v = self.ships[i]:get(PhysicsComponents.RigidBody):getRigidBody():getVelocity() - lead:getVelocity()
                local mps = 1 / unitsPerM
                local lv = lead:getVelocity()
                local slip = math.deg(math.acos(math.max(-1, math.min(1,
                    (lv.x * forward.x + lv.y * forward.y + lv.z * forward.z) / math.max(1e-12, lv:length())))))
                Log.Info("FormationTrace: leader nose %.1f deg off its course, turning %.3f rad/s", slip, lead:getVelocityA():length())
                Log.Info("FormationTrace: err right %.0f up %.0f fwd %.0f | dv right %.0f up %.0f fwd %.0f | dt %.3f",
                    (ex * right.x + ey * right.y + ez * right.z) * mps, (ex * up.x + ey * up.y + ez * up.z) * mps,
                    (ex * forward.x + ey * forward.y + ez * forward.z) * mps,
                    (v.x * right.x + v.y * right.y + v.z * right.z) * mps, (v.x * up.x + v.y * up.y + v.z * up.z) * mps,
                    (v.x * forward.x + v.y * forward.y + v.z * forward.z) * mps, dt)
            end
            local err = math.sqrt((q.x - sx) ^ 2 + (q.y - sy) ^ 2 + (q.z - sz) ^ 2) / spacing
            local s = self.stats[i] or { sum = 0, time = 0, max = 0, turnSum = 0, turnTime = 0, turnMax = 0 }
            self.stats[i] = s
            s.now = err
            -- Joining the slot at the start isn't keeping it: measured once it has been within a spacing.
            if s.joined or err < 1 then
                s.joined = true
                s.sum, s.time, s.max = s.sum + err * dt, s.time + dt, math.max(s.max, err)
                if turning then s.turnSum, s.turnTime, s.turnMax = s.turnSum + err * dt, s.turnTime + dt, math.max(s.turnMax, err) end
            end
        end
    end
end

---@return string[]
function FormationTestbed:lines()
    local lead = self.ships[1]:get(PhysicsComponents.RigidBody):getRigidBody()
    local mps = 1 / ScaleConfig:toGameUnits(1, "starSystem")
    local lines = { string.format("Formation testbed: %s, leader at %.0f m/s, turning %.3f rad/s (F: next shape)",
        FormationDefs[SHAPES[self.shapeIndex]].name, lead:getVelocity():length() * mps, lead:getVelocityA():length()) }
    for i = 2, #self.ships do
        local s = self.stats[i]
        if s then
            local member = self.ships[i]:get(Formations.FormationMember)
            local body = self.ships[i]:get(PhysicsComponents.RigidBody):getRigidBody()
            local dv = body:getVelocity() - lead:getVelocity()
            lines[#lines + 1] = string.format("Wing %d: off its slot %.2f now | %.2f mean, %.2f max | turns %.2f mean, %.2f max (spacings of %.0f m) | %.0f m/s off the leader's velocity | %s",
                i - 1, s.now or 0, s.time > 0 and s.sum / s.time or 0, s.max, s.turnTime > 0 and s.turnSum / s.turnTime or 0, s.turnMax,
                member and member.spacingM or 0, dv:length() * mps, member and member.order and self:orderText(member.order) or "-")
        end
    end
    return lines
end

---A wingman's order: its status, and whether it tracks its slot or flies a plan to it.
---@param order Entity
---@return string
function FormationTestbed:orderText(order)
    local progress = Nav.progress(order)
    local mps = 1 / ScaleConfig:toGameUnits(1, "starSystem")
    return string.format("status %s, %s, %.0f m left, speed %.0f m/s", tostring(Nav.status(order)),
        progress and progress.nextWaypoint and "flying a plan" or "tracking",
        progress and progress.distanceLeft and progress.distanceLeft * mps or -1,
        order:get(Nav.Components.NavOrder) and 0 or 0)
end

---@param data EventData
function FormationTestbed:onSim(data)
    local advance = math.min(data:deltaTime(), MAX_ADVANCE_S)
    local steps = math.max(1, math.ceil(advance / SIM_STEP_S))
    for _ = 1, steps do
        local dt = advance / steps
        self.simSteps = (self.simSteps or 0) + 1
        TravelDriveSystem:update(dt)
        ShipFlightSystem:update()
        self.world:update(dt)
    end
end

---@param data EventData
function FormationTestbed:onInput(data)
    if self:handleAppInput() then return end
    if self.started and Input:isPressed(Button.KeyboardF) then
        self.shapeIndex = self.shapeIndex % #SHAPES + 1
        self.formation:get(Formations.Formation):setShape(SHAPES[self.shapeIndex])
        self.stats = {}
    end
end

---@param data EventData
function FormationTestbed:onRender(data)
    CelestialLightingSystem:update(self.star)
    RenderCoreSystem:render(data)
    local lines = self.started and self:lines() or { "Formation testbed: starting" }
    self:immediateUI(function()
        for i, line in ipairs(lines) do
            DrawEx.TextAdditive(HUD_FONT, line, HUD_FONT_SIZE, HUD_X, HUD_Y + (i - 1) * HUD_LINE, 40, 20,
                0.9, 0.9, 0.9, 0.9, 0.0, 0.5)
        end
    end)
end

return FormationTestbed
