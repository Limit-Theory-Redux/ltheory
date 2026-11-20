-- LTheoryRedux: the main game state. Flow: main menu (HmGui) -> New Game
-- -> startGame(seed) builds the scene through UniverseManager + Rulesets
-- + SolarSystemVisualizer, then renders via RenderCoreSystem with HUD,
-- system maps, and player ship control.
Namespace.LoadInline('Legacy')
Namespace.LoadInline('Legacy.Systems')

LimitTheoryRedux = require('States.Application')

-- Sound / persistence
local SoundManager = require('Legacy.Systems.SFX.SoundManager')
local MusicPlayer  = require('Shared.Audio.MusicPlayer')
local InitFiles    = require('Legacy.Systems.Files.InitFiles')
local UserSettings = require('Shared.UserSettings')

-- UI pages (HmGui): the Main_Menu page before a game, the Gameplay page in
-- game (In_Game view with the HUD text, Paused and Controls views for the pause
-- menu). The maps and world labels are still drawn by the state's onRender
-- through the rendering systems.
local UIRouter            = require('UI.HmGui.UICore.UIRouter')
local UIPageMainMenu      = require('UI.HmGui.Pages.MainMenu')
local UIPageGameplay      = require('UI.HmGui.Pages.Gameplay')
local UIViewInGame        = require('UI.HmGui.Views.Gameplay.InGame')
local ApplicationBindings = require('States.ApplicationBindings')

-- ECS registry, physics/construct/celestial components, camera manager,
-- render core, player controller, and the simulation systems the scene
-- runs on.
local Registry                 = require('Core.ECS.Registry')
local CoreComponents           = require('Modules.Core.Components')
local PhysicsComponents        = require('Modules.Physics.Components')
local ConstructComponents      = require('Modules.Constructs.Components')
local CelestialComponents      = require('Modules.CelestialObjects.Components')
local CameraManager            = require('Modules.Cameras.Managers.CameraManager')
local CameraEntity             = require('Modules.Cameras.Entities').Camera
local RenderCoreSystem         = require('Modules.Rendering.Systems.RenderCoreSystem')
local PlayerController         = require('Modules.Constructs.Systems.PlayerController')
local AutoPilotSystem          = require('Modules.Constructs.Systems.AutoPilotSystem')
local Nav                      = require('Modules.Navigation')
local ShipFlightSystem         = require('Modules.Constructs.Systems.ShipFlightSystem')
local TravelDriveSystem        = require('Modules.Constructs.Systems.TravelDriveSystem')
local GravityWellSystem        = require('Modules.Physics.Systems.GravityWellSystem')
local CelestialLightingSystem  = require('Modules.Rendering.Systems.CelestialLightingSystem')
local LensFlareSystem          = require('Modules.Rendering.Systems.LensFlareSystem')
local WorldLabelRenderSystem   = require('Modules.UI.Systems.WorldLabelRenderSystem')
local RouteOverlayRenderSystem = require('Modules.UI.Systems.RouteOverlayRenderSystem')
local ScreenProjection         = require('Shared.Helpers.ScreenProjection')
local EntityPosition           = require('Shared.Helpers.EntityPosition')
local AsteroidFieldSystem      = require('Modules.CelestialObjects.Systems.AsteroidFieldSystem')
local GeneralActions           = require('Input.ActionBindings.GeneralActions')
local CursorManager            = require('Input.CursorManager')
local Rulesets                 = require('Config.Gen.Rulesets')
local UniverseManager          = require('Modules.CelestialObjects.Managers.UniverseManager')
local SolarSystemVisualizer    = require('Modules.CelestialObjects.Managers.SolarSystemVisualizer')
local CoordinateRebaser        = require('Modules.CelestialObjects.Managers.CoordinateRebaser')
local OrbitalSystem            = require('Modules.CelestialObjects.Systems.OrbitalSystem')
local SystemMap                = require('Modules.CelestialObjects.Systems.SystemMap')
local SystemMap3D              = require('Modules.CelestialObjects.Systems.SystemMap3D')
local MemoryReporter           = require('Modules.Profiling.MemoryReporter')
local ConstructManager         = require('Modules.Constructs.Managers.ConstructManager')
local StationGenerator         = require('Modules.Constructs.Managers.Generators.StationGenerator')
local WeaponCombatSystem       = require('Modules.Constructs.Systems.WeaponCombatSystem')
local ExplosionEffectSystem    = require('Modules.Constructs.Systems.ExplosionEffectSystem')

-- Skybox. This state also draws from the light-transport nebula, which the
-- Legacy namespace load used to register.
local Skybox = require('Shared.Generation.Skybox')
require('Shared.Generation.Nebula2')

-- How a game starts (New Game's Scenario list)
local Scenarios = require('Config.Scenarios')

local rng = RNG.FromTime()

-- Reused every frame: the autopilot's overlay for the maps and the flight view, and the flight view's projector.
local autopilotOverlay = {}
local flightProjector = {}

-- Scene tuning: spawn placement + station layout for this state. Camera
-- defaults and ship scale live in the generic game config (orbitCamera,
-- freeCamera, shipHulls).
local SceneConfig = {
    defaultSeed        = 12345,

    -- Ship spawn
    shipSpawnOffset    = 1.3,     -- Multiplier of planet radius (just outside the surface), without belt rocks
    starSpawnOffset    = 3.0,     -- Multiplier of star radius, without planets

    -- Station spawn
    maxStations        = 2,       -- Max stations to spawn (limited by planet count)
    stationScale       = 0.3,     -- ~3km station
    stationOrbitMult   = 2.5,     -- Multiplier of planet radius for station orbit distance
}

--- Free the GPU memory of a scene that was just destroyed. Its textures and
--- meshes are freed by the finalizers of their Lua objects, and the frame GC is
--- paced by Lua memory, which barely sees them (a planet's surface texture is
--- tens of MB on the GPU and a few bytes in Lua): without a full collection,
--- building a few star systems in a row runs out of GPU memory.
-- Frames between tearing a scene down and building it again (Regenerate): the
-- renderer keeps up to three frames in flight.
local REBUILD_DELAY_FRAMES = 4

local function releaseGpuMemory()
    collectgarbage('collect')
    collectgarbage('collect') -- the second pass runs the finalizers queued by the first
end

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
    GameState.startGame = function(seed, scenario) self:startGame(seed, scenario) end

    -- Presentation: borderless fullscreen with VSync off, unless user.ini says
    -- otherwise (exclusive mode forces a mode switch that lowers frame rates).
    UserSettings.useGameDisplayDefaults()
    -- Before user.ini: init lists the soundtracks that its menuTheme is matched
    -- against. The tracks get the saved volumes in loadMusic below.
    MusicPlayer:init()
    InitFiles:readUserInits()

    Window:setSize(GameState.render.resX, GameState.render.resY)
    Window:setCenteredPosition()
    -- Display mode, VSync, field of view, brightness, bloom, smoothing, texture quality
    UserSettings.applyAll()

    Window:setCursorPosition(Vec2f(GameState.ui.cursorX, GameState.ui.cursorY))
    SoundManager:init()
    MusicPlayer:loadMusic()
    -- Captures (LTHEORY_CAPTURE) are silent: nobody listens to them, and the
    -- render validation runs a dozen in a row. (Not saved: captures never
    -- write user.ini.)
    if os.getenv('LTHEORY_CAPTURE') then
        self:soundOff()
    end

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

    -- Capture variant (render validation): LTHEORY_CAPTURE_VIEW picks the menu view, only under LTHEORY_CAPTURE.
    UIPageMainMenu:setView(os.getenv('LTHEORY_CAPTURE') and os.getenv('LTHEORY_CAPTURE_VIEW') or "Title")

    -- Straight into a game, skipping the main menu: LTR_AUTOSTART=1 (with
    -- LTR_SCENARIO=<scenario id>, else the default scenario), or a test state
    -- that set `autostartScenario` (Testbeds/WeaponSystem, PlanetTest...).
    -- Under LTHEORY_CAPTURE, LTHEORY_CAPTURE_SEED fixes the seed.
    local autostart = isAppInit and self.autostartScenario
    if isAppInit and not autostart and os.getenv("LTR_AUTOSTART") == "1" then
        local id = os.getenv("LTR_SCENARIO")
        autostart = Scenarios.find(id)
        if not autostart then
            if id then Log.Warn("LTR_SCENARIO: there is no scenario '%s', starting the default one", id) end
            autostart = Scenarios.default
        end
    end
    if autostart then
        local seed = os.getenv('LTHEORY_CAPTURE') and tonumber(os.getenv('LTHEORY_CAPTURE_SEED'))
        self:startGame(seed or self.autostartSeed or rng:get64(), autostart)
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
    -- Capture variant (render validation): LTHEORY_CAPTURE_SEED fixes the menu scene, only under LTHEORY_CAPTURE.
    self.menuSeed = os.getenv('LTHEORY_CAPTURE') and tonumber(os.getenv('LTHEORY_CAPTURE_SEED')) or rng:get64()
    self.seed = self.menuSeed
    self:createSkybox()
    self:generateUniverse(Scenarios.default.ruleset)

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

    releaseGpuMemory()
end

--- Open the in-game pause menu (Escape). The Gameplay page's Paused view
--- provides Return to Game / Controls / Back to Main Menu / Exit.
function LimitTheoryRedux:openPauseMenu()
    self.pauseMenuOpen = true
    GameState:Pause()
    -- Free the cursor (visible, no grab): gameplay runs FPS-style Locked,
    -- which pins the cursor and makes the Paused view unclickable.
    CursorManager:free()
    UIPageGameplay:setView("Paused")
end

--- Close the in-game pause menu and resume the sim.
function LimitTheoryRedux:closePauseMenu()
    self.pauseMenuOpen = false
    self.controlsOpen = false
    UIPageGameplay:setView("In_Game")
    GameState:Unpause()
    -- Re-apply the gameplay cursor state for the current camera mode
    -- (Locked/Confined/Free), same pattern as the map-mode close path.
    if self.player then
        self.player:setMode(self.player:getMode())
    end
end

--- Show the Controls (help) screen; the game stays paused.
function LimitTheoryRedux:openControls()
    self.controlsOpen = true
    UIPageGameplay:setView("Controls")
end

--- Back from the Controls screen to the pause menu.
function LimitTheoryRedux:closeControls()
    self.controlsOpen = false
    UIPageGameplay:setView("Paused")
end

--- P freezes the sim without opening the pause menu.
function LimitTheoryRedux:togglePause()
    if self.pauseMenuOpen then return end
    if GameState.paused then GameState:Unpause() else GameState:Pause() end
end

function LimitTheoryRedux:toggleHUD()
    self.hudHidden = not self.hudHidden
end

--- Tear down the gameplay scene and return to the main menu.
function LimitTheoryRedux:returnToMainMenu()
    self:closePauseMenu()

    self:destroyScene()
    self.weaponCombat = nil
    RenderCoreSystem:setDebugPhysics(nil)
    -- The menu makes a skybox of its own
    if self.skybox then
        Registry:destroyEntity(self.skybox, Enums.Registry.EntityDestroyMode.DestroyChildren)
        self.skybox = nil
    end

    self.scenario = nil
    self.rebuildInFrames = nil
    UIViewInGame:setPlayer(nil)
    UIViewInGame:setOverlayLines(nil)
    self.mapMode = 0

    -- The menu needs a free cursor (closePauseMenu re-applied the gameplay
    -- capture just above); Release it before switching back to the menu.
    CursorManager:free()

    self:initMainMenu()
end

--- Start a game: the scenario's star system (ruleset-driven generation, then
--- materialization of planets/rings/belts), the player's ship, the stations
--- and the scenario's own things; then the systems that run it.
---@param seed integer|nil nil = a random seed
---@param scenario Scenario|nil nil = the default scenario
function LimitTheoryRedux:startGame(seed, scenario)
    setGameScale()
    GameState:SetState(Enums.GameStates.InGame)

    -- In-game soundtrack: queue every non-menu track and play one.
    MusicPlayer:playAmbient()

    -- Drop the menu background (its universe/world/camera) before building
    -- the gameplay scene.
    self:cleanupMenuBackground()

    self.cfg = SceneConfig
    self.scenario = scenario or Scenarios.default
    -- Bodies stay at their generated positions unless the scenario has
    -- orbits (see onSim)
    self.staticStarSystem = not self.scenario.orbits
    -- Random universe each game: a nil seed means "surprise me", never a
    -- fixed default (the same universe on every New Game would be stale).
    self.seed = seed or rng:get64()
    self.rng = RNG.Create(self.seed)
    -- The Draw HUD setting; V still toggles the HUD during the game.
    self.hudHidden = not GameState.ui.drawHud
    self.world = Physics.Create()
    self.world:setLengthUnit(Config.game.physics.lengthUnit)
    self.world:setContactNaturalFrequency(Config.game.physics.contactNaturalFrequency)
    self:attachNavigation()
    self.constructManager = ConstructManager(Registry, self.world)
    -- Every armed ship's weapons (WeaponBattery) run through this
    self.weaponCombat = WeaponCombatSystem(self.world)
    RenderCoreSystem:setDebugPhysics(self.world)

    self:createSkybox()
    self:buildScene()

    -- Player controller owns camera modes, input routing, and ship reference
    local orbit = Config.game.orbitCamera
    local free = Config.game.freeCamera
    self.player = PlayerController(self.playerShip, {
        orbit = {
            distance    = orbit.distance,
            minDistance = orbit.minDistance,
            maxDistance = orbit.maxDistance,
            zoomSpeed   = orbit.zoomSpeed,
            zoomRatio   = orbit.zoomRatio,
            smoothing   = orbit.smoothing,
        },
        free = {
            moveSpeed      = free.moveSpeed,
            fastMultiplier = free.fastMultiplier,
            mouseSensitivity = free.mouseSensitivity,
        },
    })
    UIViewInGame:setPlayer(self.player)

    -- System maps
    self.mapState = SystemMap:create()
    self.map3DState = SystemMap3D:create()
    SystemMap3D:setSkybox(self.map3DState, self.skybox)
    self.mapMode = 0  -- 0=off, 1=2D, 2=3D

    self:setupScenario()
    self:collectSceneSystems()

    -- Swap the menu for the Gameplay page. Its In_Game view draws on top of
    -- the HUD and labels that the state's onRender draws.
    UIRouter:setCurrentPage("Gameplay")
    UIPageGameplay:setView("In_Game")
end

--- The skybox for `self.seed`: nebula, stars, light direction.
function LimitTheoryRedux:createSkybox()
    self.skybox = Skybox.Create(self.seed)
end

--- The star system, the player's ship and the stations, from the scenario.
function LimitTheoryRedux:buildScene()
    self.sceneEntities = {}
    self.sceneConstructs = {}
    self:generateUniverse(self.scenario.ruleset or Scenarios.default.ruleset)
    self:spawnPlayerShip()
    if self.scenario.stations ~= false then
        self:spawnStations()
    else
        self.stations = {}
    end
end

--- Run the scenario's setup (after the scene and the player controller exist).
function LimitTheoryRedux:setupScenario()
    self.scenarioData = {}
    if self.scenario.setup then
        self.scenario.setup(self)
    end
    -- The scenario's text, which the HUD shows under its zone line
    local overlayLines = self.scenario.overlayLines
    UIViewInGame:setOverlayLines(overlayLines and function() return overlayLines(self) end or nil)
end

--- Orbits, gravity wells and the maps' entity lists, after the scene changed.
function LimitTheoryRedux:collectSceneSystems()
    -- Orbits are set up only where they run: a static scene's bodies stay at their generated places,
    -- and predictions (the planner's, the wells') take them as fixed.
    if self.staticStarSystem then
        self.orbiters, self.followers = {}, {}
    else
        self.orbiters, self.followers = OrbitalSystem:collectOrbiters(self.universe)
    end
    GravityWellSystem:collectZones()
    self.mapState._entityListBuilt = false
    self.mapState._dotCache = nil
    SystemMap:collectEntities(self.mapState, self.playerShip)
    SystemMap3D:collectEntities(self.map3DState, self.playerShip)
end

--- Generate the universe through UniverseManager + SolarSystemVisualizer.
---@param ruleset Ruleset
function LimitTheoryRedux:generateUniverse(ruleset)
    self.universe = UniverseManager:createUniverse(ruleset, self.seed)

    if not self.universe then
        Log.Error("Failed to generate universe!")
        return
    end

    -- Rebase all entities to star-system-local coordinates (near origin)
    self.systemOrigin = CoordinateRebaser:rebaseStarSystem()

    SolarSystemVisualizer:materialize(self.world)

    self:collectEntities()
end

--- Something a scenario added to the scene: it is removed with the scene
--- (Back to Main Menu, Regenerate), and labelled if it has a label.
---@param entity Entity
---@param options { label: string|nil, addBody: boolean|nil, bodyAdded: boolean|nil }|nil addBody: put its rigid body into the physics world (and take it out again on removal); bodyAdded: it is in the world already (take it out on removal)
function LimitTheoryRedux:addSceneEntity(entity, options)
    options = options or {}
    local ownsBody = options.bodyAdded == true
    if options.addBody then
        local rbCmp = entity:get(PhysicsComponents.RigidBody)
        if rbCmp and rbCmp:getRigidBody() then
            self.world:addRigidBody(rbCmp:getRigidBody())
            ownsBody = true
        end
    end
    table.insert(self.sceneEntities, { entity = entity, ownsBody = ownsBody })
    if options.label then
        table.insert(self.labeledEntities, { entity = entity, label = options.label })
    end
end

--- A construct (ConstructManager handle) a scenario added: destroyed with the
--- scene, and labelled if it has a label.
---@param handle table
---@param label string|nil
function LimitTheoryRedux:addSceneConstruct(handle, label)
    table.insert(self.sceneConstructs, handle)
    if label then
        table.insert(self.labeledEntities, { entity = handle.root, label = label })
    end
end

local function forgetLabel(labels, entity)
    for i = #labels, 1, -1 do
        if labels[i].entity == entity then
            table.remove(labels, i)
        end
    end
end

--- Remove something `addSceneEntity` added, now.
---@param entity Entity
function LimitTheoryRedux:removeSceneEntity(entity)
    for i, item in ipairs(self.sceneEntities) do
        if item.entity == entity then
            if item.ownsBody then
                local rbCmp = entity:get(PhysicsComponents.RigidBody)
                if rbCmp and rbCmp:getRigidBody() then
                    self.world:removeRigidBody(rbCmp:getRigidBody())
                end
            end
            table.remove(self.sceneEntities, i)
            break
        end
    end
    forgetLabel(self.labeledEntities, entity)
    if self.player and self.player:getOrbitTarget() == entity then
        self.player:setOrbitTarget(nil)
    end
    Registry:destroyEntity(entity, Enums.Registry.EntityDestroyMode.DestroyChildren)
end

--- Destroy a construct `addSceneConstruct` added, now.
---@param handle table
function LimitTheoryRedux:removeSceneConstruct(handle)
    for i, h in ipairs(self.sceneConstructs) do
        if h == handle then
            table.remove(self.sceneConstructs, i)
            break
        end
    end
    forgetLabel(self.labeledEntities, handle.root)
    if self.player and self.player:getOrbitTarget() == handle.root then
        self.player:setOrbitTarget(nil)
    end
    self.constructManager:destroy(handle)
end

--- Start the orbit camera around an entity (a scenario's planet, station...).
---@param entity Entity
---@param distances table|nil zoom range, see `orbitDistances`
function LimitTheoryRedux:setOrbitFocus(entity, distances)
    self.player:setOrbitTarget(entity, distances)
    -- From the lit side: a little off the sun's direction, from slightly above
    local sun = CameraManager:getStarDir()
    self.player:setOrbitAngles(math.atan2(sun.x, sun.z) + 0.5, 0.25)
    self.player:setMode(Enums.CameraMode.Orbit)
end

--- A zoom range for orbiting something of this radius.
---@param radius number
---@return table
function LimitTheoryRedux:orbitDistances(radius)
    return {
        distance    = radius * 3.0,
        minDistance = radius * 1.2,
        maxDistance = radius * 40.0,
        zoomRatio   = 0.15,
    }
end

--- Give the game's physics world a navigation world, which flies the orders the ships are given
--- (the player's autopilot among them).
function LimitTheoryRedux:attachNavigation()
    Nav.attach(self.world)
    self.navAttached = true
end

--- Remove what a game built: the scenario's things, the stations, the star
--- system and (unless kept) the player's ship.
---@param keepShip boolean|nil
function LimitTheoryRedux:destroyScene(keepShip)
    if self.scenario and self.scenario.teardown then
        self.scenario.teardown(self)
    end
    -- The orders end while the ships' bodies are still in the physics world; the autopilot lets go
    -- of its ended order.
    if self.navAttached then
        Nav.detach()
        self.navAttached = false
        AutoPilotSystem:update()
    end
    -- Beams, projectiles and effect lights, before the ships they came from
    if self.weaponCombat then
        self.weaponCombat:clear()
    end
    -- Before the things it may follow go
    if self.player then
        self.player:setOrbitTarget(nil)
    end

    for i = #(self.sceneEntities or {}), 1, -1 do
        self:removeSceneEntity(self.sceneEntities[i].entity)
    end
    for i = #(self.sceneConstructs or {}), 1, -1 do
        self:removeSceneConstruct(self.sceneConstructs[i])
    end

    for _, station in ipairs(self.stations or {}) do
        local rbCmp = station:get(PhysicsComponents.RigidBody)
        if rbCmp and rbCmp:getRigidBody() then
            self.world:removeRigidBody(rbCmp:getRigidBody())
        end
        Registry:destroyEntity(station, Enums.Registry.EntityDestroyMode.DestroyChildren)
    end
    self.stations = {}

    if self.universe then
        Registry:destroyEntity(self.universe, Enums.Registry.EntityDestroyMode.DestroyChildren)
        self.universe = nil
    end

    if not keepShip then
        if self.playerShipHandle then
            self.constructManager:destroy(self.playerShipHandle)
        elseif self.playerShip then
            Registry:destroyEntity(self.playerShip, Enums.Registry.EntityDestroyMode.DestroyChildren)
        end
        self.playerShipHandle = nil
        self.playerShip = nil
    end

    self.planets = {}
    self.labeledEntities = {}
    self.starEntity = nil
    self.beltEntities = {}
    AsteroidFieldSystem:cleanup(self.world)

    releaseGpuMemory()
end

--- Collect planets, labels, the star and the belts from the registry views.
function LimitTheoryRedux:collectEntities()
    self.planets = {}
    self.labeledEntities = {}
    self.beltEntities = {}
    self.starEntity = nil

    for entity in Registry:view(CelestialComponents.Planet) do
        table.insert(self.planets, entity)
        table.insert(self.labeledEntities, { entity = entity, label = tostring(entity) })
    end
    for entity in Registry:view(CelestialComponents.Moon) do
        table.insert(self.labeledEntities, { entity = entity, label = tostring(entity) })
    end
    for entity in Registry:view(CelestialComponents.Star) do
        self.starEntity = entity
    end
    for entity in Registry:view(CelestialComponents.AsteroidBelt) do
        table.insert(self.beltEntities, entity)
    end
end

--- The position belt asteroid data is relative to: the parent body when it
--- has a transform, else the belt's own transform.
---@param beltEntity Entity
---@return number x, number y, number z
local function beltRenderOrigin(beltEntity)
    local originEntity = beltEntity
    local parentCmp = beltEntity:get(CoreComponents.Parent)
    if parentCmp then
        local p = parentCmp:getParent()
        if p and p:get(PhysicsComponents.Transform) then originEntity = p end
    end
    local t = originEntity:get(PhysicsComponents.Transform)
    if not t then return 0, 0, 0 end
    local p = t:getPos()
    return p.x, p.y, p.z
end

--- The asteroid of any belt nearest to a point.
---@param belts Entity[]
---@param point Position
---@return table|nil asteroid Its baked data
---@return number x, number y, number z Its world position
local function nearestBeltAsteroid(belts, point)
    local best, bestDistSq, bx, by, bz = nil, math.huge, 0, 0, 0
    for _, beltEntity in ipairs(belts) do
        local beltCmp = beltEntity:get(CelestialComponents.AsteroidBeltData)
        local asteroids = beltCmp and beltCmp:getAsteroidData() or {}
        local ox, oy, oz = beltRenderOrigin(beltEntity)
        for i = 1, #asteroids do
            local a = asteroids[i]
            local dx = (ox + a.px) - point.x
            local dy = (oy + a.py) - point.y
            local dz = (oz + a.pz) - point.z
            local d = dx * dx + dy * dy + dz * dz
            if d < bestDistSq then
                best, bestDistSq = a, d
                bx, by, bz = ox + a.px, oy + a.py, oz + a.pz
            end
        end
    end
    return best, bx, by, bz
end

--- Where the player's ship starts when the scenario doesn't say: next to the
--- belt rock nearest the first planet (so the first thing the player sees is
--- a rock up close, with the ring/planet nearby); without rocks, just outside
--- the first planet; without planets, well clear of the star.
---@return Position
function LimitTheoryRedux:defaultShipSpawn()
    local cfg = self.cfg
    local planet = self.planets and self.planets[1]

    if planet then
        local planetPos = planet:get(PhysicsComponents.Transform):getPos()

        local a, ax, ay, az = nearestBeltAsteroid(self.beltEntities or {}, planetPos)
        if a then
            -- Just off the rock's surface, on the side away from the planet
            local clearDist = a.scale * 2.0
            local px, py, pz = planetPos.x, planetPos.y, planetPos.z
            local nx, ny, nz = ax - px, ay - py, az - pz
            local len = math.sqrt(nx * nx + ny * ny + nz * nz)
            if len > 1e-9 then nx, ny, nz = nx / len, ny / len, nz / len end
            return Position(ax + nx * clearDist, ay + ny * clearDist, az + nz * clearDist)
        end

        -- No rocks: just outside the planet, on its side away from the star
        local radius = planet:get(PhysicsComponents.Transform):getScale()
        local sx, sy, sz = 0, 0, 0
        if self.starEntity then
            local p = self.starEntity:get(PhysicsComponents.Transform):getPos()
            sx, sy, sz = p.x, p.y, p.z
        end
        local nx, ny, nz = planetPos.x - sx, planetPos.y - sy, planetPos.z - sz
        local len = math.sqrt(nx * nx + ny * ny + nz * nz)
        if len > 1e-9 then nx, ny, nz = nx / len, ny / len, nz / len else nx, ny, nz = 1, 0, 0 end
        local d = radius * cfg.shipSpawnOffset
        return Position(planetPos.x + nx * d, planetPos.y + ny * d, planetPos.z + nz * d)
    end

    -- No planets: well clear of the star (it sits at the origin otherwise)
    if self.starEntity then
        local t = self.starEntity:get(PhysicsComponents.Transform)
        local p = t:getPos()
        return Position(p.x + t:getScale() * cfg.starSpawnOffset, p.y, p.z)
    end
    return Position(0, 0, 0)
end

--- Spawn the player's ship where the scenario says, or at the default spot.
function LimitTheoryRedux:spawnPlayerShip()
    local spawnPos = self.scenario.spawnShip and self.scenario.spawnShip(self) or self:defaultShipSpawn()
    self:createPlayerShip(spawnPos, Enums.ShipType.Fighter, 1, self.rng:get31())
end

--- Build a flyable ship for the player.
---@param position Position
---@param shipType integer Enums.ShipType
---@param hullIdx integer index into Config.game.shipHulls (size)
---@param seed integer
function LimitTheoryRedux:createPlayerShip(position, shipType, hullIdx, seed)
    local hullCfg = Config.game.shipHulls

    self.playerShipHandle = self.constructManager:createShip({
        seed = seed,
        shipType = shipType,
        config = {
            position = position,
            scale = hullCfg.scale[hullIdx],
            isKinematic = false,
            -- The Legacy basic and capital generators size the hull by its
            -- index; the fighter's has its own size
            hull = shipType ~= Enums.ShipType.Fighter and hullIdx or nil,
            -- Mass, drag and thrust: Solo is the only flight profile so far
            flightProfile = Enums.FlightProfile.Solo,
        },
    })
    self.playerShip = self.playerShipHandle.root

    self.playerShip:add(ConstructComponents.TravelDrive())
    self.playerShip:add(ConstructComponents.AutoPilot())
    self.playerShip:add(ConstructComponents.ShipFlightControl())
    self.playerShip:add(ConstructComponents.ControlInput())
    self.playerShip:add(PhysicsComponents.GravityWell(
        ConstructComponents.FlightProfile.ProfileOf(self.playerShip).linearDrag))

    Log.Info("Player ship spawned at (%.1f, %.1f, %.1f)", position.x, position.y, position.z)
end

--- Swap the player's ship for a new one in the same place (a scenario's
--- showcase of hulls, say).
---@param shipType integer Enums.ShipType
---@param hullIdx integer index into Config.game.shipHulls
---@param seed integer
function LimitTheoryRedux:replacePlayerShip(shipType, hullIdx, seed)
    local oldBody = self.playerShip:get(PhysicsComponents.RigidBody):getRigidBody()
    local position = oldBody:getPos()
    -- The orbit camera goes back to following the ship (the new one)
    if self.player:getOrbitTarget() == self.playerShip then
        self.player:setOrbitTarget(nil)
    end
    self.constructManager:destroy(self.playerShipHandle)
    self:createPlayerShip(Position(position.x, position.y, position.z), shipType, hullIdx, seed)
    self.player:setShip(self.playerShip)
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

--- Fills the autopilot's overlay (target, route, progress) from the player's order and hands it to both
--- maps. Returns it, or nil while the autopilot is off.
---@return table|nil
function LimitTheoryRedux:updateAutopilotOverlay()
    local ship = self.playerShip
    local targetPos = AutoPilotSystem:getTargetPosition(ship)
    if not targetPos then
        self.mapState.autopilotOverlay = nil
        self.map3DState.autopilotOverlay = nil
        return nil
    end
    local overlay = autopilotOverlay
    overlay.targetPos = targetPos
    overlay.route = AutoPilotSystem:getRoute(ship, overlay.route)
    overlay.progress = AutoPilotSystem:getProgress(ship)
    overlay.planning = AutoPilotSystem:isPlanning(ship)
    overlay.shipPos = EntityPosition.Get(ship)
    self.mapState.autopilotOverlay = overlay
    self.map3DState.autopilotOverlay = overlay
    return overlay
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
    local overlay = self:updateAutopilotOverlay()

    -- The HUD text (In_Game view) hides with the HUD toggle and in the 3D map.
    UIViewInGame:setHudShown(not self.hudHidden and self.mapMode ~= 2)

    if self.mapMode == 2 then
        SystemMap3D:collectEntities(self.map3DState, self.playerShip)
        SystemMap3D:render(self.map3DState, data)

        self:immediateUI(function()
            -- The flare goes under the HUD and the pause menu
            LensFlareSystem:draw(self.starEntity, self.labeledEntities)
            Gui:draw()
            SystemMap3D:renderOverlay(self.map3DState, 0, 0, Window:width(), Window:height())
        end)
    else
        RenderCoreSystem:render(data)

        self:immediateUI(function()
            -- The flares go under the HUD and the pause menu
            LensFlareSystem:draw(self.starEntity, self.labeledEntities)
            ExplosionEffectSystem:drawFlares()
            Gui:draw()
            -- These draw after Gui:draw, so they'd sit on top of the pause menu
            -- and the Controls screen: hide them while those are open.
            local showHud = not self.hudHidden and not self.pauseMenuOpen
            -- Name labels next to planets, stations and the nearest asteroids:
            -- the Show HUD Indicator Names setting.
            local showLabels = showHud and GameState.ui.showIndicatorNames
            if showLabels then
                WorldLabelRenderSystem:draw(self.labeledEntities)
            end

            local spawned = AsteroidFieldSystem:getSpawnedEntities()
            if #spawned > 0 and showLabels then
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
                SystemMap:collectEntities(self.mapState, self.playerShip)
                SystemMap:draw(self.mapState, 0, 0, Window:width(), Window:height())
            elseif overlay and showHud and overlay.route and #overlay.route > 0 then
                -- Flight view: a marker on the next waypoint
                RouteOverlayRenderSystem:drawNextMarker(
                    ScreenProjection.active(flightProjector), overlay.route[1], overlay.shipPos)
            end
        end)
    end
end

---@param data EventData
function LimitTheoryRedux:onInput(data)
    if self:handleAppInput() then return end

    -- Regenerate waits a few frames between tearing down and building
    if self.rebuildInFrames then
        self.rebuildInFrames = self.rebuildInFrames - 1
        if self.rebuildInFrames <= 0 then
            self.rebuildInFrames = nil
            self:finishRegenerate()
        end
    end

    if not self.playerShip then return end
    local dt = data:deltaTime()

    -- Escape toggles the pause menu (Return to Game / Controls / Back to Main
    -- Menu / Exit); from the Controls screen it goes back to the pause menu.
    if Input:isPressed(ApplicationBindings.Escape) then
        if self.controlsOpen then
            self:closeControls()
        elseif self.pauseMenuOpen then
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
            SystemMap:collectEntities(self.mapState, self.playerShip)
            CursorManager:free()
        elseif self.mapMode == 2 then
            SystemMap3D:collectEntities(self.map3DState, self.playerShip)
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
                    -- A point on a belt keeps its place on the planet it goes round
                    local frame = sel._clickLocalX and sel._parentEntity or nil
                    AutoPilotSystem:engage(self.playerShip, sel.clickPos, frame)
                else
                    AutoPilotSystem:engage(self.playerShip, sel.entity)
                end
                self.mapMode = 0
                self.mapState.enabled = false
                self.mapClosedFrame = true
            end
        else
            SystemMap3D:updateInput(self.map3DState, dt)

            if GeneralActions.AutoPilotToggle:isPressed() and self.map3DState.selected then
                AutoPilotSystem:engage(self.playerShip, self.map3DState.selected.entity)
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

    if self.scenario.onInput then
        self.scenario.onInput(self, dt)
    end

    if GeneralActions.Regenerate:isPressed() then
        self:regenerateScenario()
    end

    if GeneralActions.ReloadShaders:isPressed() then
        local Material = require("Shared.Rendering.Material")
        Cache.ReloadShaders()
        Material.ReloadAll()
    end
end

--- PreSim: bookkeeping, then the writers of the ships' control input.
---@param data EventData
function LimitTheoryRedux:onPreSim(data)
    if not self.playerShip then return end
    local dt = data:deltaTime()

    AsteroidFieldSystem:update(dt, self.beltEntities, self.world, self.playerShip)
    SystemMap3D:updateTrails(self.map3DState, dt)
    MemoryReporter:update(dt)

    self.player:update(dt)
end

--- Sim: this handler owns the order, since the bus has none inside a stage. Bodies move, then
--- ships are actuated from their control input, then physics steps.
---@param data EventData
function LimitTheoryRedux:onSim(data)
    if not self.playerShip then return end
    local dt = data:deltaTime()

    -- Static star system: skip orbital motion so bodies stay at their
    -- generated positions (deterministic scene while orbital mechanics
    -- are still being developed). Asteroid fields still follow parents.
    if not self.staticStarSystem then
        OrbitalSystem:update(self.orbiters, dt, self.followers)
    end
    AsteroidFieldSystem:updatePositions()
    GravityWellSystem:update(dt, self.world:getSolverSubsteps())

    TravelDriveSystem:update(dt)
    ShipFlightSystem:update()
    if self.scenario.update then self.scenario.update(self, dt) end
    -- Weapons: targets and turrets before the physics step, AI, aim and
    -- firing right after it (shots in flight: onPostSim)
    self.weaponCombat:preUpdate(dt)
    self.world:update(dt)
    self.weaponCombat:update(dt)
    if self.scenario.postUpdate then self.scenario.postUpdate(self, dt) end
end

---@param data EventData
function LimitTheoryRedux:onPostSim(data)
    if not self.playerShip then return end
    -- Projectiles, beams, expired effect lights
    self.weaponCombat:postUpdate(data:deltaTime())
    -- Explosions age, and go when they are done
    ExplosionEffectSystem:update(data:deltaTime())
end

--- Regenerate (B): what the scenario does for it, or by default a new seed
--- and everything built again.
function LimitTheoryRedux:regenerateScenario()
    if self.scenario.regenerate then
        self.scenario.regenerate(self)
    else
        self.seed = self.rng:get31()
        self:regenerate()
    end
end

--- Build the star system, the player's ship, the stations and the scenario's
--- things again from `self.seed` (Regenerate picks a new seed first).
---
--- The old scene goes now and the new one comes a few frames later: the GPU
--- frees the old scene's textures only once the frames that drew them are
--- done, and building at once would need room for both (a Standard Solar
--- System alone takes about 4 GB).
function LimitTheoryRedux:regenerate()
    self:destroyScene()
    UIViewInGame:setPlayer(nil) -- no ship for the HUD meanwhile
    self.rebuildInFrames = REBUILD_DELAY_FRAMES
end

--- The second half of `regenerate`: build the scene again.
function LimitTheoryRedux:finishRegenerate()
    self.rng = RNG.Create(self.seed)
    self:attachNavigation()
    self:buildScene()
    self.player:setShip(self.playerShip)
    UIViewInGame:setPlayer(self.player)
    self:setupScenario()
    self:collectSceneSystems()
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
