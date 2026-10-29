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
local AutoPilotSystem         = require("Modules.Constructs.Systems.AutoPilotSystem")
local CursorManager           = require("Input.CursorManager")
local GeneralActions          = require("Input.ActionBindings.GeneralActions")
local ShipActions             = require("Input.ActionBindings.ShipActions")

-- Fire input above this stops the travel drive.
local FIRE_THRESHOLD           = 0.5
-- Aiming dead zones, as fractions of the full axis.
local AIM_DEAD_ZONE_YAW        = 0.004
local AIM_DEAD_ZONE_PITCH      = 0.008
-- First person: cursor movement as a fraction of the window accumulates into the aim
-- target (limited to FPS_AIM_LIMIT), which decays toward centre at FPS_AIM_DECAY_RATE per second.
local FPS_AIM_SENSITIVITY      = 1.0
local FPS_AIM_LIMIT            = 0.8
local FPS_AIM_DECAY_RATE       = 4.0
-- Steering while the travel drive is active: window fraction moved to full axis.
local DRIVE_STEER_SENSITIVITY  = 4.0
local DRIVE_STEER_DEAD_ZONE    = 0.0001

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
        CursorManager:locked()
        if renderCmp then renderCmp:setVisible(false) end
    elseif mode == Enums.CameraMode.Chase then
        CursorManager:confined()
        if renderCmp then renderCmp:setVisible(true) end
    else
        CursorManager:free()
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

--- Request the travel drive: the toggle key engages an idle drive and stops a running
--- one, and firing stops it. Applied by the drive system on its next update.
---@param input ControlInputComponent
function PlayerController:requestDrive(input)
    local drive = self.ship:get(ConstructComponents.TravelDrive)
    if not drive then return end

    if ShipActions.TravelDrive:isPressed() then
        input:setDriveRequest(drive:getState() == "idle"
            and Enums.TravelDriveRequest.Engage
            or Enums.TravelDriveRequest.Disengage)
    end
    if ShipActions.Fire:get() > FIRE_THRESHOLD then
        input:setDriveRequest(Enums.TravelDriveRequest.Disengage)
    end
end

--- Mouse steering while the drive is active: by cursor movement in every camera mode.
---@return ControlAxis yaw, ControlAxis pitch
local function driveSteerAxes()
    local delta = Input:mouse():delta()
    local yaw   = Math.Clamp(delta.x / Window:width() * DRIVE_STEER_SENSITIVITY, -1.0, 1.0)
    local pitch = Math.Clamp(-delta.y / Window:height() * DRIVE_STEER_SENSITIVITY, -1.0, 1.0)
    if math.abs(yaw)   < DRIVE_STEER_DEAD_ZONE then yaw   = 0 end
    if math.abs(pitch) < DRIVE_STEER_DEAD_ZONE then pitch = 0 end
    return yaw, pitch
end

--- First-person aiming: cursor movement accumulates into a target that decays back
--- to centre when the mouse stops.
---@param dt number
---@param flightCtrl ShipFlightControlComponent|nil
---@return ControlAxis yaw, ControlAxis pitch
local function firstPersonAxes(dt, flightCtrl)
    if not flightCtrl then return 0, 0 end

    local delta = Input:mouse():delta()
    local yaw = Math.Clamp(
        flightCtrl:getTargetYaw() + delta.x / Window:width() * FPS_AIM_SENSITIVITY,
        -FPS_AIM_LIMIT, FPS_AIM_LIMIT)
    local pitch = Math.Clamp(
        flightCtrl:getTargetPitch() - delta.y / Window:height() * FPS_AIM_SENSITIVITY,
        -FPS_AIM_LIMIT, FPS_AIM_LIMIT)

    local decay = math.exp(-FPS_AIM_DECAY_RATE * dt)
    flightCtrl:setTargetYaw(yaw * decay)
    flightCtrl:setTargetPitch(pitch * decay)
    return yaw, pitch
end

--- Write the manual flight axes from the player's input devices.
---@param dt number
---@param input ControlInputComponent
function PlayerController:writeAxes(dt, input)
    local drive = self.ship:get(ConstructComponents.TravelDrive)
    local yaw, pitch
    if drive and drive:isActive() then
        yaw, pitch = driveSteerAxes()
    else
        if self.mode == Enums.CameraMode.FirstPerson then
            yaw, pitch = firstPersonAxes(dt, self.ship:get(ConstructComponents.ShipFlightControl))
        else
            -- Chase: the cursor's position on screen steers.
            yaw, pitch = ShipActions.Yaw:get(), ShipActions.Pitch:get()
        end
        if math.abs(yaw)   < AIM_DEAD_ZONE_YAW   then yaw   = 0 end
        if math.abs(pitch) < AIM_DEAD_ZONE_PITCH then pitch = 0 end

        local cfg = Config.game.shipFlight
        if cfg.invertYaw   then yaw   = -yaw end
        if cfg.invertPitch then pitch = -pitch end
    end

    input:setThrustForward(ShipActions.ThrustZ:get())
    input:setThrustRight(ShipActions.ThrustX:get())
    input:setThrustUp(ShipActions.ThrustY:get())
    input:setRoll(ShipActions.Roll:get())
    input:setBoost(ShipActions.Boost:get())
    input:setYaw(yaw)
    input:setPitch(pitch)
end

--- Update the player controller (call each frame from onSim, before the actuation systems)
---@param dt number Delta time
function PlayerController:update(dt)
    for _, binding in pairs(ShipActions) do
        binding:update(dt)
    end

    -- The drive is requested even with the map open: the ship keeps moving.
    local input = self.ship:get(ConstructComponents.ControlInput)
    if input then
        self:requestDrive(input)
    end

    -- AutoPilot always updates (even with map open — ship keeps navigating)
    AutoPilotSystem:update(dt, self.ship)

    -- Don't process other input when blocked (map open, menu, etc.)
    if self.inputBlocked then
        if input then input:clearAxes() end
        return
    end

    -- Handle camera mode cycling
    GeneralActions.CycleCamera:update(dt)
    if GeneralActions.CycleCamera:isPressed() then
        self:cycleMode()
    end

    -- Manual flight only when piloting AND not on autopilot
    if input then
        if self:isPiloting() and not AutoPilotSystem:isActive(self.ship) then
            self:writeAxes(dt, input)
        else
            input:clearAxes()
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
    -- The old ship must not keep flying on its last input.
    local oldInput = self.ship and self.ship:get(ConstructComponents.ControlInput)
    if oldInput then oldInput:clearAxes() end

    self.ship = shipEntity

    local chase = self.controllers[Enums.CameraMode.Chase]
    if chase then chase:setTarget(shipEntity) end

    local fp = self.controllers[Enums.CameraMode.FirstPerson]
    if fp then fp:setTarget(shipEntity) end

    local orbit = self.controllers[Enums.CameraMode.Orbit]
    if orbit then orbit:setTarget(shipEntity) end
end

return PlayerController
