-- LTheoryRedux: the main game state. Flow: main menu (HmGui) -> New Game
-- -> startGame(seed) builds the scene through UniverseManager + Rulesets
-- + SolarSystemVisualizer, then renders via RenderCoreSystem with HUD,
-- system maps, and player ship control.
Namespace.LoadInline('Legacy')
Namespace.LoadInline('Legacy.Systems')

LimitTheoryRedux = require('States.Application')

-- Sound / persistence
local SoundManager = require('Legacy.Systems.SFX.SoundManager')
local MusicPlayer = require('Legacy.Systems.SFX.MusicPlayer')
local InitFiles = require('Legacy.Systems.Files.InitFiles')

-- UI pages (HmGui). The main menu is the only HmGui page; in-game UI is
-- drawn by the state's onRender through the rendering systems.
local UIRouter = require('UI.HmGui.UICore.UIRouter')
local UIPageMainMenu = require('UI.HmGui.Pages.MainMenu')

-- ECS registry, physics/construct/celestial components, camera manager,
-- render core, player controller, and the simulation systems the scene
-- runs on.
local Registry = require('Core.ECS.Registry')
local CoreComponents = require('Modules.Core.Components')
local PhysicsComponents = require('Modules.Physics.Components')
local ConstructComponents = require('Modules.Constructs.Components')
local CameraManager = require('Modules.Cameras.Managers.CameraManager')
local RenderCoreSystem = require('Modules.Rendering.Systems.RenderCoreSystem')
local PlayerController = require('Modules.Constructs.Systems.PlayerController')
local AutoPilotSystem = require('Modules.Constructs.Systems.AutoPilotSystem')
local GravityWellSystem = require('Modules.Physics.Systems.GravityWellSystem')
local CelestialLightingSystem = require('Modules.Rendering.Systems.CelestialLightingSystem')
local LensFlareSystem = require('Modules.Rendering.Systems.LensFlareSystem')
local GameplayHUDSystem = require('Modules.UI.Systems.GameplayHUDSystem')
local WorldLabelRenderSystem = require('Modules.UI.Systems.WorldLabelRenderSystem')
local AsteroidFieldSystem = require('Modules.CelestialObjects.Systems.AsteroidFieldSystem')
local GeneralActions = require('Input.ActionBindings.GeneralActions')
local CursorManager = require('Input.CursorManager')
local Rulesets = require('Config.Gen.Rulesets')
local UniverseManager = require('Modules.CelestialObjects.Managers.UniverseManager')
local SolarSystemVisualizer = require('Modules.CelestialObjects.Managers.SolarSystemVisualizer')
local CoordinateRebaser = require('Modules.CelestialObjects.Managers.CoordinateRebaser')
local OrbitalSystem = require('Modules.CelestialObjects.Systems.OrbitalSystem')
local SystemMap = require('Modules.CelestialObjects.Systems.SystemMap')
local SystemMap3D = require('Modules.CelestialObjects.Systems.SystemMap3D')
local ShipGenerator = require('Modules.Constructs.Managers.Generators.ShipGenerator')
local StationGenerator = require('Modules.Constructs.Managers.Generators.StationGenerator')

-- Legacy (still needed for skybox generation)
local Generator = require('Legacy.Systems.Gen.Generator')
local Starfield = require('Legacy.Systems.Gen.Starfield')

local rng = RNG.FromTime()

-- Scene tuning: spawn placement + station layout for this state. Camera
-- defaults and ship scale live in the generic game config (orbitCamera,
-- freeCamera, shipHulls).
local SceneConfig = {
    defaultSeed        = 12345,

    -- Ship spawn
    shipSpawnOffset    = 1.3,     -- Multiplier of planet radius (just outside the surface)

    -- Station spawn
    maxStations        = 2,       -- Max stations to spawn (limited by planet count)
    stationScale       = 0.3,     -- ~3km station
    stationOrbitMult   = 2.5,     -- Multiplier of planet radius for station orbit distance
}

local function setMenuScale()
    -- sizes for background star system
    Config.gen.scaleSystem    = Config.gen.scaleSystemBack
    Config.gen.scalePlanet    = Config.gen.scalePlanetBack
    Config.gen.scalePlanetMod = Config.gen.scalePlanetModBack
    GameState.render.zNear    = Config.gen.zNearBack
    GameState.render.zFar     = Config.gen.zFarBack
end

local function setGameScale()
    -- Use the "real" system generation sizes for a gameplay star system
    Config.gen.scaleSystem    = Config.gen.scaleSystemReal
    Config.gen.scalePlanet    = Config.gen.scalePlanetReal
    Config.gen.scalePlanetMod = Config.gen.scalePlanetModReal
    GameState.render.zNear    = Config.gen.zNearReal
    GameState.render.zFar     = Config.gen.zFarReal
end

---@diagnostic disable-next-line: duplicate-set-field
function LimitTheoryRedux:onInit()
    -- Expose the scene builder to the main menu (New Game button)
    GameState.startGame = function(seed) self:startGame(seed) end

    MusicPlayer:init()
    InitFiles:readUserInits()

    Window:setSize(GameState.render.resX, GameState.render.resY)
    Window:setCenteredPosition()
    Window:setPresentMode(PresentMode.NoVsync)
    -- Presentation: borderless fullscreen keeps the desktop refresh rate
    -- (exclusive mode forces a mode switch that lowers frame rates).
    Window:setFullscreen(true, false)

    Window:setCursorPosition(Vec2f(GameState.ui.cursorX, GameState.ui.cursorY))
    SoundManager:init()
    MusicPlayer:loadMusic()

    self:initMainMenu(true)
end

function LimitTheoryRedux:initMainMenu(isAppInit)
    GameState:SetState(Enums.GameStates.MainMenu)
    setMenuScale()

    -- Register UI pages once
    if isAppInit then
        UIRouter:addPage(UIPageMainMenu)
    end
    Input:setCursorVisible(true)

    UIPageMainMenu:setView("Title")

    -- Autonomous test hook: LTR_AUTOSTART=1 skips the main menu and jumps
    -- straight into gameplay (same flow as clicking "New Game").
    if os.getenv("LTR_AUTOSTART") == "1" then
        self:startGame(rng:get64())
    else
        UIRouter:setCurrentPage("Main_Menu")
    end
end

--- Build the scene: universe generation (ruleset-driven), materialization
--- (planets/rings/belts via the texture-fetch renderer), player ship +
--- stations, and all systems.
---@param seed integer
function LimitTheoryRedux:startGame(seed)
    setGameScale()
    GameState:SetState(Enums.GameStates.InGame)

    self.cfg = SceneConfig
    self.seed = seed or SceneConfig.defaultSeed
    self.rng = RNG.Create(self.seed)
    self.world = Physics.Create()

    self:createSkybox()
    self:generateUniverse()
    self:spawnPlayerShip()
    self:spawnStations()

    -- Orbital simulation + gravity wells
    self.orbiters, self.followers = OrbitalSystem:collectOrbiters(self.universe)
    GravityWellSystem:collectZones(self.universe)

    -- System maps
    self.mapState = SystemMap:create()
    SystemMap:collectEntities(self.mapState, self.universe, self.playerShip)
    self.map3DState = SystemMap3D:create()
    SystemMap3D:collectEntities(self.map3DState, self.universe, self.playerShip)
    SystemMap3D:setSkybox(self.map3DState, self.skybox)
    self.mapMode = 0  -- 0=off, 1=2D, 2=3D

    -- Player controller owns camera modes, input routing, and ship reference
    local cam = Config.game.orbitCamera
    local free = Config.game.freeCamera
    self.player = PlayerController(self.playerShip, {
        chase = {
            distance     = cam.distance,
            minDistance   = cam.minDistance,
            maxDistance   = cam.maxDistance,
            zoomSpeed    = cam.zoomSpeed,
            smoothing    = cam.smoothing,
        },
        free = {
            moveSpeed      = free.moveSpeed,
            fastMultiplier = free.fastMultiplier,
            mouseSensitivity = free.mouseSensitivity,
        },
    })

    EventBus:subscribe(Event.Sim, self, self.onStateSim)

    -- Close the menu; in-game UI (HUD, labels) is drawn by the state's
    -- onRender through the rendering systems, not the HmGui pages.
    UIRouter:clearCurrentPage()
end

---@param seed integer
function LimitTheoryRedux:createSkybox()
    local SkyboxEntity = require('Modules.CelestialObjects.Entities.SkyboxEntity')
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
            CameraManager:setStarDir(placeholder.starDir)
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

--- Generate the universe through UniverseManager + SolarSystemVisualizer.
function LimitTheoryRedux:generateUniverse()
    self.universe = UniverseManager:createUniverse(Rulesets.SinglePlanetWithBelt, self.seed)

    if not self.universe then
        Log.Error("Failed to generate universe!")
        return
    end

    -- Rebase all entities to star-system-local coordinates (near origin)
    self.systemOrigin = CoordinateRebaser:rebaseStarSystem(self.universe)

    SolarSystemVisualizer:materialize(self.universe, self.world)

    self.planets = {}
    self.labeledEntities = {}
    self.beltEntities = {}
    self.starEntity = nil
    self:collectEntities(self.universe)
end

--- Walk the generated universe and collect planets/labels/star/belts.
---@param entity Entity
function LimitTheoryRedux:collectEntities(entity)
    local name = tostring(entity)

    if name:find("PlanetEntity") then
        table.insert(self.planets, entity)
        table.insert(self.labeledEntities, { entity = entity, label = name })
    elseif name:find("MoonEntity") then
        table.insert(self.labeledEntities, { entity = entity, label = name })
    elseif name:find("StarEntity") then
        self.starEntity = entity
    elseif name:find("AsteroidBeltEntity") then
        table.insert(self.beltEntities, entity)
    end

    local childrenCmp = entity:get(CoreComponents.Children)
    if childrenCmp then
        for child in childrenCmp:iterChildren() do
            self:collectEntities(child)
        end
    end
end

--- Spawn the player ship near the first planet.
function LimitTheoryRedux:spawnPlayerShip()
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
        scale = Config.game.shipHulls.scale[1],  -- Solo hull (~50m fighter)
        isKinematic = false,
    })

    self.playerShip:add(ConstructComponents.TravelDrive())
    self.playerShip:add(ConstructComponents.AutoPilot())
    self.playerShip:add(ConstructComponents.ShipFlightControl())

    local hullIdx = 1
    local hullCfg = Config.game.shipHulls
    local rbCmp = self.playerShip:get(PhysicsComponents.RigidBody)
    local rb = rbCmp:getRigidBody()
    rb:setMass(hullCfg.mass[hullIdx])
    rb:setDrag(flightCfg.linearDrag, flightCfg.angularDrag)
    rb:setFriction(0)
    rb:setSleepThreshold(0, 0)
    self.world:addRigidBody(rb)

    Log.Info("Player ship spawned at (%.1f, %.1f, %.1f)", spawnPos.x, spawnPos.y, spawnPos.z)
end

--- Spawn stations near planets.
function LimitTheoryRedux:spawnStations()
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
    end
end

---@param data EventData
function LimitTheoryRedux:onRender(data)
    if not self.playerShip then
        -- Menu phase: nothing 3D to render yet (the HmGui pages draw
        -- through Gui:draw below).
        self:immediateUI(function()
            Gui:draw()
        end)
        return
    end

    CelestialLightingSystem:update(self.starEntity)

    if self.mapMode == 2 then
        SystemMap3D:collectEntities(self.map3DState, self.universe, self.playerShip)
        SystemMap3D:render(self.map3DState, data)

        self:immediateUI(function()
            LensFlareSystem:draw(self.starEntity, self.labeledEntities)
            SystemMap3D:renderOverlay(self.map3DState, 0, 0, Window:width(), Window:height())
        end)
    else
        RenderCoreSystem:render(data)

        self:immediateUI(function()
            LensFlareSystem:draw(self.starEntity, self.labeledEntities)
            GameplayHUDSystem:draw(self.playerShip, self.player:getModeName(), self.player:isPiloting())
            WorldLabelRenderSystem:draw(self.labeledEntities)

            local spawned = AsteroidFieldSystem:getSpawnedEntities()
            if #spawned > 0 then
                local eye = CameraManager:getEye()
                if eye then
                    for _, aInfo in ipairs(spawned) do
                        aInfo._dist = (aInfo.pos.x - eye.x)^2 + (aInfo.pos.z - eye.z)^2
                    end
                    table.sort(spawned, function(a, b) return a._dist < b._dist end)
                end
                local asteroidLabels = {}
                for i = 1, math.min(10, #spawned) do
                    table.insert(asteroidLabels, {
                        entity = spawned[i].entity,
                        label = spawned[i].label,
                    })
                end
                WorldLabelRenderSystem:draw(asteroidLabels, 20000)
            end

            if self.mapMode == 1 then
                local ap = self.playerShip:get(ConstructComponents.AutoPilot)
                if ap and ap:isActive() then
                    local tPos = ap:getTargetPos()
                    local tEnt = ap:getTargetEntity()
                    if tEnt then
                        local tRb = tEnt:get(PhysicsComponents.RigidBody)
                        if tRb and tRb:getRigidBody() then tPos = tRb:getRigidBody():getPos() end
                    end
                    self.mapState.autopilotTargetPos = tPos
                    self.mapState.autopilotPos = ap.interceptPos or tPos
                else
                    self.mapState.autopilotPos = nil
                    self.mapState.autopilotTargetPos = nil
                end
                SystemMap:collectEntities(self.mapState, self.universe, self.playerShip)
                SystemMap:draw(self.mapState, 0, 0, Window:width(), Window:height())
            end
        end)
    end
end

---@param data EventData
function LimitTheoryRedux:onInput(data)
    if not self.playerShip then return end
    local dt = data:deltaTime()

    GeneralActions.CycleMapMode:update(dt)
    GeneralActions.AutoPilotToggle:update(dt)
    GeneralActions.Regenerate:update(dt)
    GeneralActions.ReloadShaders:update(dt)

    if GeneralActions.CycleMapMode:isPressed() then
        self.mapMode = (self.mapMode + 1) % 3
        self.mapState.enabled = (self.mapMode == 1)
        self.map3DState.enabled = (self.mapMode == 2)

        if self.mapMode == 1 then
            self.mapState._entityListBuilt = false
            self.mapState._dotCache = nil
            SystemMap:collectEntities(self.mapState, self.universe, self.playerShip)
            CursorManager:free()
        elseif self.mapMode == 2 then
            SystemMap3D:collectEntities(self.map3DState, self.universe, self.playerShip)
            SystemMap3D:activate(self.map3DState)
        else
            SystemMap3D:deactivate(self.map3DState)
            self.mapClosedFrame = true
        end
    end

    if self.mapClosedFrame and self.mapMode == 0 then
        self.mapClosedFrame = false
        self.player:setMode(self.player:getMode())
    end

    if self.mapMode > 0 then
        local ctrl = self.player:getActiveController()
        if ctrl and ctrl:isEnabled() then ctrl:disable() end
        self.player:setInputBlocked(true)

        if self.mapMode == 1 then
            SystemMap:updateInput(self.mapState, dt)

            if GeneralActions.AutoPilotToggle:isPressed() and self.mapState.selected then
                local sel = self.mapState.selected
                if sel.clickPos then
                    local parentEnt = sel._parentEntity
                    local localX = sel._clickLocalX
                    local localZ = sel._clickLocalZ
                    AutoPilotSystem:engagePosition(self.playerShip, sel.clickPos, nil, parentEnt, localX, localZ)
                else
                    AutoPilotSystem:engageEntity(self.playerShip, sel.entity)
                end
                self.mapMode = 0
                self.mapState.enabled = false
                self.mapClosedFrame = true
            end
        else
            SystemMap3D:updateInput(self.map3DState, dt)

            if GeneralActions.AutoPilotToggle:isPressed() and self.map3DState.selected then
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

    if GeneralActions.AutoPilotToggle:isPressed() then
        if AutoPilotSystem:isActive(self.playerShip) then
            AutoPilotSystem:disengage(self.playerShip)
        end
    end

    if GeneralActions.Regenerate:isPressed() then
        self.seed = self.rng:get31()
        self:regenerate()
    end

    if GeneralActions.ReloadShaders:isPressed() then
        local Material = require("Shared.Rendering.Material")
        Cache.ReloadShaders()
        Material.ReloadAll()
    end
end

---@param data EventData
function LimitTheoryRedux:onStateSim(data)
    if not self.playerShip then return end
    local dt = data:deltaTime()

    OrbitalSystem:update(self.orbiters, dt, self.followers)
    AsteroidFieldSystem:updatePositions()
    GravityWellSystem:update(dt, self.playerShip)
    AsteroidFieldSystem:update(dt, self.beltEntities, self.world)
    SystemMap3D:updateTrails(self.map3DState, dt)

    self.player:update(dt)
    self.world:update(dt)
end

--- Rebuild the scene with a new seed.
function LimitTheoryRedux:regenerate()
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
    self.beltEntities = {}
    AsteroidFieldSystem:cleanup(self.world)
    self.rng = RNG.Create(self.seed)
    self:generateUniverse()
    self:spawnPlayerShip()
    self:spawnStations()
    self.player:setShip(self.playerShip)

    self.orbiters, self.followers = OrbitalSystem:collectOrbiters(self.universe)
    GravityWellSystem:collectZones(self.universe)
    self.mapState._entityListBuilt = false
    self.mapState._dotCache = nil
    SystemMap:collectEntities(self.mapState, self.universe, self.playerShip)
    SystemMap3D:collectEntities(self.map3DState, self.universe, self.playerShip)
end

function LimitTheoryRedux:soundOn()
    GameState.audio.soundEnabled = true
    MusicPlayer:setVolume(MusicPlayer.lastVolume)
end

function LimitTheoryRedux:soundOff()
    GameState.audio.soundEnabled = false
    MusicPlayer:setVolume(0)
end

--* any operations we want to do before exiting the game
function LimitTheoryRedux:exit()
    -- Update Session vars ; temporary until we have a save state
    GameState.player.startupCamera = GameState.player.currentCamera
    -- Write player-specific game variables to preserve them across gameplay sessions
    InitFiles:writeUserInits()

    Engine:exit()
end

return LimitTheoryRedux
