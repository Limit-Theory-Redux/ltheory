local CameraController = require("Modules.Cameras.Managers").CameraController

local PhysicsComponents = require("Modules.Physics.Components")

--todo: fix when right clicking it jumps the camera (does not happpen if mouse has delta when right clicking??)

---@class OrbitCameraController : CameraController
---@field target Entity|nil The entity to orbit around
---@field targetOffset Vec3f Offset from target position
---@field distance number Current distance from target
---@field minDistance number Minimum zoom distance
---@field maxDistance number Maximum zoom distance
---@field zoomSpeed number Zoom speed multiplier
---@field mouseSensitivity number Mouse look sensitivity
---@field yaw number Yaw angle in radians (around world Y)
---@field pitch number Pitch angle in radians
---@field minPitch number Minimum pitch angle in radians
---@field maxPitch number Maximum pitch angle in radians
---@field smoothing number Camera smoothing factor
---@field currentDistance number Smoothed distance for camera lag
---@field mouseCaptured boolean Whether the mouse is currently captured
---@field lastMousePos Vec2f|nil Last mouse position when captured
---@field justCaptured boolean Whether the mouse was just captured this frame
---@field wasCapturedLastFrame boolean Whether the mouse was captured last frame
---@overload fun(self: OrbitCameraController, entity: Entity, config?: table): OrbitCameraController
---@overload fun(entity: Entity, config?: table): OrbitCameraController
local OrbitCameraController = Subclass("OrbitCameraController", CameraController, function(self, entity, config)
    self:initController(entity)

    Input:setCursorVisible(true) -- Start with cursor visible

    config = config or {}

    -- Target settings
    self.target = nil -- Set via setTarget()
    self.targetOffset = config.targetOffset or Vec3f(0, 0, 0)

    -- Distance settings
    self.distance = config.distance or 20.0
    self.minDistance = config.minDistance or 2.0
    self.maxDistance = config.maxDistance or 100.0
    self.zoomSpeed = config.zoomSpeed or 5.0

    -- Rotation settings
    self.mouseSensitivity = config.mouseSensitivity or 0.003
    self.yaw = config.initialYaw or 0.0
    self.pitch = config.initialPitch or 0.3
    self.minPitch = config.minPitch or -math.pi / 2 + 0.01 -- Almost straight down
    self.maxPitch = config.maxPitch or math.pi / 2 - 0.01  -- Almost straight up

    -- Smoothing
    self.smoothing = config.smoothing or 0.1
    self.currentDistance = self.distance

    -- Mouse capture
    self.mouseCaptured = false
    self.lastMousePos = nil
    self.justCaptured = false
    self.wasCapturedLastFrame = false
end)

---Set the target entity to orbit around
---@param target Entity|nil The entity to orbit
function OrbitCameraController:setTarget(target)
    self.target = target

    -- Initialize camera position
    self:updateCameraPosition(0)
end

---Set offset from target position
---@param offset Vec3f Offset vector
function OrbitCameraController:setTargetOffset(offset)
    self.targetOffset = offset
end

---Set orbit distance
---@param distance number Distance from target
function OrbitCameraController:setDistance(distance)
    self.distance = Math.Clamp(distance, self.minDistance, self.maxDistance)
end

---@param dt number
function OrbitCameraController:onInput(dt)
    if not self.enabled or not Window:isFocused() then return end

    -- Toggle mouse capture with right click
    if Input:mouse():isPressed(MouseControl.Right) then
        self.mouseCaptured = not self.mouseCaptured
        self.justCaptured = self.mouseCaptured
    end

    if self.mouseCaptured then
        GameState.render.gameWindow:cursor():setGrabMode(CursorGrabMode.Locked)
        Input:setCursorVisible(false)

        local delta = Input:mouse():delta()
        if delta:length() > 0.001 then
            self.yaw   = self.yaw - delta.x * self.mouseSensitivity
            self.pitch = self.pitch - delta.y * self.mouseSensitivity
            self.pitch = Math.Clamp(self.pitch, self.minPitch, self.maxPitch)
            self:updateCameraPosition(dt)
        end
    else
        GameState.render.gameWindow:cursor():setGrabMode(CursorGrabMode.None)
        Input:setCursorVisible(true)
    end

    -- Zoom with mouse wheel
    local scroll = Input:mouse():value(MouseControl.ScrollY)
    if math.abs(scroll) > 0.001 then
        self.distance = Math.Clamp(self.distance - scroll * self.zoomSpeed, self.minDistance, self.maxDistance)
    end
end

---Update function called every frame
---@param dt number Delta time
function OrbitCameraController:onPreRender(dt)
    if not self.enabled then return end
    self:updateCameraPosition(dt)
end

---Update camera position based on target and angles (WORLD SPACE ORBIT)
---@param dt number Delta time
function OrbitCameraController:updateCameraPosition(dt)
    if not self.target then return end

    -- Smooth distance interpolation
    local smoothFactor = 1.0 - math.exp(-10.0 * dt * (1.0 / self.smoothing))
    self.currentDistance = self.currentDistance + (self.distance - self.currentDistance) * smoothFactor

    -- Get target position
    local targetPos = Position(0, 0, 0)
    local rbCmp = self.target:get(PhysicsComponents.RigidBody)
    if rbCmp then
        targetPos = rbCmp:getRigidBody():getPos()
    end

    -- Apply target offset (in world space)
    local offsetTarget = Position(
        targetPos.x + self.targetOffset.x,
        targetPos.y + self.targetOffset.y,
        targetPos.z + self.targetOffset.z
    )

    -- Calculate camera position using spherical coordinates in WORLD SPACE
    -- Yaw rotates around world Y axis
    -- Pitch tilts up/down
    local x = math.cos(self.pitch) * math.sin(self.yaw)
    local y = math.sin(self.pitch)
    local z = math.cos(self.pitch) * math.cos(self.yaw)

    -- Position camera at distance from target
    local camPos = Position(
        offsetTarget.x + x * self.currentDistance,
        offsetTarget.y + y * self.currentDistance,
        offsetTarget.z + z * self.currentDistance
    )

    self:setPosition(camPos)

    -- Look at target from camera position
    local lookDir = Vec3f(
        offsetTarget.x - camPos.x,
        offsetTarget.y - camPos.y,
        offsetTarget.z - camPos.z
    ):normalize()

    -- Use world up for camera orientation
    local worldUp = Vec3f(0, 1, 0)
    local rot = Quat.FromLook(lookDir, worldUp)
    self:setRotation(rot)
end

---Get current orbit angles
---@return number yaw, number pitch
function OrbitCameraController:getAngles()
    return self.yaw, self.pitch
end

---Set orbit angles directly
---@param yaw number Yaw angle in radians
---@param pitch number Pitch angle in radians
function OrbitCameraController:setAngles(yaw, pitch)
    self.yaw = yaw
    self.pitch = Math.Clamp(pitch, self.minPitch, self.maxPitch)
end

return OrbitCameraController
