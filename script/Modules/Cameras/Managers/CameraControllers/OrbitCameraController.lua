local CameraController  = require("Modules.Cameras.Managers").CameraController
local PhysicsComponents = require("Modules.Physics.Components")
local CameraActions     = require("Input.ActionBindings.CameraActions")

---@class OrbitCameraController : CameraController
---@overload fun(entity: Entity, config?: table): OrbitCameraController
local OrbitCameraController = Subclass("OrbitCameraController", CameraController, function(self, entity, config)
    self:initController(entity)

    config = config or {}

    self.target = nil
    self.targetOffset = config.targetOffset or Vec3f(0, 0, 0)

    self.distance = config.distance or 20.0
    self.minDistance = config.minDistance or 2.0
    self.maxDistance = config.maxDistance or 100.0
    self.zoomSpeed = config.zoomSpeed or 5.0

    self.mouseSensitivity = config.mouseSensitivity or 0.003
    self.yaw = config.initialYaw or 0.0
    self.pitch = config.initialPitch or 0.3
    self.minPitch = config.minPitch or -math.pi / 2 + 0.01
    self.maxPitch = config.maxPitch or math.pi / 2 - 0.01

    self.smoothing = config.smoothing or 0.1
    self.currentDistance = self.distance
end)

function OrbitCameraController:setTarget(target)
    self.target = target
    self:updateCameraPosition(0)
end

function OrbitCameraController:setTargetOffset(offset)
    self.targetOffset = offset
end

function OrbitCameraController:setDistance(distance)
    self.distance = Math.Clamp(distance, self.minDistance, self.maxDistance)
end

---@param dt number
function OrbitCameraController:onInput(dt)
    if not self.enabled or not Window:isFocused() then return end

    -- Update bindings
    CameraActions.Yaw:update(dt)
    CameraActions.Pitch:update(dt)
    CameraActions.Zoom:update(dt)

    -- Hold right mouse to rotate (CameraActions.Yaw/Pitch are Combo with MouseRight)
    local yawInput = CameraActions.Yaw:get()
    local pitchInput = CameraActions.Pitch:get()
    if math.abs(yawInput) > 0.001 or math.abs(pitchInput) > 0.001 then
        self.yaw   = self.yaw - yawInput * self.mouseSensitivity
        self.pitch = self.pitch + pitchInput * self.mouseSensitivity
        self.pitch = Math.Clamp(self.pitch, self.minPitch, self.maxPitch)
    end

    -- Zoom
    local zoom = CameraActions.Zoom:get()
    if math.abs(zoom) > 0.001 then
        self.distance = Math.Clamp(self.distance - zoom * self.zoomSpeed, self.minDistance, self.maxDistance)
    end
end

---@param dt number
function OrbitCameraController:onPreRender(dt)
    if not self.enabled then return end
    self:updateCameraPosition(dt)
end

function OrbitCameraController:updateCameraPosition(dt)
    if not self.target then return end

    local zoomLerp = math.min(1.0, 4.0 * dt)
    self.currentDistance = self.currentDistance + (self.distance - self.currentDistance) * zoomLerp

    local targetPos = Position(0, 0, 0)
    local rbCmp = self.target:get(PhysicsComponents.RigidBody)
    if rbCmp and rbCmp:getRigidBody() then
        targetPos = rbCmp:getRigidBody():getPos()
    end

    local cx = targetPos.x + self.targetOffset.x
    local cy = targetPos.y + self.targetOffset.y
    local cz = targetPos.z + self.targetOffset.z

    local cosPitch = math.cos(self.pitch)
    local camPos = Position(
        cx + cosPitch * math.sin(self.yaw) * self.currentDistance,
        cy + math.sin(self.pitch) * self.currentDistance,
        cz + cosPitch * math.cos(self.yaw) * self.currentDistance
    )

    self.transform:setPos(camPos)

    local lookDir = Vec3f(cx - camPos.x, cy - camPos.y, cz - camPos.z):normalize()
    local rot = Quat.FromLook(lookDir, Vec3f(0, 1, 0))
    self.transform:setRot(rot)
end

function OrbitCameraController:getAngles()
    return self.yaw, self.pitch
end

function OrbitCameraController:setAngles(yaw, pitch)
    self.yaw = yaw
    self.pitch = Math.Clamp(pitch, self.minPitch, self.maxPitch)
end

return OrbitCameraController
