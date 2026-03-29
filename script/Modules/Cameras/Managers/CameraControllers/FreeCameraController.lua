local CameraController = require("Modules.Cameras.Managers").CameraController
local CameraActions    = require("Input.ActionBindings.CameraActions")
local CursorManager    = require("Input.CursorManager")

---@class FreeCameraController : CameraController
---@overload fun(entity: Entity, config?: table): FreeCameraController
local FreeCameraController = Subclass("FreeCameraController", CameraController, function(self, entity, config)
    self:initController(entity)

    config = config or {}

    self.moveSpeed = config.moveSpeed or 20.0
    self.fastMultiplier = config.fastMultiplier or 5.0
    self.slowMultiplier = config.slowMultiplier or 0.1
    self.mouseSensitivity = config.mouseSensitivity or 1
    self.rollSpeed = config.rollSpeed or 2.0

    self.yaw = 0.0
    self.pitch = 0.0
    self.roll = 0.0
    self.allowRoll = config.allowRoll ~= false

    self.smoothing = config.smoothing or 0.1
    self.velocity = Vec3f(0, 0, 0)
    self.moveInput = Vec3f(0, 0, 0)

    self.mouseCaptured = false

    local rot = self:getRotation()
    local euler = rot:toEuler()
    self.yaw = euler.y
    self.pitch = euler.x
    self.roll = euler.z
end)

---@param dt number
function FreeCameraController:onInput(dt)
    if not self.enabled or not Window:isFocused() then return end

    -- Update action bindings
    CameraActions.MouseCapture:update(dt)
    CameraActions.Yaw:update(dt)
    CameraActions.Pitch:update(dt)
    CameraActions.Roll:update(dt)
    CameraActions.TranslateX:update(dt)
    CameraActions.TranslateY:update(dt)
    CameraActions.TranslateZ:update(dt)
    CameraActions.FastMove:update(dt)
    CameraActions.SlowMove:update(dt)

    local invertX = -1
    local invertY = -1

    -- Toggle mouse capture
    if CameraActions.MouseCapture:isPressed() then
        self.mouseCaptured = not self.mouseCaptured
    end

    if self.mouseCaptured then
        CursorManager:locked()

        local delta = Input:mouse():delta()
        if delta:length() > 0.001 then
            local yawDelta   = delta.x * invertX * self.mouseSensitivity * dt
            local pitchDelta = delta.y * invertY * self.mouseSensitivity * dt
            self:rotate(yawDelta, pitchDelta, 0)
        end
    else
        CursorManager:free()
    end

    -- Roll
    if self.allowRoll then
        local roll = CameraActions.Roll:get()
        if math.abs(roll) > 0.001 then
            self:rotate(0, 0, roll * self.rollSpeed * dt)
        end
    end

    -- Movement
    local moveX = CameraActions.TranslateX:get()
    local moveY = CameraActions.TranslateY:get()
    local moveZ = CameraActions.TranslateZ:get()
    local moveDir = Vec3f(moveX, moveY, moveZ)

    if moveDir:length() > 0.001 then
        moveDir = moveDir:normalize()
    end

    local speed = self.moveSpeed
    if CameraActions.FastMove:isDown() then speed = speed * self.fastMultiplier end
    if CameraActions.SlowMove:isDown() then speed = speed * self.slowMultiplier end

    if moveDir:length() > 0.001 then
        self:move(moveDir, speed, dt)
    else
        self.velocity:ilerp(Vec3f(0, 0, 0), 1.0 - math.exp(-10.0 * dt))
        local pos = self:getPosition()
        pos = pos + Position(self.velocity.x * dt, self.velocity.y * dt, self.velocity.z * dt)
        self:setPosition(pos)
    end
end

function FreeCameraController:rotate(yawDelta, pitchDelta, rollDelta)
    local currentRot = self:getRotation()
    local right = currentRot:getRight()
    local up = currentRot:getUp()
    local forward = currentRot:getForward()

    local yawRot = Quat.FromAxisAngle(up, yawDelta)
    local pitchRot = Quat.FromAxisAngle(right, pitchDelta)
    local rollRot = Quat.FromAxisAngle(forward, rollDelta)

    local newRot = currentRot
    newRot = yawRot:mul(newRot)
    newRot = pitchRot:mul(newRot)
    newRot = rollRot:mul(newRot)

    self:setRotation(newRot)

    local euler = newRot:toEuler()
    self.yaw = euler.y
    self.pitch = euler.x
    self.roll = euler.z
end

function FreeCameraController:move(direction, speed, dt)
    local pos      = self:getPosition()
    local rot      = self:getRotation()
    local forward  = rot:getForward()
    local right    = rot:getRight()
    local up       = rot:getUp()

    local worldDir = Vec3f(0, 0, 0)
    worldDir       = worldDir + forward:muls(direction.z)
    worldDir       = worldDir + right:muls(direction.x)
    worldDir       = worldDir + up:muls(direction.y)

    if worldDir:length() > 0.001 then
        worldDir = worldDir:normalize()
    end

    local targetVelocity = worldDir:muls(speed)
    self.velocity:ilerp(targetVelocity, 1.0 - math.exp(-10.0 * dt * (1.0 / self.smoothing)))

    pos = pos + Position(self.velocity.x * dt, self.velocity.y * dt, self.velocity.z * dt)
    self:setPosition(pos)
end

function FreeCameraController:setAngles(yaw, pitch, roll)
    self.yaw = yaw
    self.pitch = pitch
    self.roll = roll or 0.0
    self:setRotation(Quat.FromEuler(self.pitch, self.yaw, self.roll))
end

function FreeCameraController:getAngles()
    return self.yaw, self.pitch, self.roll
end

function FreeCameraController:resetRoll()
    self.roll = 0.0
    self:setRotation(Quat.FromEuler(self.pitch, self.yaw, self.roll))
end

function FreeCameraController:setRollEnabled(enabled)
    self.allowRoll = enabled
end

function FreeCameraController:onPreRender(dt)
end

return FreeCameraController
