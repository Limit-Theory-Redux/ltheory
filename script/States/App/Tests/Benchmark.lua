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
local AsteroidFieldSystem = require("Modules.CelestialObjects.Systems.AsteroidFieldSystem")
local AsteroidBeltRenderer = require("Modules.CelestialObjects.Systems.AsteroidBeltRenderer")
local AsteroidMeshPool    = require("Modules.CelestialObjects.Systems.AsteroidMeshPool")
local CelestialComponents = require("Modules.CelestialObjects.Components")
local CoreComponents      = require("Modules.Core.Components")
local RenderComp          = require("Modules.Rendering.Components").Render
local Entity              = require("Core.ECS.Entity")

local Benchmark = Subclass("Benchmark", PlanetTest)

-- Deterministic path. Tune to stress the render path.
local BENCH_ORBIT_RADIUS  = 800       -- camera orbit radius (planet radius * factor)
local BENCH_ORBIT_SPEED   = 0.05      -- rad/s (slow, steady load)
local BENCH_WARMUP_FRAMES = 180       -- skip logging (generation + GC settle)
local BENCH_LOG_INTERVAL  = 120       -- frames between BENCH log lines
local BENCH_BELT_COUNT    = 4000      -- asteroids in the belt (ECS spawns near camera)
local BENCH_ORBIT_TIME    = 15.0      -- s of orbit before zooming to an asteroid
local BENCH_ZOOM_TIME     = 4.0       -- s to fly from orbit to the asteroid
local BENCH_LOOK_TIME     = 6.0       -- s holding the close-up on the asteroid
local BENCH_RETURN_TIME   = 4.0       -- s to fly back to orbit
local BENCH_MOON_ZOOM_TIME = 4.0      -- s to fly from orbit to the moon
local BENCH_MOON_LOOK_TIME = 6.0      -- s holding the close-up on the moon
local BENCH_LOOK_SPIN     = -0.15     -- rad/s: turntable direction (negative = clockwise)
local BENCH_ZOOM_DIST_F   = 3.0       -- camera distance = asteroid scale * this
local BENCH_ZOOM_DIST_MIN = 5.0       -- ... but never closer than this
local BENCH_MOON_DIST_F  = 2.0       -- camera distance = moon scale * this
local BENCH_MOON_DIST_MIN = 4.0      -- ... but never closer than this
local BENCH_ZOOM_PITCH    = 0.3       -- viewing angle at the close-up
local BENCH_PLANET_CLEAR  = 1.6       -- min camera distance from planet center (x radius)
local BENCH_SPAWN_TOTAL   = 100       -- max concurrent spawned asteroid entities (game default)
local BENCH_SPAWN_PER_UPDATE = 5     -- max new entities per spawn check (game default)
local BENCH_BELT_DRAWN    = 20000     -- max belt-rendered asteroids per frame (render them ALL)

function Benchmark:onInit()
    PlanetTest.onInit(self)

    -- Match the game's real ECS spawn caps (100 concurrent, 5/update).
    -- Belt DATA stays dense (thousands of asteroids - realistic for a
    -- belt, per SolarSystemVisualizer's count formula). The per-frame
    -- draw limit is set to render ALL belt asteroids: instancing exists
    -- to draw the whole belt, not a 200-rock sample (see the planetary
    -- rings work). Extend the belt render distance: the camera orbits at
    -- BENCH_ORBIT_RADIUS far outside the belt, so the game's
    -- belt-spread-derived cutoff would cull everything.
    AsteroidFieldSystem.setSpawnCaps(BENCH_SPAWN_TOTAL, BENCH_SPAWN_PER_UPDATE)
    AsteroidBeltRenderer.setMaxDrawnPerFrame(BENCH_BELT_DRAWN)
    AsteroidBeltRenderer.setRenderDistSq((BENCH_ORBIT_RADIUS + 1500) ^ 2)

    -- PlanetTest.onInit uses RNG.FromTime() and creates the ring with only
    -- 20% probability; force a fixed seed and ALWAYS create the ring so the
    -- benchmark scene is deterministic and includes the ring (the visual
    -- anchor for judging render quality).
    self.seed = 0
    self.rng = RNG.Create(0)

    if not self.ring then
        self:createPlanetRing(12345)
    end
    if not self.moons then
        self:createMoons(0, 3)
    end

    -- Asteroid belt: ECS entities spawn near the camera as it orbits,
    -- stressing the instanced asteroid render path. The belt sits INSIDE
    -- the planet ring band (same radii, same tilt against the planet axis)
    -- so asteroids visually belong to the ring.
    self.beltEntities = {}
    local beltEntity = Entity.Create("AsteroidBeltEntity", CoreComponents.Seed(12345))
    local beltTransform = beltEntity:get(PhysicsComponents.Transform)
    if beltTransform then beltTransform:setPos(Position(0, 0, 0)) end
    local lodMesh = AsteroidMeshPool:getFromSeed(12345)

    -- Ring geometry (stored by createPlanetRing): belt fills the ring band
    local beltInner = self.ringInnerRadius or 200
    local beltOuter = self.ringOuterRadius or beltInner + 200
    local beltRadius = (beltInner + beltOuter) * 0.5
    local beltWidth = math.max(beltOuter - beltInner, 100)
    local beltTilt = self.ringTiltRad or math.rad(25)

    local asteroids = AsteroidBeltRenderer.generateBeltAsteroids({
        orbitRadius = beltRadius,
        width = beltWidth,
        count = BENCH_BELT_COUNT,
        inclination = beltTilt,
        seed = 12345,
        minScale = 1.0,
        maxScale = 6.0,
    })
    beltEntity:add(CelestialComponents.AsteroidBelt(asteroids, beltRadius, beltWidth, lodMesh))
    local renderCmp = beltEntity:get(RenderComp)
    if not renderCmp then
        renderCmp = RenderComp()
        beltEntity:add(renderCmp)
    end
    renderCmp:setRenderFn(AsteroidBeltRenderer.createRenderFn(asteroids, lodMesh))
    renderCmp:setVisible(true)
    table.insert(self.beltEntities, beltEntity)

    -- Deterministic orbit camera around the planet (override PlanetTest's
    -- interactive orbit defaults: no drag, no zoom, fixed radius).
    -- Scale the orbit radius to the scene: sit just outside the belt so
    -- asteroids are big in frame (a fixed 800 would leave them tiny dots).
    self.orbitAngle = 0
    self.orbitPitch = 0.2
    self.orbitRadius = BENCH_ORBIT_RADIUS
    if self.ringOuterRadius then
        self.orbitRadius = self.ringOuterRadius * 1.6
    end
    self.autoRotationSpeed = BENCH_ORBIT_SPEED
    self.isDragging = false
    self.dragReleaseTimer = 0

    -- Multi-scene camera state machine
    self.phase = "orbit"
    self.phaseTime = 0
    self.planetRadius = self.planet:get(PhysicsComponents.RigidBody):getRadius() or 100
    self:setCameraPosRot(self:orbitCamPos(), self.planetPos)

    -- Frame accounting
    self.frameCount = 0
end

-- Wrap PlanetTest's ring creation so the belt can match its geometry.
function Benchmark:createPlanetRing(seed)
    PlanetTest.createPlanetRing(self, seed)

    -- Ring band geometry + tilt (for the asteroid belt to sit inside)
    local planetRadius = self.planet:get(PhysicsComponents.RigidBody):getRadius()
    self.ringInnerRadius = planetRadius + planetRadius * 0.65
    self.ringOuterRadius = self.ringOuterRadius or self.ringInnerRadius + planetRadius
    self.ringTiltRad = self.ringTiltRad or math.rad(25)
end

-- Input is a benchmark: no human interaction, no camera drag.
function Benchmark:onStateInput(data)
    -- intentionally empty (PlanetTest's input handler is not called)
end

-- Camera rig helpers -------------------------------------------------------
function Benchmark:setCameraPosRot(pos, lookAt)
    local camEntity = CameraManager:getActiveCameraEntity()
    if not camEntity then return end
    local transform = camEntity:get(PhysicsComponents.Transform)
    if not transform then return end

    transform:setPos(Position(pos.x, pos.y, pos.z))
    local lookDir = (lookAt - pos):normalize()
    transform:setRot(Quat.FromLook(lookDir, Vec3f(0, 1, 0)))
end

-- Orbit position around the planet at the current orbit angle/pitch/radius.
function Benchmark:orbitCamPos()
    local x = math.sin(self.orbitAngle) * math.cos(self.orbitPitch) * self.orbitRadius
    local y = math.sin(self.orbitPitch) * self.orbitRadius
    local z = math.cos(self.orbitAngle) * math.cos(self.orbitPitch) * self.orbitRadius
    return Vec3f(self.planetPos.x + x, self.planetPos.y + y, self.planetPos.z + z)
end

-- Fly camera from `from` to `to` over `duration` seconds, easing in/out.
-- The look target blends from `lookFrom` to `lookTo` with the same easing
-- so the view direction glides instead of snapping. The path arcs around
-- the planet: any point closer than BENCH_PLANET_CLEAR * planetRadius is
-- pushed radially out to that distance, so the flight never crosses the
-- planet. Returns true once the flight is complete.
function Benchmark:flyTo(from, to, lookFrom, lookTo, duration)
    local t = math.min(1.0, self.phaseTime / duration)
    local e = self:easeInOutCubic(t)

    local pos = Vec3f(
        from.x + (to.x - from.x) * e,
        from.y + (to.y - from.y) * e,
        from.z + (to.z - from.z) * e
    )

    -- Arc around the planet: radial push-out for points inside the clearance
    local r = pos - self.planetPos
    local len = r:length()
    local minDist = self.planetRadius * BENCH_PLANET_CLEAR
    if len < minDist and len > 0.0001 then
        r = r * (minDist / len)
        pos = self.planetPos + r
    end

    -- Blended look target: planet -> asteroid during the flight
    local look = Vec3f(
        lookFrom.x + (lookTo.x - lookFrom.x) * e,
        lookFrom.y + (lookTo.y - lookFrom.y) * e,
        lookFrom.z + (lookTo.z - lookFrom.z) * e
    )
    self:setCameraPosRot(pos, look)
    return t >= 1.0
end

-- Pick a random spawned asteroid as the zoom target. Prefers larger ones
-- (they read better at close-up). Returns { pos=Vec3f, scale=number } or nil.
function Benchmark:pickZoomTarget()
    local spawned = AsteroidFieldSystem:getSpawnedEntities()
    if not spawned or #spawned == 0 then return nil end

    -- Try a few times for a decent-size asteroid; fall back to random
    local best, bestScale = nil, 0
    for attempt = 1, 8 do
        local idx = self.rng:getInt(1, #spawned)
        local a = spawned[idx]
        if a and a.pos then
            local s = a.scale or 1
            if s > bestScale then
                bestScale = s
                best = a
            end
            if s >= 2.0 then break end
        end
    end
    if not best then return nil end
    -- a.pos is a Position (world coords); convert to Vec3f for camera math
    return {
        pos = Vec3f(best.pos.x, best.pos.y, best.pos.z),
        scale = bestScale,
    }
end

-- Pick the first moon as the zoom target. Returns { pos=Vec3f, scale=number }
-- or nil if no moons exist.
function Benchmark:pickMoonTarget()
    if not self.moons or #self.moons == 0 then return nil end
    local moon = self.moons[1]
    local rbCmp = moon.entity:get(PhysicsComponents.RigidBody)
    if not rbCmp then return nil end
    local rb = rbCmp:getRigidBody()
    if not rb then return nil end
    local p = rb:getPos()
    local s = rb:getScale() or 1
    return {
        pos = Vec3f(p.x, p.y, p.z),
        scale = s,
    }
end

-- Multi-scene camera state machine ------------------------------------------
-- Phase "orbit":  circle the planet (steady wide view)
-- Phase "zoom":   fly to a random asteroid, holding a blended look
-- Phase "look":   slow orbit around the asteroid (see the mesh from angles)
-- Phase "return": fly back to the orbit ring
-- Then the same zoom/look/return cycle for the moon, then back to orbit.
function Benchmark:updateBenchCamera(dt)
    self.phaseTime = self.phaseTime + dt

    if self.phase == "orbit" then
        self.orbitAngle = self.orbitAngle + BENCH_ORBIT_SPEED * dt
        self:setCameraPosRot(self:orbitCamPos(), self.planetPos)

        if self.phaseTime >= BENCH_ORBIT_TIME then
            local target = self:pickZoomTarget()
            if target then
                self.zoomTarget = target
                self.zoomFrom = self:orbitCamPos()

                -- Distance proportional to asteroid size (so it fills the
                -- frame), with a floor so tiny rocks don't clip the near plane
                local dist = math.max(target.scale * BENCH_ZOOM_DIST_F, BENCH_ZOOM_DIST_MIN)
                self.zoomDist = dist

                -- Approach from the current orbit side, slightly above
                local dir = (target.pos - self.zoomFrom):normalize()
                local up = Vec3f(0, 1, 0)
                local side = dir:cross(up):normalize()
                local offset = side * dist + up * dist * 0.4
                self.zoomTo = target.pos + offset
                self.lookAngle = 0
                self.phase = "zoom"
                self.phaseTime = 0
            end
        end
    elseif self.phase == "zoom" then
        local done = self:flyTo(self.zoomFrom, self.zoomTo,
            self.planetPos, self.zoomTarget.pos, BENCH_ZOOM_TIME)
        if done then
            -- Start the turntable at the current horizontal direction so the
            -- zoom->look transition is seamless (no rotation snap)
            local dx = self.zoomTo.x - self.zoomTarget.pos.x
            local dz = self.zoomTo.z - self.zoomTarget.pos.z
            self.lookAngle = math.atan2(dz, dx)
            self.phase = "look"
            self.phaseTime = 0
        end
    elseif self.phase == "look" then
        -- Turntable: orbit the camera around the asteroid on a circle in the
        -- world XZ plane (world Y axis), always looking at the asteroid.
        -- This shows the mesh from all sides without tilting the view plane.
        self.lookAngle = self.lookAngle + BENCH_LOOK_SPIN * dt
        local offset = Vec3f(
            math.cos(self.lookAngle) * self.zoomDist,
            self.zoomDist * 0.4,   -- fixed height above the asteroid
            math.sin(self.lookAngle) * self.zoomDist
        )
        local pos = self.zoomTarget.pos + offset
        self:setCameraPosRot(pos, self.zoomTarget.pos)

        if self.phaseTime >= BENCH_LOOK_TIME then
            self.returnFrom = pos
            self.phase = "return"
            self.phaseTime = 0
        end
    elseif self.phase == "return" then
        self.orbitAngle = self.orbitAngle + BENCH_ORBIT_SPEED * dt
        local orbitPos = self:orbitCamPos()
        local done = self:flyTo(self.returnFrom, orbitPos,
            self.zoomTarget.pos, self.planetPos, BENCH_RETURN_TIME)
        if done then
            -- After the asteroid cycle, zoom to the moon
            local moonTarget = self:pickMoonTarget()
            if moonTarget then
                self.zoomTarget = moonTarget
                self.zoomFrom = orbitPos
                -- Distance proportional to moon size (moon scale = its
                -- radius), with a floor so we never sit inside it. 1.8x
                -- radius puts the camera just outside the surface.
                self.zoomDist = math.max(moonTarget.scale * BENCH_MOON_DIST_F, BENCH_MOON_DIST_MIN)

                local dir = (moonTarget.pos - self.zoomFrom):normalize()
                local up = Vec3f(0, 1, 0)
                local side = dir:cross(up):normalize()
                local offset = side * self.zoomDist + up * self.zoomDist * 0.3
                self.zoomTo = moonTarget.pos + offset
                self.lookAngle = 0
                self.phase = "moon-zoom"
                self.phaseTime = 0
            else
                self.phase = "orbit"
                self.phaseTime = 0
            end
        end
    elseif self.phase == "moon-zoom" then
        -- Follow the moon's live position (it orbits the planet)
        local live = self:pickMoonTarget()
        if live then self.zoomTarget.pos = live.pos end
        local done = self:flyTo(self.zoomFrom, self.zoomTo,
            self.planetPos, self.zoomTarget.pos, BENCH_MOON_ZOOM_TIME)
        if done then
            local dx = self.zoomTo.x - self.zoomTarget.pos.x
            local dz = self.zoomTo.z - self.zoomTarget.pos.z
            self.lookAngle = math.atan2(dz, dx)
            self.phase = "moon-look"
            self.phaseTime = 0
        end
    elseif self.phase == "moon-look" then
        -- Follow the moon's live position, then turntable around it
        local live = self:pickMoonTarget()
        if live then self.zoomTarget.pos = live.pos end
        self.lookAngle = self.lookAngle + BENCH_LOOK_SPIN * dt
        local offset = Vec3f(
            math.cos(self.lookAngle) * self.zoomDist,
            self.zoomDist * 0.3,   -- fixed height above the moon
            math.sin(self.lookAngle) * self.zoomDist
        )
        local pos = self.zoomTarget.pos + offset
        self:setCameraPosRot(pos, self.zoomTarget.pos)

        if self.phaseTime >= BENCH_MOON_LOOK_TIME then
            self.returnFrom = pos
            self.phase = "moon-return"
            self.phaseTime = 0
        end
    elseif self.phase == "moon-return" then
        local live = self:pickMoonTarget()
        if live then self.zoomTarget.pos = live.pos end
        self.orbitAngle = self.orbitAngle + BENCH_ORBIT_SPEED * dt
        local orbitPos = self:orbitCamPos()
        local done = self:flyTo(self.returnFrom, orbitPos,
            self.zoomTarget.pos, self.planetPos, BENCH_RETURN_TIME)
        if done then
            self.phase = "orbit"
            self.phaseTime = 0
        end
    end
end

function Benchmark:onStatePreRender(data)
    local dt = data:deltaTime()
    self.timer:update(dt)

    self:updateBenchCamera(dt)

    self.frameCount = self.frameCount + 1
    if self.frameCount > BENCH_WARMUP_FRAMES and self.frameCount % BENCH_LOG_INTERVAL == 0 then
        local ft = RenderCoreSystem:getSmoothFrameTime(true)
        Log.Info("BENCH frame=%d phase=%s frametime=%.2f ms fps=%.1f mem=%.0f KB",
            self.frameCount, self.phase, ft, 1000.0 / math.max(ft, 0.001), GC.GetMemory())
    end
end

function Benchmark:onStateSim(data)
    PlanetTest.onStateSim(self, data)

    -- Asteroid field: rate-limited spawn/despawn near the camera
    AsteroidFieldSystem:update(data:deltaTime(), self.beltEntities, self.world)
    AsteroidFieldSystem:updatePositions()
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
