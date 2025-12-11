local CameraController = require("Modules.Cameras.Managers").CameraController
local PhysicsComponents = require("Modules.Physics.Components")

---@class FirstPersonCameraController : CameraController
---@field target Entity|nil The entity to attach to
---@field eyeOffset Vec3f Offset from target position (eye height)
---@field mouseSensitivity number Mouse look sensitivity
---@field yaw number Current yaw angle in radians
---@field pitch number Current pitch angle in radians
---@field minPitch number Minimum pitch angle in radians
---@field maxPitch number Maximum pitch angle in radians
---@field smoothing number Camera smoothing factor
---@field currentYaw number Smoothed yaw
---@field currentPitch number Smoothed pitch
---@field mouseCaptured boolean Whether the mouse is currently captured
---@field lastMousePos Vec2f|nil Last mouse position when captured
---@field justCaptured boolean Whether the mouse was just captured this frame
---@field wasCapturedLastFrame boolean Whether the mouse was captured last frame
---@field followRotation boolean Whether to follow target's rotation
---@overload fun(self: FirstPersonCameraController, entity: Entity, config?: table): FirstPersonCameraController
---@overload fun(entity: Entity, config?: table): FirstPersonCameraController
local FirstPersonCameraController = Subclass("FirstPersonCameraController", CameraController, function(self, entity, config)
    self:initController(entity)

    Input:setCursorVisible(false)

    config = config or {}

    -- Target settings
    self.target = nil                                     -- Set via setTarget()
    self.eyeOffset = config.eyeOffset or Vec3f(0, 1.8, 0) -- Default eye height

    -- Rotation settings
    self.mouseSensitivity = config.mouseSensitivity or 0.003
    self.yaw = config.initialYaw or 0.0
    self.pitch = config.initialPitch or 0.0
    self.minPitch = config.minPitch or -math.pi / 2 + 0.1
    self.maxPitch = config.maxPitch or math.pi / 2 - 0.1

    -- Smoothing
    self.smoothing = config.smoothing or 0.05
    self.currentYaw = self.yaw
    self.currentPitch = self.pitch

    -- Follow target rotation
    self.followRotation = config.followRotation or false

    -- Mouse capture
    self.mouseCaptured = false
    self.lastMousePos = nil
    self.justCaptured = false
    self.wasCapturedLastFrame = false
end)

---Set the target entity to attach to
---@param target Entity The entity to follow
function FirstPersonCameraController:setTarget(target)
    self.target = target
    self:updateCameraPosition(0)
end

---Set eye offset from target position
---@param offset Vec3f Offset vector (typically Y is eye height)
function FirstPersonCameraController:setEyeOffset(offset)
    self.eyeOffset = offset
end

---Enable or disable following target's rotation
---@param follow boolean Whether to follow target rotation
function FirstPersonCameraController:setFollowRotation(follow)
    self.followRotation = follow
end

---@param dt number
function FirstPersonCameraController:onInput(dt)
    if not self.enabled or not Window:isFocused() then return end
    -- something?
end

function FirstPersonCameraController:onPreRender(dt)
    if not self.enabled then return end
    -- Update camera position
    self:updateCameraPosition(dt)
end

---Update camera position based on target
---@param dt number Delta time
function FirstPersonCameraController:updateCameraPosition(dt)
    if not self.target then return end

    -- Smooth interpolation for look angles
    local smoothFactor = 1.0 - math.exp(-10.0 * dt * (1.0 / self.smoothing))
    self.currentYaw = self.currentYaw + (self.yaw - self.currentYaw) * smoothFactor
    self.currentPitch = self.currentPitch + (self.pitch - self.currentPitch) * smoothFactor

    -- Get target position and rotation
    local targetPos = Position(0, 0, 0)
    local rbCmp = self.target:get(PhysicsComponents.RigidBody)
    if rbCmp then
        targetPos = rbCmp:getRigidBody():getPos()
    end

    -- Calculate eye position
    local eyePos = targetPos

    if self.followRotation then
        -- Rotate eye offset by target rotation
        local rotatedOffset = targetRot:rotate(self.eyeOffset)
        eyePos = Position(
            targetPos.x + rotatedOffset.x,
            targetPos.y + rotatedOffset.y,
            targetPos.z + rotatedOffset.z
        )

        -- Combine target yaw with camera yaw
        local targetEuler = targetRot:toEuler()
        local combinedYaw = targetEuler.y + self.currentYaw
        local rot = Quat.FromEuler(self.currentPitch, combinedYaw, 0)
        self:setRotation(rot)
    else
        -- Simple offset without rotation
        eyePos = Position(
            targetPos.x + self.eyeOffset.x,
            targetPos.y + self.eyeOffset.y,
            targetPos.z + self.eyeOffset.z
        )

        -- Independent camera rotation
        local rot = Quat.FromEuler(self.currentPitch, self.currentYaw, 0)
        self:setRotation(rot)
    end

    self:setPosition(eyePos)
end

---Get current look angles
---@return number yaw, number pitch
function FirstPersonCameraController:getAngles()
    return self.yaw, self.pitch
end

---Set look angles directly
---@param yaw number Yaw angle in radians
---@param pitch number Pitch angle in radians
function FirstPersonCameraController:setAngles(yaw, pitch)
    self.yaw = yaw
    self.pitch = Math.Clamp(pitch, self.minPitch, self.maxPitch)
end

---Get the current look direction vector
---@return Vec3f Look direction (normalized)
function FirstPersonCameraController:getLookDirection()
    local rot = self:getRotation()
    return rot:getForward()
end

return FirstPersonCameraController
