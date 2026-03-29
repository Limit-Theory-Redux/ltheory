local CameraEntity            = require("Modules.Cameras.Entities").Camera
local CameraManager           = require("Modules.Cameras.Managers.CameraManager")
local _                       = require("Modules.Cameras.Systems.CameraSystem") -- subscribes to Input + PreRender
local CameraDataComponent     = require("Modules.Cameras.Components.CameraDataComponent")
local ChaseCameraController   = require("Modules.Cameras.Managers.CameraControllers.ChaseCameraController")
local OrbitCameraController   = require("Modules.Cameras.Managers.CameraControllers.OrbitCameraController")
local FirstPersonCameraController = require("Modules.Cameras.Managers.CameraControllers.FirstPersonCameraController")
local FreeCameraController    = require("Modules.Cameras.Managers.CameraControllers.FreeCameraController")
local RTSCameraController     = require("Modules.Cameras.Managers.CameraControllers.RTSCameraController")
local PhysicsComponents       = require("Modules.Physics.Components")
local ConstructComponents     = require("Modules.Constructs.Components")
local RenderComp              = require("Modules.Rendering.Components").Render
local ShipFlightSystem        = require("Modules.Constructs.Systems.ShipFlightSystem")
local TravelDriveSystem       = require("Modules.Constructs.Systems.TravelDriveSystem")
local AutoPilotSystem         = require("Modules.Constructs.Systems.AutoPilotSystem")

---@class PlayerController
---@overload fun(shipEntity: Entity, config?: table): PlayerController
local PlayerController = Class("PlayerController", function(self, shipEntity, config)
    config = config or {}
    self.ship = shipEntity
    self.mode = Enums.CameraMode.Chase
    self.controllers = {}
    self.cameraEntities = {}

    -- Modes where the player is piloting the ship (flight input active)
    self.pilotingModes = {
        [Enums.CameraMode.FirstPerson] = true,
        [Enums.CameraMode.Chase]       = true,
    }

    -- Modes to cycle through with the mode switch key
    self.cycleModes = config.cycleModes or {
        Enums.CameraMode.Chase,
        Enums.CameraMode.FirstPerson,
        Enums.CameraMode.Orbit,
        Enums.CameraMode.Free,
    }

    self:_createCameras(config)
    self:setMode(self.mode)
end)

--- Create all camera entities and controllers
---@param config table
function PlayerController:_createCameras(config)
    local cc = config.chase or {}
    local fp = config.firstPerson or {}
    local orb = config.orbit or {}
    local free = config.free or {}
    local rts = config.rts or {}

    -- Chase camera (behind ship, legacy CameraChase style)
    local chaseCfg = Config.game.chaseCamera or {}
    local camChase = CameraEntity()
    CameraManager:registerCamera("PlayerChase", camChase)
    self.controllers[Enums.CameraMode.Chase] = ChaseCameraController(camChase, {
        posRel    = cc.posRel or Vec3f(0, 2.5, -10),
        lookAtRel = cc.lookAtRel or Vec3f(0, 0, 1000),
        radius    = chaseCfg.defaultRadius or 1.0,
        minRadius = chaseCfg.minRadius or 0.5,
        maxRadius = chaseCfg.maxRadius or 8.0,
        zoomSpeed = chaseCfg.zoomSpeed or 0.15,
    })
    camChase:get(CameraDataComponent):setController(self.controllers[Enums.CameraMode.Chase])
    self.controllers[Enums.CameraMode.Chase]:setTarget(self.ship)
    self.cameraEntities[Enums.CameraMode.Chase] = camChase

    -- First person camera (cockpit)
    local camFP = CameraEntity()
    CameraManager:registerCamera("PlayerFirstPerson", camFP)
    self.controllers[Enums.CameraMode.FirstPerson] = FirstPersonCameraController(camFP, {
        eyeOffset        = fp.eyeOffset or Vec3f(0, 0.5, 1.5),
        mouseSensitivity = fp.mouseSensitivity or 0.003,
        followRotation   = true,
        smoothing        = fp.smoothing or 0.05,
    })
    camFP:get(CameraDataComponent):setController(self.controllers[Enums.CameraMode.FirstPerson])
    self.controllers[Enums.CameraMode.FirstPerson]:setTarget(self.ship)
    self.cameraEntities[Enums.CameraMode.FirstPerson] = camFP

    -- Orbit camera (free orbit, wider range)
    local camOrbit = CameraEntity()
    CameraManager:registerCamera("PlayerOrbit", camOrbit)
    self.controllers[Enums.CameraMode.Orbit] = OrbitCameraController(camOrbit, {
        distance       = orb.distance or 50,
        minDistance     = orb.minDistance or 5,
        maxDistance     = orb.maxDistance or 500,
        zoomSpeed      = orb.zoomSpeed or 10.0,
        smoothing      = orb.smoothing or 0.1,
        mouseSensitivity = orb.mouseSensitivity or 0.003,
    })
    camOrbit:get(CameraDataComponent):setController(self.controllers[Enums.CameraMode.Orbit])
    self.controllers[Enums.CameraMode.Orbit]:setTarget(self.ship)
    self.cameraEntities[Enums.CameraMode.Orbit] = camOrbit

    -- Free camera (detached)
    local camFree = CameraEntity()
    CameraManager:registerCamera("PlayerFree", camFree)
    self.controllers[Enums.CameraMode.Free] = FreeCameraController(camFree, {
        moveSpeed        = free.moveSpeed or 100.0,
        fastMultiplier   = free.fastMultiplier or 10.0,
        mouseSensitivity = free.mouseSensitivity or 0.003,
    })
    camFree:get(CameraDataComponent):setController(self.controllers[Enums.CameraMode.Free])
    self.cameraEntities[Enums.CameraMode.Free] = camFree

    -- RTS camera (strategy view)
    local camRTS = CameraEntity()
    CameraManager:registerCamera("PlayerRTS", camRTS)
    self.controllers[Enums.CameraMode.RTS] = RTSCameraController(camRTS, {
        height     = rts.height or 500,
        minHeight  = rts.minHeight or 10,
        maxHeight  = rts.maxHeight or 100000,
        moveSpeed  = rts.moveSpeed or 50.0,
        smoothing  = rts.smoothing or 0.12,
    })
    camRTS:get(CameraDataComponent):setController(self.controllers[Enums.CameraMode.RTS])
    self.cameraEntities[Enums.CameraMode.RTS] = camRTS
end

local modeToCamera = {
    [Enums.CameraMode.Chase]       = "PlayerChase",
    [Enums.CameraMode.FirstPerson] = "PlayerFirstPerson",
    [Enums.CameraMode.Orbit]       = "PlayerOrbit",
    [Enums.CameraMode.Free]        = "PlayerFree",
    [Enums.CameraMode.RTS]         = "PlayerRTS",
}

--- Switch to a camera mode
---@param mode integer Enums.CameraMode value
function PlayerController:setMode(mode)
    -- Disable all controllers
    for _, controller in pairs(self.controllers) do
        controller:disable()
    end

    -- Transfer position context when switching
    local oldController = self.controllers[self.mode]
    local newController = self.controllers[mode]

    if oldController and newController and mode == Enums.CameraMode.Free then
        newController:setPosition(oldController:getPosition())
        newController:setRotation(oldController:getRotation())
    end

    if mode == Enums.CameraMode.RTS then
        local rbCmp = self.ship:get(PhysicsComponents.RigidBody)
        if rbCmp and rbCmp:getRigidBody() then
            local pos = rbCmp:getRigidBody():getPos()
            newController:setFocusPoint(Vec3f(pos.x, pos.y, pos.z))
        end
    end

    self.mode = mode
    newController:enable()
    CameraManager:setActiveCamera(modeToCamera[mode])

    -- Manage cursor and ship visibility based on mode
    local renderCmp = self.ship:get(RenderComp)
    if mode == Enums.CameraMode.FirstPerson then
        GameState.render.gameWindow:cursor():setGrabMode(CursorGrabMode.Locked)
        Input:setCursorVisible(false)
        if renderCmp then renderCmp:setVisible(false) end
    elseif mode == Enums.CameraMode.Chase then
        GameState.render.gameWindow:cursor():setGrabMode(CursorGrabMode.Confined)
        Input:setCursorVisible(true)
        if renderCmp then renderCmp:setVisible(true) end
    else
        GameState.render.gameWindow:cursor():setGrabMode(CursorGrabMode.None)
        Input:setCursorVisible(true)
        if renderCmp then renderCmp:setVisible(true) end
    end

    Log.Info("Camera mode: %s", Enums.CameraModeNames[mode])
end

--- Get the current camera mode
---@return integer mode Enums.CameraMode value
function PlayerController:getMode()
    return self.mode
end

--- Get the current mode name
---@return string
function PlayerController:getModeName()
    return Enums.CameraModeNames[self.mode] or "Unknown"
end

--- Cycle to the next camera mode
function PlayerController:cycleMode()
    local currentIdx = 1
    for i, mode in ipairs(self.cycleModes) do
        if mode == self.mode then
            currentIdx = i
            break
        end
    end
    local nextIdx = (currentIdx % #self.cycleModes) + 1
    self:setMode(self.cycleModes[nextIdx])
end

--- Returns true if the player is currently piloting (flight input should be active)
---@return boolean
function PlayerController:isPiloting()
    return self.pilotingModes[self.mode] == true
end

--- Get the active camera controller
---@return CameraController
function PlayerController:getActiveController()
    return self.controllers[self.mode]
end

--- Set whether input is blocked (e.g. when system map is open)
---@param blocked boolean
function PlayerController:setInputBlocked(blocked)
    self.inputBlocked = blocked
end

--- Update the player controller (call each frame from onSim)
---@param dt number Delta time
function PlayerController:update(dt)
    -- Travel drive always updates (even with map open — ship keeps moving)
    local ShipActions = require("Input.ActionBindings.ShipActions")
    for _, binding in pairs(ShipActions) do
        binding:update(dt)
    end
    TravelDriveSystem:update(dt, self.ship)

    -- Don't process other input when blocked (map open, menu, etc.)
    if self.inputBlocked then return end

    -- Handle camera mode cycling
    if Input:isPressed(Button.KeyboardC) then
        self:cycleMode()
    end

    -- AutoPilot overrides manual flight
    AutoPilotSystem:update(dt, self.ship)

    -- Manual flight only when piloting AND not on autopilot
    if self:isPiloting() and not AutoPilotSystem:isActive(self.ship) then
        local isFPS = (self.mode == Enums.CameraMode.FirstPerson)
        local drive = self.ship:get(ConstructComponents.TravelDrive)
        if drive and drive:isActive() then
            ShipFlightSystem:updateSteeringOnly(dt, self.ship, isFPS)
        else
            ShipFlightSystem:update(dt, self.ship, isFPS, isFPS)
        end
    end
end

--- Get the player ship entity
---@return Entity
function PlayerController:getShip()
    return self.ship
end

--- Set a new ship for the player to control
---@param shipEntity Entity
function PlayerController:setShip(shipEntity)
    self.ship = shipEntity

    local chase = self.controllers[Enums.CameraMode.Chase]
    if chase then chase:setTarget(shipEntity) end

    local fp = self.controllers[Enums.CameraMode.FirstPerson]
    if fp then fp:setTarget(shipEntity) end

    local orbit = self.controllers[Enums.CameraMode.Orbit]
    if orbit then orbit:setTarget(shipEntity) end
end

return PlayerController
