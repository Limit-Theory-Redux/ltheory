local CameraController = require("Modules.Cameras.Managers").CameraController

---@class CinematicCameraController : CameraController
---@field targetPosition Vec3f The target camera position in world coordinates
---@field currentPosition Vec3f The current smoothed camera position
---@field targetRotation Quat The target camera rotation
---@field currentRotation Quat The current smoothed camera rotation
---@field targetFocusPoint Vec3f|nil Optional target focus point (camera will look at this)
---@field currentFocusPoint Vec3f|nil Current smoothed focus point
---@field positionSmoothing number Position smoothing time constant in seconds
---@field rotationSmoothing number Rotation smoothing time constant in seconds
---@field focusSmoothing number Focus point smoothing time constant in seconds
---@field useFocusPoint boolean Whether to automatically orient camera toward focus point
local CinematicCameraController = Subclass("CinematicCameraController", CameraController, function(self, entity, cfg)
    self:initController(entity)

    cfg = cfg or {}

    -- Position
    local initialPos = cfg.initialPosition or Vec3f(0, 100, 0)
    self.targetPosition = initialPos
    self.currentPosition = initialPos

    -- Rotation
    local initialRot = cfg.initialRotation or Quat.Identity()
    self.targetRotation = initialRot
    self.currentRotation = initialRot

    -- Focus point (optional)
    self.targetFocusPoint = cfg.initialFocusPoint
    self.currentFocusPoint = cfg.initialFocusPoint
    self.useFocusPoint = cfg.useFocusPoint or false

    -- Smoothing
    self.positionSmoothing = cfg.positionSmoothing or 0.2
    self.rotationSmoothing = cfg.rotationSmoothing or 0.2
    self.focusSmoothing = cfg.focusSmoothing or 0.2
end)

---Set the target position for the camera
---@param position Vec3f
---@param instant boolean|nil If true, move instantly without smoothing
function CinematicCameraController:setTargetPosition(position, instant)
    self.targetPosition = position
    if instant then
        self.currentPosition = position
    end
end

---Set the target rotation for the camera
---@param rotation Quat
---@param instant boolean|nil If true, rotate instantly without smoothing
function CinematicCameraController:setTargetRotation(rotation, instant)
    self.targetRotation = rotation
    self.useFocusPoint = false -- Disable focus point when manually setting rotation
    if instant then
        self.currentRotation = rotation
    end
end

---Set the target focus point (camera will look at this point)
---@param focusPoint Vec3f
---@param instant boolean|nil If true, update instantly without smoothing
function CinematicCameraController:setTargetFocusPoint(focusPoint, instant)
    self.targetFocusPoint = focusPoint
    self.useFocusPoint = true
    if instant then
        self.currentFocusPoint = focusPoint
    end
end

---Disable focus point mode and use manual rotation instead
function CinematicCameraController:disableFocusPoint()
    self.useFocusPoint = false
    self.targetFocusPoint = nil
    self.currentFocusPoint = nil
end

---Set both position and focus point at once
---@param position Vec3f
---@param focusPoint Vec3f
---@param instant boolean|nil If true, update instantly without smoothing
function CinematicCameraController:setPositionAndFocus(position, focusPoint, instant)
    self:setTargetPosition(position, instant)
    self:setTargetFocusPoint(focusPoint, instant)
end

---Get the current camera position
---@return Vec3f
function CinematicCameraController:getCurrentPosition()
    return self.currentPosition
end

---Get the current camera rotation
---@return Quat
function CinematicCameraController:getCurrentRotation()
    return self.currentRotation
end

---Get the current focus point (if any)
---@return Vec3f|nil
function CinematicCameraController:getCurrentFocusPoint()
    return self.currentFocusPoint
end

---Set smoothing values
---@param positionSmoothing number|nil
---@param rotationSmoothing number|nil
---@param focusSmoothing number|nil
function CinematicCameraController:setSmoothing(positionSmoothing, rotationSmoothing, focusSmoothing)
    if positionSmoothing then self.positionSmoothing = positionSmoothing end
    if rotationSmoothing then self.rotationSmoothing = rotationSmoothing end
    if focusSmoothing then self.focusSmoothing = focusSmoothing end
end

---Called before render to update camera position
---@param dt number Delta time in seconds
function CinematicCameraController:onPreRender(dt)
    if not self.enabled then return end
    self:updateCameraTransform(dt)
end

---Update camera transform with smoothing
---@param dt number Delta time in seconds
function CinematicCameraController:updateCameraTransform(dt)
    -- Smooth position
    local posTau = math.max(1e-4, self.positionSmoothing)
    local posAlpha = 1.0 - math.exp(-dt / posTau)

    self.currentPosition = Vec3f(
        self.currentPosition.x + (self.targetPosition.x - self.currentPosition.x) * posAlpha,
        self.currentPosition.y + (self.targetPosition.y - self.currentPosition.y) * posAlpha,
        self.currentPosition.z + (self.targetPosition.z - self.currentPosition.z) * posAlpha
    )

    self:setPosition(Position(self.currentPosition.x, self.currentPosition.y, self.currentPosition.z))

    -- Handle rotation (either focus point or manual rotation)
    if self.useFocusPoint and self.targetFocusPoint then
        -- Smooth focus point
        if not self.currentFocusPoint then
            self.currentFocusPoint = self.targetFocusPoint
        else
            local focusTau = math.max(1e-4, self.focusSmoothing)
            local focusAlpha = 1.0 - math.exp(-dt / focusTau)

            self.currentFocusPoint = Vec3f(
                self.currentFocusPoint.x + (self.targetFocusPoint.x - self.currentFocusPoint.x) * focusAlpha,
                self.currentFocusPoint.y + (self.targetFocusPoint.y - self.currentFocusPoint.y) * focusAlpha,
                self.currentFocusPoint.z + (self.targetFocusPoint.z - self.currentFocusPoint.z) * focusAlpha
            )
        end

        -- Calculate look direction
        local lookDir = Vec3f(
            self.currentFocusPoint.x - self.currentPosition.x,
            self.currentFocusPoint.y - self.currentPosition.y,
            self.currentFocusPoint.z - self.currentPosition.z
        ):normalize()

        local worldUp = Vec3f(0, 1, 0)
        self.currentRotation = Quat.FromLook(lookDir, worldUp)
        self:setRotation(self.currentRotation)
    else
        -- Smooth rotation using quaternion slerp
        local rotTau = math.max(1e-4, self.rotationSmoothing)
        local rotAlpha = 1.0 - math.exp(-dt / rotTau)

        self.currentRotation = self.currentRotation:slerp(self.targetRotation, rotAlpha)
        self:setRotation(self.currentRotation)
    end
end

---Empty input handler (no input processing for cinematic camera)
---@param dt number Delta time in seconds
function CinematicCameraController:onInput(dt)
    -- Cinematic camera ignores all input
end

return CinematicCameraController
