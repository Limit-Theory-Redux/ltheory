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
local UIPageGameplay = require('UI.HmGui.Pages.Gameplay')
local ApplicationBindings = require('States.ApplicationBindings')

-- ECS registry, physics/construct/celestial components, camera manager,
-- render core, player controller, and the simulation systems the scene
-- runs on.
local Registry = require('Core.ECS.Registry')
local CoreComponents = require('Modules.Core.Components')
local PhysicsComponents = require('Modules.Physics.Components')
local ConstructComponents = require('Modules.Constructs.Components')
local CelestialComponents = require('Modules.CelestialObjects.Components')
local CameraManager = require('Modules.Cameras.Managers.CameraManager')
local CameraEntity = require('Modules.Cameras.Entities').Camera
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
local MemoryReporter = require('Modules.Profiling.MemoryReporter')
local ConstructManager = require('Modules.Constructs.Managers.ConstructManager')
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
        UIRouter:addPage(UIPageGameplay)
    end
    Input:setCursorVisible(true)

    UIPageMainMenu:setView("Title")

    -- Autonomous test hook: LTR_AUTOSTART=1 skips the main menu and jumps
    -- straight into gameplay (same flow as clicking "New Game").
    if os.getenv("LTR_AUTOSTART") == "1" then
        self:startGame(rng:get64())
    else
        -- Menu phase: spawn a star system behind the menu (the legacy
        -- GameView background path is gone; the modern pipeline renders it).
        self:createMenuBackground()
        UIRouter:setCurrentPage("Main_Menu")
    end
end

--- Spawn the menu background star system through the modern scene pipeline
--- (UniverseManager + Rulesets + SolarSystemVisualizer) with its own camera,
--- mirroring the camera pattern of the working testbed states.
function LimitTheoryRedux:createMenuBackground()
    if self.menuWorld then return end

    self.world = Physics.Create()
    self.menuWorld = self.world

    -- Skybox + universe at menu scale (Config.gen scaleSystemBack etc.)
    setMenuScale()
    self.menuSeed = rng:get64()
    self.seed = self.menuSeed
    self:createSkybox()
    self:generateUniverse()

    -- Orbit camera around the first planet (same registration pattern as
    -- PlanetTest: no controller, transform driven by updateMenuCamera).
    local camEntity = CameraEntity()
    CameraManager:registerCamera("MenuCamera", camEntity)
    CameraManager:setActiveCamera("MenuCamera")

    local planetPos = Position(0, 0, 0)
    local planetRadius = 100
    if self.planets and self.planets[1] then
        local planet = self.planets[1]
        local t = planet:get(PhysicsComponents.Transform)
        if t then
            planetPos = t:getPos()
            planetRadius = math.max(t:getScale(), 100)
        end
    end
    self.menuPlanetPos = planetPos
    self.menuOrbitAngle = 0.0
    self.menuOrbitSpeed = 0.02
    self.menuOrbitPitch = 0.25
    self.menuOrbitRadius = planetRadius * 4.0
    self:updateMenuCamera(0)
end

--- Advance the menu camera on its orbit around the menu planet.
---@param dt number
function LimitTheoryRedux:updateMenuCamera(dt)
    if not self.menuWorld then return end

    self.menuOrbitAngle = self.menuOrbitAngle + (self.menuOrbitSpeed * (dt or 0))

    local camEntity = CameraManager:getActiveCameraEntity()
    if not camEntity then return end
    local transform = camEntity:get(PhysicsComponents.Transform)
    if not transform then return end

    local r = self.menuOrbitRadius
    local x = math.cos(self.menuOrbitAngle) * r
    local y = math.sin(self.menuOrbitPitch) * r
    local z = math.sin(self.menuOrbitAngle) * r
    local targetPos = Vec3f(self.menuPlanetPos.x, self.menuPlanetPos.y, self.menuPlanetPos.z)
    local camPos = Vec3f(targetPos.x + x, targetPos.y + y, targetPos.z + z)

    transform:setPos(Position(camPos.x, camPos.y, camPos.z))
    local lookDir = (targetPos - camPos):normalize()
    transform:setRot(Quat.FromLook(lookDir, Vec3f(0, 1, 0)))
end

--- Tear down the menu background before the game builds its own scene.
function LimitTheoryRedux:cleanupMenuBackground()
    if not self.menuWorld then return end

    if self.universe then
        -- Bodies must leave the physics world before their entities die.
        local bodies = {}
        for entity, rbCmp in Registry:iterEntities(PhysicsComponents.RigidBody) do
            local rb = rbCmp and rbCmp:getRigidBody()
            if rb then
                table.insert(bodies, { entity = entity, rb = rb })
            end
        end
        for _, b in ipairs(bodies) do
            self.menuWorld:removeRigidBody(b.rb)
        end
        Registry:destroyEntity(self.universe, Enums.Registry.EntityDestroyMode.DestroyChildren)
    end

    -- The skybox entity is standalone (not attached to the universe).
    if self.skybox then
        Registry:destroyEntity(self.skybox, Enums.Registry.EntityDestroyMode.DestroyChildren)
        self.skybox = nil
    end

    self.universe = nil
    self.planets = {}
    self.labeledEntities = {}
    self.beltEntities = {}
    self.starEntity = nil
    self.menuWorld = nil
    self.world = nil
end

--- Open the in-game pause menu (Escape). The Gameplay page's Paused view
--- provides Return to Game / Back to Main Menu / Exit.
function LimitTheoryRedux:openPauseMenu()
    self.pauseMenuOpen = true
    GameState:Pause()
    -- Free the cursor (visible, no grab): gameplay runs FPS-style Locked,
    -- which pins the cursor and makes the Paused view unclickable.
    CursorManager:free()
    UIRouter:setCurrentPage("Gameplay")
    UIPageGameplay:setView("Paused")
end

--- Close the in-game pause menu and resume the sim.
function LimitTheoryRedux:closePauseMenu()
    self.pauseMenuOpen = false
    UIRouter:clearCurrentPage()
    GameState:Unpause()
    -- Re-apply the gameplay cursor state for the current camera mode
    -- (Locked/Confined/Free), same pattern as the map-mode close path.
    if self.player then
        self.player:setMode(self.player:getMode())
    end
end

--- Tear down the gameplay scene and return to the main menu.
function LimitTheoryRedux:returnToMainMenu()
    self:closePauseMenu()

    if self._simTunnel then
        EventBus:unsubscribe(self._simTunnel)
        self._simTunnel = nil
    end

    -- Same teardown as regenerate: bodies leave the world first.
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

    self.playerShip = nil
    self.stations = {}
    self.mapMode = 0

    -- The menu needs a free cursor (closePauseMenu re-applied the gameplay
    -- capture just above); Release it before switching back to the menu.
    CursorManager:free()

    self:initMainMenu()
end

--- Build the scene: universe generation (ruleset-driven), materialization
--- (planets/rings/belts via the texture-fetch renderer), player ship +
--- stations, and all systems.
---@param seed integer
function LimitTheoryRedux:startGame(seed)
    setGameScale()
    GameState:SetState(Enums.GameStates.InGame)

    -- Drop the menu background (its universe/world/camera) before building
    -- the gameplay scene.
    self:cleanupMenuBackground()

    self.cfg = SceneConfig
    -- Static scene: bodies stay at generated positions (see onStateSim)
    self.staticStarSystem = true
    -- Random universe each game: a nil seed means "surprise me", never a
    -- fixed default (the same universe on every New Game would be stale).
    self.seed = seed or rng:get64()
    self.rng = RNG.Create(self.seed)
    self.world = Physics.Create()
    self.constructManager = ConstructManager(Registry, self.world)

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

    self._simTunnel = EventBus:subscribe(Event.Sim, self, self.onStateSim)

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

--- Spawn the player ship next to the nearest belt asteroid (so the first
--- thing the player sees is a rock up close, with the ring/planet nearby).
function LimitTheoryRedux:spawnPlayerShip()
    local cfg = self.cfg
    local flightCfg = Config.game.shipFlight

    local spawnPos = Position(0, 0, 0)
    if #self.planets > 0 then
        local planet = self.planets[1]
        local planetPos = planet:get(PhysicsComponents.Transform):getPos()

        -- Nearest belt asteroid to the planet: belt data is baked
        -- relative to the belt's render origin (the parent body when one
        -- exists, else the belt's own transform).
        local beltEntity = self.beltEntities and self.beltEntities[1]
        local beltCmp = beltEntity and beltEntity:get(CelestialComponents.AsteroidBelt)
        local asteroids = beltCmp and beltCmp:getAsteroidData() or {}

        local ox, oy, oz = 0, 0, 0
        if beltEntity then
            local originEntity = beltEntity
            local parentCmp = beltEntity:get(CoreComponents.Parent)
            if parentCmp then
                local p = parentCmp:getParent()
                if p and p:get(PhysicsComponents.Transform) then originEntity = p end
            end
            local t = originEntity:get(PhysicsComponents.Transform)
            if t then local p = t:getPos() ox, oy, oz = p.x, p.y, p.z end
        end

        local bestIdx, bestDistSq = 1, math.huge
        for i = 1, #asteroids do
            local a = asteroids[i]
            local dx = (ox + a.px) - planetPos.x
            local dy = (oy + a.py) - planetPos.y
            local dz = (oz + a.pz) - planetPos.z
            local d = dx * dx + dy * dy + dz * dz
            if d < bestDistSq then bestDistSq = d; bestIdx = i end
        end

        local a = asteroids[bestIdx]
        if a then
            -- Just off the rock's surface, on the side away from the planet
            local ax, ay, az = ox + a.px, oy + a.py, oz + a.pz
            local clearDist = a.scale * 2.0
            local px, py, pz = planetPos.x, planetPos.y, planetPos.z
            local nx, ny, nz = ax - px, ay - py, az - pz
            local len = math.sqrt(nx * nx + ny * ny + nz * nz)
            if len > 1e-9 then nx, ny, nz = nx / len, ny / len, nz / len end
            spawnPos = Position(ax + nx * clearDist, ay + ny * clearDist, az + nz * clearDist)
        end
    end

    self.playerShipHandle = self.constructManager:createShip({
        seed = self.rng:get31(),
        shipType = Enums.ShipType.Fighter,
        config = {
            position = spawnPos,
            scale = Config.game.shipHulls.scale[1],
            isKinematic = false,
        },
    })
    self.playerShip = self.playerShipHandle.root

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
        -- Menu phase: the background star system renders through the modern
        -- core first; the HmGui pages draw on top via Gui:draw below.
        if self.menuWorld then
            self:updateMenuCamera(data:deltaTime())
            CelestialLightingSystem:update(self.starEntity)
            RenderCoreSystem:render(data)
        end
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
            Gui:draw()
            LensFlareSystem:draw(self.starEntity, self.labeledEntities)
            SystemMap3D:renderOverlay(self.map3DState, 0, 0, Window:width(), Window:height())
        end)
    else
        RenderCoreSystem:render(data)

        self:immediateUI(function()
            Gui:draw()
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

    -- Escape toggles the pause menu (Return to Game / Back to Main Menu / Exit).
    if Input:isPressed(ApplicationBindings.Escape) then
        if self.pauseMenuOpen then
            self:closePauseMenu()
        else
            self:openPauseMenu()
        end
    end
    if self.pauseMenuOpen then return end

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

    -- Static star system: skip orbital motion so bodies stay at their
    -- generated positions (deterministic scene while orbital mechanics
    -- are still being developed). Asteroid fields still follow parents.
    if not self.staticStarSystem then
        OrbitalSystem:update(self.orbiters, dt, self.followers)
    end
    AsteroidFieldSystem:updatePositions()
    GravityWellSystem:update(dt, self.playerShip)
    AsteroidFieldSystem:update(dt, self.beltEntities, self.world, self.playerShip)
    SystemMap3D:updateTrails(self.map3DState, dt)

    MemoryReporter:update(dt)

    self.player:update(dt)
    self.world:update(dt)
end

--- Rebuild the scene with a new seed.
function LimitTheoryRedux:regenerate()
    if self.universe then
        Registry:destroyEntity(self.universe, Enums.Registry.EntityDestroyMode.DestroyChildren)
    end
    if self.playerShipHandle then
        self.constructManager:destroy(self.playerShipHandle)
        self.playerShipHandle = nil
    elseif self.playerShip then
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
