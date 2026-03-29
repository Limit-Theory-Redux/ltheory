local Application             = require('States.Application')

---@class SolarSystemPlayable: Application
local SolarSystemPlayable     = Subclass("SolarSystemPlayable", Application)

local Registry                = require("Core.ECS.Registry")
local DeltaTimer              = require("Shared.Tools.DeltaTimer")

-- ECS Components
local CoreComponents          = require("Modules.Core.Components")
local PhysicsComponents       = require("Modules.Physics.Components")
local ConstructComponents     = require("Modules.Constructs.Components")

-- Camera
local CameraManager           = require("Modules.Cameras.Managers.CameraManager")

-- Systems
local RenderCoreSystem        = require("Modules.Rendering.Systems.RenderCoreSystem")
local PlayerController        = require("Modules.Constructs.Systems.PlayerController")
local AutoPilotSystem         = require("Modules.Constructs.Systems.AutoPilotSystem")
local GravityWellSystem       = require("Modules.Physics.Systems.GravityWellSystem")

-- Extracted systems
local CelestialLightingSystem = require("Modules.Rendering.Systems.CelestialLightingSystem")
local LensFlareSystem         = require("Modules.Rendering.Systems.LensFlareSystem")
local GameplayHUDSystem       = require("Modules.UI.Systems.GameplayHUDSystem")
local WorldLabelRenderSystem  = require("Modules.UI.Systems.WorldLabelRenderSystem")
local CoordinateRebaser       = require("Modules.CelestialObjects.Managers.CoordinateRebaser")

-- Generation + Simulation
local Rulesets                = require("Config.Gen.Rulesets")
local UniverseManager         = require("Modules.CelestialObjects.Managers.UniverseManager")
local SolarSystemVisualizer   = require("Modules.CelestialObjects.Managers.SolarSystemVisualizer")
local OrbitalSystem           = require("Modules.CelestialObjects.Systems.OrbitalSystem")
local SystemMap               = require("Modules.CelestialObjects.Systems.SystemMap")
local SystemMap3D             = require("Modules.CelestialObjects.Systems.SystemMap3D")

-- Generators
local ShipGenerator           = require("Modules.Constructs.Managers.Generators.ShipGenerator")
local StationGenerator        = require("Modules.Constructs.Managers.Generators.StationGenerator")

-- Entities
local SkyboxEntity            = require("Modules.CelestialObjects.Entities.SkyboxEntity")

-- Legacy (still needed for skybox generation)
local Generator               = require("Legacy.Systems.Gen.Generator")
local Starfield               = require("Legacy.Systems.Gen.Starfield")

function SolarSystemPlayable:onInit()
    self.cfg = Config.game.solarSystemPlayable

    Window:setPresentMode(PresentMode.NoVsync)
    Window:setFullscreen(false, true)

    self.seed = self.cfg.defaultSeed
    self.rng = RNG.Create(self.seed)
    self.timer = DeltaTimer("SolarSystemPlayable")

    self.world = Physics.Create()

    self:createSkybox()
    self:generateSolarSystem()
    self:spawnPlayerShip()
    self:spawnStations()

    -- Collect orbiting bodies for orbital simulation
    self.orbiters = OrbitalSystem:collectOrbiters(self.universe)
    Log.Info("Found %d orbiting bodies", #self.orbiters)

    -- Collect gravity well zones
    GravityWellSystem:collectZones(self.universe)

    -- System maps (2D and 3D)
    self.mapState = SystemMap:create()
    SystemMap:collectEntities(self.mapState, self.universe, self.playerShip)
    self.map3DState = SystemMap3D:create()
    SystemMap3D:collectEntities(self.map3DState, self.universe, self.playerShip)
    SystemMap3D:setSkybox(self.map3DState, self.skybox)
    self.mapMode = 0  -- 0=off, 1=2D, 2=3D

    -- Player controller owns camera modes, input routing, and ship reference
    self.player = PlayerController(self.playerShip, {
        chase = {
            distance     = self.cfg.orbitDistance,
            minDistance   = self.cfg.orbitMinDistance,
            maxDistance   = self.cfg.orbitMaxDistance,
            zoomSpeed    = self.cfg.orbitZoomSpeed,
            smoothing    = self.cfg.orbitSmoothing,
        },
        free = {
            moveSpeed      = self.cfg.freeMoveSpeed,
            fastMultiplier = self.cfg.freeFastMult,
            mouseSensitivity = self.cfg.freeMouseSens,
        },
    })

    EventBus:subscribe(Event.Sim, self, self.onStateSim)
end

function SolarSystemPlayable:createSkybox()
    self.skybox = SkyboxEntity(self.seed, function(entity, blendMode)
        local placeholder = entity:get(CoreComponents.Empty)
        if not placeholder then
            placeholder = entity:add(CoreComponents.Empty)
        end

        if not placeholder.envMap then
            require("Legacy.Systems.Gen.Nebula.Nebula1")
            local nebulaRNG     = RNG.Create(entity:get(CoreComponents.Seed):getSeed() + 0xC0104FULL)
            local starAngle     = nebulaRNG:getDir2()
            placeholder.starDir = Vec3f(starAngle.x, 0, starAngle.y)
            placeholder.envMap  = Generator.Get('Nebula', nebulaRNG)(nebulaRNG, Config.gen.nebulaRes, placeholder.starDir)
            placeholder.irMap   = placeholder.envMap:genIRMap(256)
            placeholder.stars   = Starfield(nebulaRNG, Config.gen.nStars(nebulaRNG))
            ShaderVar.PushFloat3('starDir', placeholder.starDir.x, placeholder.starDir.y, placeholder.starDir.z)
            ShaderVar.PushTexCube('envMap', placeholder.envMap)
            ShaderVar.PushTexCube('irMap', placeholder.irMap)
        end

        if blendMode == BlendMode.Disabled then
            RenderState.PushDepthWritable(false)
            local shader = Cache.Shader('farplane', 'skybox')
            RenderState.PushCullFace(CullFace.None)
            shader:start()
            Draw.Box3(Box3f(-1, -1, -1, 1, 1, 1))
            shader:stop()
            RenderState.PopCullFace()
            RenderState.PopDepthWritable()
        elseif blendMode == BlendMode.Additive then
            local shader = Cache.Shader('farplane', 'starbg')
            shader:start()
            shader:setFloat('brightnessScale', 3)
            shader:setTexCube('irMap', placeholder.irMap)
            shader:setTexCube('envMap', placeholder.envMap)
            placeholder.stars:draw()
            shader:stop()
        end
    end)
end

function SolarSystemPlayable:generateSolarSystem()
    Log.Info("Generating solar system with seed %d...", self.seed)
    self.universe = UniverseManager:createUniverse(Rulesets.StandardSolarSystem, self.seed)

    if not self.universe then
        Log.Error("Failed to generate solar system!")
        return
    end

    -- Rebase all entities to star-system-local coordinates (near origin)
    self.systemOrigin = CoordinateRebaser:rebaseStarSystem(self.universe)

    SolarSystemVisualizer:materialize(self.universe, self.world)

    self.planets = {}
    self.labeledEntities = {}
    self:_collectEntities(self.universe)
    Log.Info("Found %d planets in generated solar system", #self.planets)
end

function SolarSystemPlayable:_collectEntities(entity, parentPos)
    local name = tostring(entity)

    if name:find("PlanetEntity") then
        table.insert(self.planets, entity)
    end

    -- Collect all named celestial objects for labels
    local label = nil
    local isMoon = false
    if name:find("StarEntity") then
        label = "Star"
        self.starEntity = entity
    elseif name:find("PlanetEntity") then
        label = "Planet " .. #self.planets
    elseif name:find("MoonEntity") then
        label = "Moon"; isMoon = true
    elseif name:find("SpaceStationEntity") then
        label = "Station"
    end

    local entityPos = parentPos
    if label then
        local transform = entity:get(PhysicsComponents.Transform)
        if transform then
            local pos = transform:getPos()
            local scale = transform:getScale()
            entityPos = pos
            Log.Info("  %s at (%.0f, %.0f, %.0f) scale=%.0f", label, pos.x, pos.y, pos.z, scale)
            table.insert(self.labeledEntities, {
                entity = entity,
                label = label,
                isMoon = isMoon,
                parentPos = parentPos,
            })
        end
    end

    local childrenCmp = entity:get(CoreComponents.Children)
    if childrenCmp then
        for child in childrenCmp:iterChildren() do
            self:_collectEntities(child, entityPos)
        end
    end
end

function SolarSystemPlayable:spawnPlayerShip()
    local cfg = self.cfg
    local flightCfg = Config.game.shipFlight

    local spawnPos = Position(0, 0, 0)
    if #self.planets > 0 then
        local planet = self.planets[1]
        local transform = planet:get(PhysicsComponents.Transform)
        local planetPos = transform:getPos()
        local planetScale = transform:getScale()
        local offset = planetScale * cfg.shipSpawnOffset
        spawnPos = Position(planetPos.x + offset, planetPos.y + offset * 0.5, planetPos.z)
    end

    self.playerShip = ShipGenerator:createFighter(self.rng:get31(), {
        position = spawnPos,
        scale = cfg.shipScale,
        isKinematic = false,
    })

    -- Add ship-specific ECS components
    self.playerShip:add(ConstructComponents.TravelDrive())
    self.playerShip:add(ConstructComponents.AutoPilot())
    self.playerShip:add(ConstructComponents.ShipFlightControl())

    -- Apply hull-class physics (Solo = index 1)
    local hullIdx = 1
    local hullCfg = Config.game.shipHulls
    local rbCmp = self.playerShip:get(PhysicsComponents.RigidBody)
    local rb = rbCmp:getRigidBody()
    rb:setMass(hullCfg.mass[hullIdx])
    rb:setDrag(flightCfg.linearDrag, flightCfg.angularDrag)
    rb:setFriction(0)
    rb:setSleepThreshold(0, 0)
    self.world:addRigidBody(rb)

    Log.Info("Player ship spawned at (%.1f, %.1f, %.1f)",
        spawnPos.x, spawnPos.y, spawnPos.z)
end

function SolarSystemPlayable:spawnStations()
    self.stations = {}
    local cfg = self.cfg

    local numStations = math.min(cfg.maxStations, #self.planets)
    for i = 1, numStations do
        local planet = self.planets[i]
        local transform = planet:get(PhysicsComponents.Transform)
        local planetPos = transform:getPos()
        local planetScale = transform:getScale()

        local orbitDist = planetScale * cfg.stationOrbitMult
        local angle = (i - 1) * math.pi * 0.7
        local stationPos = Position(
            planetPos.x + math.cos(angle) * orbitDist,
            planetPos.y,
            planetPos.z + math.sin(angle) * orbitDist
        )

        local station = StationGenerator:createStation(self.rng:get31(), {
            position = stationPos,
            scale = cfg.stationScale,
            isKinematic = true,
        })

        local stationRbCmp = station:get(PhysicsComponents.RigidBody)
        if stationRbCmp and stationRbCmp:getRigidBody() then
            self.world:addRigidBody(stationRbCmp:getRigidBody())
        end

        table.insert(self.stations, station)
        table.insert(self.labeledEntities, { entity = station, label = "Station " .. i })
        Log.Info("Station %d spawned near planet %d", i, i)
    end
end

---@param data EventData
function SolarSystemPlayable:onRender(data)
    CelestialLightingSystem:update(self.starEntity)

    if self.mapMode == 2 then
        -- 3D map: render through proper pipeline with map camera
        SystemMap3D:collectEntities(self.map3DState, self.universe, self.playerShip)
        SystemMap3D:render(self.map3DState, data)

        self:immediateUI(function()
            LensFlareSystem:draw(self.starEntity, self.labeledEntities)
            SystemMap3D:renderOverlay(self.map3DState, 0, 0, Window:width(), Window:height())
        end)
    else
        -- Normal scene
        RenderCoreSystem:render(data)

        self:immediateUI(function()
            LensFlareSystem:draw(self.starEntity, self.labeledEntities)
            GameplayHUDSystem:draw(self.playerShip, self.player:getModeName(), self.player:isPiloting())
            WorldLabelRenderSystem:draw(self.labeledEntities)

            if self.mapMode == 1 then
                SystemMap:collectEntities(self.mapState, self.universe, self.playerShip)
                SystemMap:draw(self.mapState, 0, 0, Window:width(), Window:height())
            end
        end)
    end
end

---@param data EventData
function SolarSystemPlayable:onInput(data)
    -- Cycle map mode: off -> 2D -> 3D -> off
    if Input:isPressed(Button.KeyboardM) then
        self.mapMode = (self.mapMode + 1) % 3
        self.mapState.enabled = (self.mapMode == 1)
        self.map3DState.enabled = (self.mapMode == 2)

        if self.mapMode == 1 then
            SystemMap:collectEntities(self.mapState, self.universe, self.playerShip)
            Input:setCursorVisible(true)
        elseif self.mapMode == 2 then
            SystemMap3D:collectEntities(self.map3DState, self.universe, self.playerShip)
            SystemMap3D:activate(self.map3DState)
            Input:setCursorVisible(true)
        else
            SystemMap3D:deactivate(self.map3DState)
            self.mapClosedFrame = true
        end
    end

    -- Handle deferred map close
    if self.mapClosedFrame and self.mapMode == 0 then
        self.mapClosedFrame = false
        self.player:setMode(self.player:getMode())
    end

    -- When any map is open: block ship/camera input, process map input
    if self.mapMode > 0 then
        local ctrl = self.player:getActiveController()
        if ctrl and ctrl:isEnabled() then ctrl:disable() end
        self.player:setInputBlocked(true)

        if self.mapMode == 1 then
            SystemMap:updateInput(self.mapState, data:deltaTime())

            -- N key: navigate to selected entity on 2D map
            if Input:isPressed(Button.KeyboardN) and self.mapState.selected then
                AutoPilotSystem:engageEntity(self.playerShip, self.mapState.selected.entity)
                self.mapMode = 0
                self.mapState.enabled = false
                self.mapClosedFrame = true
            end
        else
            SystemMap3D:updateInput(self.map3DState, data:deltaTime())

            -- N key: navigate to selected entity on 3D map
            if Input:isPressed(Button.KeyboardN) and self.map3DState.selected then
                AutoPilotSystem:engageEntity(self.playerShip, self.map3DState.selected.entity)
                self.mapMode = 0
                self.map3DState.enabled = false
                SystemMap3D:deactivate(self.map3DState)
                self.mapClosedFrame = true
            end
        end
        return
    else
        self.player:setInputBlocked(false)
    end

    -- N key: toggle autopilot (cancel if active)
    if Input:isPressed(Button.KeyboardN) then
        if AutoPilotSystem:isActive(self.playerShip) then
            AutoPilotSystem:disengage(self.playerShip)
        end
    end

    -- Regenerate with new seed
    if Input:isPressed(Button.KeyboardB) then
        self.seed = self.rng:get31()
        self:regenerate()
    end
end

---@param data EventData
function SolarSystemPlayable:onStateSim(data)
    local dt = data:deltaTime()

    OrbitalSystem:update(self.orbiters, dt)
    GravityWellSystem:update(dt, self.playerShip)
    SystemMap3D:updateTrails(self.map3DState, dt)

    self.player:update(dt)
    self.world:update(dt)
end

function SolarSystemPlayable:regenerate()
    if self.universe then
        Registry:destroyEntity(self.universe, Enums.Registry.EntityDestroyMode.DestroyChildren)
    end
    if self.playerShip then
        local rbCmp = self.playerShip:get(PhysicsComponents.RigidBody)
        if rbCmp and rbCmp:getRigidBody() then
            self.world:removeRigidBody(rbCmp:getRigidBody())
        end
        Registry:destroyEntity(self.playerShip, Enums.Registry.EntityDestroyMode.DestroyChildren)
    end
    for _, station in ipairs(self.stations or {}) do
        local rbCmp = station:get(PhysicsComponents.RigidBody)
        if rbCmp and rbCmp:getRigidBody() then
            self.world:removeRigidBody(rbCmp:getRigidBody())
        end
        Registry:destroyEntity(station, Enums.Registry.EntityDestroyMode.DestroyChildren)
    end

    self.starEntity = nil
    self.rng = RNG.Create(self.seed)
    self:generateSolarSystem()
    self:spawnPlayerShip()
    self:spawnStations()
    self.player:setShip(self.playerShip)

    -- Rebuild orbital + map + gravity well data
    self.orbiters = OrbitalSystem:collectOrbiters(self.universe)
    GravityWellSystem:collectZones(self.universe)
    SystemMap:collectEntities(self.mapState, self.universe, self.playerShip)
    SystemMap3D:collectEntities(self.map3DState, self.universe, self.playerShip)
end

return SolarSystemPlayable
