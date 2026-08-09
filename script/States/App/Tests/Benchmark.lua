-- Benchmark: scripted, reproducible performance test state.
--
-- Reuses PlanetTest's proven scene (planet + ring + moons + skybox) via
-- subclassing, then overrides the interactive bits:
--   - onStateInput: disabled (no human input -> deterministic)
--   - updateOrbitCamera: fixed-rate orbit (no drag/zoom)
--   - frame-time logging: BENCH lines every BENCH_LOG_INTERVAL frames,
--     skipping BENCH_WARMUP_FRAMES (generation + GC settle)
--
-- Same seed every run => same system => same camera path => comparable
-- before/after measurements.
--
-- Launch: cargo run -p ltr --features stats-server -- --stats-server 8777 Benchmark
-- Stats:   curl http://127.0.0.1:8777/stats.json  +  /profile.json
-- Log:     grep BENCH <logfile>
local PlanetTest = require('States.App.Tests.PlanetTest')

local CameraManager       = require("Modules.Cameras.Managers.CameraManager")
local PhysicsComponents   = require("Modules.Physics.Components")
local RenderCoreSystem    = require("Modules.Rendering.Systems.RenderCoreSystem")

local Benchmark = Subclass("Benchmark", PlanetTest)

-- Deterministic path. Tune to stress the render path.
local BENCH_ORBIT_RADIUS  = 800       -- camera orbit radius (planet radius * factor)
local BENCH_ORBIT_SPEED   = 0.05      -- rad/s (slow, steady load)
local BENCH_WARMUP_FRAMES = 180       -- skip logging (generation + GC settle)
local BENCH_LOG_INTERVAL  = 120       -- frames between BENCH log lines

function Benchmark:onInit()
    PlanetTest.onInit(self)

    -- PlanetTest.onInit uses RNG.FromTime(); force a fixed seed so every
    -- run is the same system. (Scene is already built by PlanetTest.onInit:
    -- planet + ring + moons + skybox.)
    self.seed = 0
    self.rng = RNG.Create(0)

    -- Deterministic orbit camera around the planet (override PlanetTest's
    -- interactive orbit defaults: no drag, no zoom, fixed radius).
    self.orbitAngle = 0
    self.orbitPitch = 0.2
    self.orbitRadius = BENCH_ORBIT_RADIUS
    self.autoRotationSpeed = BENCH_ORBIT_SPEED
    self.isDragging = false
    self.dragReleaseTimer = 0
    self:updateOrbitCamera(0)

    -- Frame accounting
    self.frameCount = 0
end

-- Input is a benchmark: no human interaction, no camera drag.
function Benchmark:onStateInput(data)
    -- intentionally empty (PlanetTest's input handler is not called)
end

-- Camera: fixed-rate orbit only (override PlanetTest's interactive zoom/
-- drag/transition logic with a pure deterministic path).
function Benchmark:updateOrbitCamera(dt)
    local targetPos = self.planetPos

    local x = math.sin(self.orbitAngle) * math.cos(self.orbitPitch) * self.orbitRadius
    local y = math.sin(self.orbitPitch) * self.orbitRadius
    local z = math.cos(self.orbitAngle) * math.cos(self.orbitPitch) * self.orbitRadius
    local camPos = Vec3f(targetPos.x + x, targetPos.y + y, targetPos.z + z)

    local camEntity = CameraManager:getActiveCameraEntity()
    if not camEntity then return end
    local transform = camEntity:get(PhysicsComponents.Transform)
    if not transform then return end

    transform:setPos(Position(camPos.x, camPos.y, camPos.z))
    local lookDir = (targetPos - camPos):normalize()
    transform:setRot(Quat.FromLook(lookDir, Vec3f(0, 1, 0)))
end

function Benchmark:onStatePreRender(data)
    local dt = data:deltaTime()
    self.timer:update(dt)

    self.orbitAngle = self.orbitAngle + BENCH_ORBIT_SPEED * dt
    self:updateOrbitCamera(dt)

    self.frameCount = self.frameCount + 1
    if self.frameCount > BENCH_WARMUP_FRAMES and self.frameCount % BENCH_LOG_INTERVAL == 0 then
        local ft = RenderCoreSystem:getSmoothFrameTime(true)
        Log.Info("BENCH frame=%d frametime=%.2f ms fps=%.1f mem=%.0f KB",
            self.frameCount, ft, 1000.0 / math.max(ft, 0.001), GC.GetMemory())
    end
end

function Benchmark:onRender(data)
    RenderCoreSystem:render(data)

    self:immediateUI(function()
        local ft = RenderCoreSystem:getSmoothFrameTime(true)
        UI.DrawEx.TextAdditive('Unageo-Medium',
            string.format("BENCH %d | %.2f ms | %.1f FPS | mem %.0f KB",
                self.frameCount, ft, 1000.0 / math.max(ft, 0.001), GC.GetMemory()),
            11, 40, 40, 40, 20, 0.9, 0.9, 0.9, 0.9, 0.0, 0.5)
    end)
end

return Benchmark
