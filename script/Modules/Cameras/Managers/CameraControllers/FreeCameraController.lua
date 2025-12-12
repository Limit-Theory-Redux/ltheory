local CameraController = require("Modules.Cameras.Managers").CameraController

---@class FreeCameraController : CameraController
---@field moveSpeed number Base movement speed
---@field fastMultiplier number Speed multiplier when fast mode is active
---@field slowMultiplier number Speed multiplier when slow mode is active
---@field mouseSensitivity number Mouse look sensitivity
---@field rollSpeed number Roll rotation speed
---@field yaw number Current yaw angle in radians
---@field pitch number Current pitch angle in radians
---@field roll number Current roll angle in radians
---@field velocity Vec3f Current velocity for smoothing
---@field smoothing number Movement smoothing factor
---@field allowRoll boolean Whether roll is enabled
---@field moveInput Vec3f Current movement input direction
---@field mouseCaptured boolean Whether the mouse is currently captured
---@field lastMousePos Vec2f|nil Last mouse position when captured
---@field justCaptured boolean Whether the mouse was just captured this frame
---@field wasCapturedLastFrame boolean Whether the mouse was captured last frame
---@overload fun(self: FreeCameraController, entity: Entity, config?: table): FreeCameraController
---@overload fun(entity: Entity, config?: table): FreeCameraController
local FreeCameraController = Subclass("FreeCameraController", CameraController, function(self, entity, config)
    self:initController(entity)

    config = config or {}

    -- Movement settings
    self.moveSpeed = config.moveSpeed or 20.0
    self.fastMultiplier = config.fastMultiplier or 5.0
    self.slowMultiplier = config.slowMultiplier or 0.1
    self.mouseSensitivity = config.mouseSensitivity or 1
    self.rollSpeed = config.rollSpeed or 2.0

    -- Rotation state
    self.yaw = 0.0
    self.pitch = 0.0
    self.roll = 0.0
    self.allowRoll = config.allowRoll ~= false -- Default true

    -- Movement smoothing
    self.smoothing = config.smoothing or 0.1
    self.velocity = Vec3f(0, 0, 0)
    self.moveInput = Vec3f(0, 0, 0)

    self.mouseCaptured = false

    -- Initialize from current transform
    local rot = self:getRotation()
    local euler = rot:toEuler()
    self.yaw = euler.y
    self.pitch = euler.x
    self.roll = euler.z
end)

---* all of this since mouse confinement doesn´t seem to work properly *
---@param dt number
function FreeCameraController:onInput(dt)
    if not self.enabled or not Window:isFocused() then return end

    -- Axis invert configuration: 1 = normal, -1 = inverted
    local invertX = -1
    local invertY = -1

    -- Toggle mouse capture
    if Input:mouse():isPressed(MouseControl.Right) then
        self.mouseCaptured = not self.mouseCaptured
    end

    if self.mouseCaptured then
        GameState.render.gameWindow:cursor():setGrabMode(CursorGrabMode.Locked)
        Input:setCursorVisible(false)

        local delta = Input:mouse():delta()

        if delta:length() > 0.001 then
            -- Apply inverted axes and sensitivity
            local yawDelta   = delta.x * invertX * self.mouseSensitivity * dt
            local pitchDelta = delta.y * invertY * self.mouseSensitivity * dt

            self:rotate(yawDelta, pitchDelta, 0)
        end
    else
        GameState.render.gameWindow:cursor():setGrabMode(CursorGrabMode.None)
        Input:setCursorVisible(true)
    end


    -- Roll control (optional)
    if self.allowRoll then
        if Input:keyboard():isDown(Button.KeyboardQ) then
            self:rotate(0, 0, -self.rollSpeed * dt) -- needs to be inverted
        end
        if Input:keyboard():isDown(Button.KeyboardE) then
            self:rotate(0, 0, self.rollSpeed * dt) -- needs to be inverted
        end
    end

    -- Movement input
    local moveDir = Vec3f(0, 0, 0)
    if Input:keyboard():isDown(Button.KeyboardW) then moveDir.z = moveDir.z + 1 end
    if Input:keyboard():isDown(Button.KeyboardS) then moveDir.z = moveDir.z - 1 end
    if Input:keyboard():isDown(Button.KeyboardA) then moveDir.x = moveDir.x - 1 end
    if Input:keyboard():isDown(Button.KeyboardD) then moveDir.x = moveDir.x + 1 end
    if Input:keyboard():isDown(Button.KeyboardSpace) then moveDir.y = moveDir.y + 1 end
    if Input:keyboard():isDown(Button.KeyboardControlLeft) then moveDir.y = moveDir.y - 1 end

    if moveDir:length() > 0.001 then
        moveDir = moveDir:normalize()
    end

    -- Speed modifiers
    local speed = self.moveSpeed
    if Input:keyboard():isDown(Button.KeyboardShiftLeft) then speed = speed * self.fastMultiplier end
    if Input:keyboard():isDown(Button.KeyboardAltLeft) then speed = speed * self.slowMultiplier end

    -- Apply movement
    if moveDir:length() > 0.001 then
        self:move(moveDir, speed, dt)
    else
        self.velocity:ilerp(Vec3f(0, 0, 0), 1.0 - math.exp(-10.0 * dt))
        local pos = self:getPosition()
        pos = pos + Position(self.velocity.x * dt, self.velocity.y * dt, self.velocity.z * dt)
        self:setPosition(pos)
    end
end

---Rotate the camera using quaternion rotations around local axes
---@param yawDelta number Yaw rotation delta (around local up axis)
---@param pitchDelta number Pitch rotation delta (around local right axis)
---@param rollDelta number Roll rotation delta (around local forward axis)
function FreeCameraController:rotate(yawDelta, pitchDelta, rollDelta)
    local currentRot = self:getRotation()

    -- All rotations are in local space for proper free camera behavior
    local right = currentRot:getRight()
    local up = currentRot:getUp()
    local forward = currentRot:getForward()

    -- Yaw: rotate around camera's local up axis (not world up!)
    local yawRot = Quat.FromAxisAngle(up, yawDelta)

    -- Pitch: rotate around camera's local right axis
    local pitchRot = Quat.FromAxisAngle(right, pitchDelta)

    -- Roll: rotate around camera's local forward axis
    local rollRot = Quat.FromAxisAngle(forward, rollDelta)

    -- Apply all rotations in local space
    local newRot = currentRot
    newRot = yawRot:mul(newRot)
    newRot = pitchRot:mul(newRot)
    newRot = rollRot:mul(newRot)

    self:setRotation(newRot)

    -- Update stored angles for reference
    local euler = newRot:toEuler()
    self.yaw = euler.y
    self.pitch = euler.x
    self.roll = euler.z
end

---Move the camera in local space
---@param direction Vec3f Movement direction (normalized, in local space)
---@param speed number Movement speed
---@param dt number Delta time
function FreeCameraController:move(direction, speed, dt)
    local pos      = self:getPosition()
    local rot      = self:getRotation()

    -- Get LOCAL camera axes (already correct after roll!)
    local forward  = rot:getForward()
    local right    = rot:getRight()
    local up       = rot:getUp()

    -- Full local-space movement (6DOF)
    local worldDir = Vec3f(0, 0, 0)
    worldDir       = worldDir + forward:muls(direction.z) -- W/S
    worldDir       = worldDir + right:muls(direction.x)   -- A/D
    worldDir       = worldDir + up:muls(direction.y)      -- Space/Ctrl

    -- Optional: normalize if diagonal (feels better)
    if worldDir:length() > 0.001 then
        worldDir = worldDir:normalize()
    end

    local targetVelocity = worldDir:muls(speed)
    self.velocity:ilerp(targetVelocity, 1.0 - math.exp(-10.0 * dt * (1.0 / self.smoothing)))

    pos = pos + Position(self.velocity.x * dt, self.velocity.y * dt, self.velocity.z * dt)
    self:setPosition(pos)
end

---Set camera angles directly
---@param yaw number Yaw angle in radians
---@param pitch number Pitch angle in radians
---@param roll? number Roll angle in radians (optional)
function FreeCameraController:setAngles(yaw, pitch, roll)
    self.yaw = yaw
    self.pitch = pitch
    self.roll = roll or 0.0

    local rot = Quat.FromEuler(self.pitch, self.yaw, self.roll)
    self:setRotation(rot)
end

---Get current camera angles
---@return number yaw, number pitch, number roll
function FreeCameraController:getAngles()
    return self.yaw, self.pitch, self.roll
end

---Reset roll to zero
function FreeCameraController:resetRoll()
    self.roll = 0.0
    local rot = Quat.FromEuler(self.pitch, self.yaw, self.roll)
    self:setRotation(rot)
end

---Enable or disable roll
---@param enabled boolean Whether to allow roll
function FreeCameraController:setRollEnabled(enabled)
    self.allowRoll = enabled
end

function FreeCameraController:onPreRender(dt)
    if not self.enabled then return end
    -- Update camera position
end

return FreeCameraController
