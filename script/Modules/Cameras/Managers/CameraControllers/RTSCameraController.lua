local CameraController = require("Modules.Cameras.Managers").CameraController

---@class RTSCameraController : CameraController
---@field focusPoint Vec3f The current camera focus point in world coordinates
---@field focusPlaneY number The Y position of the plane the camera is focusing on
---@field targetPlaneY number The target Y plane for smooth shift-scroll movement
---@field pitch number The fixed pitch angle of the camera in radians
---@field yaw number The yaw angle of the camera in radians
---@field height number The target camera height from the focus plane
---@field currentHeight number The smoothed current camera height
---@field minHeight number Minimum allowed camera height
---@field maxHeight number Maximum allowed camera height
---@field heightSpeed number Scroll zoom speed
---@field planeChangeSpeed number Shift + scroll plane speed
---@field minPlaneY number Minimum Y of the plane
---@field maxPlaneY number Maximum Y of the plane
---@field keepCameraWorldYOnPlaneChange boolean Unused in smooth implementation
---@field rotationSpeed number Keyboard rotation speed (Q/E)
---@field mouseSensitivity number Mouse rotation sensitivity
---@field moveSpeed number Camera pan speed
---@field zoomSpeedScale number Zoom-dependent movement scaling
---@field cameraRelativeMovement boolean Whether WASD movement is relative to camera
---@field minX number Minimum world X
---@field maxX number Maximum world X
---@field minZ number Minimum world Z
---@field maxZ number Maximum world Z
---@field smoothing number Smoothing time constant in seconds
---@field mouseCaptured boolean Is the right mouse button currently held
---@field _wasMouseCaptured boolean Internal previous mouse capture state
---@field lastMousePos Vec2f|nil Last mouse position
---@field _cursorVisible boolean Current cursor visibility
local RTSCameraController = Subclass("RTSCameraController", CameraController, function(self, entity, cfg)
    self:initController(entity)

    cfg = cfg or {}

    -- Focus / geometry
    self.focusPoint = cfg.initialFocus or Vec3f(0, 0, 0)
    self.focusPlaneY = self.focusPoint.y
    self.targetPlaneY = self.focusPlaneY
    self.pitch = cfg.pitch or math.rad(60)
    self.yaw = cfg.initialYaw or 0.0

    -- Height
    self.height = cfg.height or 500
    self.currentHeight = self.height
    self.minHeight = cfg.minHeight or 10.0
    self.maxHeight = cfg.maxHeight or 100000.0
    self.heightSpeed = cfg.heightSpeed or 15.0

    -- Plane change
    self.planeChangeSpeed = cfg.planeChangeSpeed or 15.0
    self.minPlaneY = cfg.minPlaneY or -100000.0
    self.maxPlaneY = cfg.maxPlaneY or 100000.0
    self.keepCameraWorldYOnPlaneChange = cfg.keepCameraWorldYOnPlaneChange == nil and true or cfg.keepCameraWorldYOnPlaneChange

    -- Rotation
    self.rotationSpeed = cfg.rotationSpeed or 2.0
    self.mouseSensitivity = cfg.mouseSensitivity or 0.003

    -- Movement
    self.moveSpeed = cfg.moveSpeed or 30.0
    self.zoomSpeedScale = cfg.zoomSpeedScale or 0.003
    self.cameraRelativeMovement = cfg.cameraRelativeMovement == nil and true or cfg.cameraRelativeMovement

    -- Map bounds
    self.minX = cfg.minX or -1e9
    self.maxX = cfg.maxX or 1e9
    self.minZ = cfg.minZ or -1e9
    self.maxZ = cfg.maxZ or 1e9

    -- Smoothing
    self.smoothing = cfg.smoothing or 0.12

    -- Mouse capture
    self.mouseCaptured = false
    self._wasMouseCaptured = false
    self.lastMousePos = nil

    -- Cursor visibility
    self._cursorVisible = true
    Input:setCursorVisible(true)
end)

---@param point Vec3f
function RTSCameraController:setFocusPoint(point)
    self.focusPoint = point
    self.focusPlaneY = point.y
    self.targetPlaneY = self.focusPlaneY
    self:updateCameraPosition(0)
end

---@param h number
function RTSCameraController:setHeight(h)
    self.height = Math.Clamp(h, self.minHeight, self.maxHeight)
end

---@param p number
function RTSCameraController:setPitch(p)
    self.pitch = Math.Clamp(p, math.rad(1), math.rad(89))
end

---@param y number
function RTSCameraController:setYaw(y)
    self.yaw = y
end

---@return number, number
function RTSCameraController:getAngles()
    return self.yaw, self.pitch
end

---@return Vec3f
function RTSCameraController:getFocusPoint()
    return self.focusPoint
end

---@param dt number Delta time in seconds
function RTSCameraController:onInput(dt)
    if not self.enabled or not Window:isFocused() then
        if self._wasMouseCaptured then
            self._wasMouseCaptured = false
            Input:setCursorVisible(true)
            self._cursorVisible = true
            self.lastMousePos = nil
        end
        return
    end

    -- Mouse capture
    if Input:mouse():isPressed(MouseControl.Right) then self.mouseCaptured = true end
    if Input:mouse():isReleased(MouseControl.Right) then self.mouseCaptured = false end

    if self.mouseCaptured ~= self._wasMouseCaptured then
        if self.mouseCaptured then
            Input:setCursorVisible(false)
            self._cursorVisible = false
            self.lastMousePos = Input:mouse():position()
        else
            Input:setCursorVisible(true)
            self._cursorVisible = true
            self.lastMousePos = nil
        end
        self._wasMouseCaptured = self.mouseCaptured
    end

    -- Mouse rotation
    if self.mouseCaptured then
        local cur = Input:mouse():position()
        local delta = Vec2f(0, 0)
        if self.lastMousePos then delta = cur - self.lastMousePos end
        if delta:length() > 0.0001 then self.yaw = self.yaw - delta.x * self.mouseSensitivity end
        self.lastMousePos = cur
    end

    -- Keyboard rotation
    if Input:keyboard():isDown(KeyboardButton.Q) then self.yaw = self.yaw - self.rotationSpeed * dt end
    if Input:keyboard():isDown(KeyboardButton.E) then self.yaw = self.yaw + self.rotationSpeed * dt end

    -- WASD movement
    local inputX, inputZ = 0.0, 0.0
    if Input:keyboard():isDown(KeyboardButton.W) then inputZ = inputZ + 1.0 end
    if Input:keyboard():isDown(KeyboardButton.S) then inputZ = inputZ - 1.0 end
    if Input:keyboard():isDown(KeyboardButton.A) then inputX = inputX - 1.0 end
    if Input:keyboard():isDown(KeyboardButton.D) then inputX = inputX + 1.0 end

    if math.abs(inputX) > 1e-4 or math.abs(inputZ) > 1e-4 then
        local invlen = 1.0 / math.sqrt(inputX * inputX + inputZ * inputZ)
        inputX = inputX * invlen
        inputZ = inputZ * invlen

        local moveSpeed = self.moveSpeed
        local zoomScale = 1.0 + (self.currentHeight * (self.zoomSpeedScale or 0.0))
        moveSpeed = moveSpeed * Math.Clamp(zoomScale, 0.25, 8.0)

        local forwardX = -math.sin(self.yaw)
        local forwardZ = -math.cos(self.yaw)
        local rightX = math.cos(self.yaw)
        local rightZ = -math.sin(self.yaw)

        local worldDX = rightX * inputX + forwardX * inputZ
        local worldDZ = rightZ * inputX + forwardZ * inputZ

        self.focusPoint = Vec3f(
            Math.Clamp(self.focusPoint.x + worldDX * moveSpeed * dt, self.minX, self.maxX),
            self.focusPoint.y,
            Math.Clamp(self.focusPoint.z + worldDZ * moveSpeed * dt, self.minZ, self.maxZ)
        )
    end

    -- Mouse wheel
    local scroll = Input:mouse():value(MouseControl.ScrollY)
    if math.abs(scroll) > 1e-4 then
        local inverted = -scroll
        local shiftDown = Input:keyboard():isDown(KeyboardButton.ShiftLeft) or Input:keyboard():isDown(KeyboardButton.ShiftRight)

        if shiftDown then
            -- Shift + scroll: adjust target plane Y only (no zoom)
            self.targetPlaneY = Math.Clamp(self.targetPlaneY + inverted * self.planeChangeSpeed, self.minPlaneY, self.maxPlaneY)
        else
            -- Normal zoom
            self.height = Math.Clamp(self.height + inverted * self.heightSpeed, self.minHeight, self.maxHeight)
        end
    end
end

-- Called before render
---@param dt number Delta time in seconds
function RTSCameraController:onPreRender(dt)
    if not self.enabled then return end
    self:updateCameraPosition(dt)
end

-- Update camera position
---@param dt number Delta time in seconds
function RTSCameraController:updateCameraPosition(dt)
    local tau = math.max(1e-4, self.smoothing)
    local alpha = 1.0 - math.exp(-dt / tau)

    -- Smooth zoom
    self.currentHeight = self.currentHeight + (self.height - self.currentHeight) * alpha

    -- Smooth plane movement
    if not self.targetPlaneY then self.targetPlaneY = self.focusPlaneY end
    local alphaPlane = 1.0 - math.exp(-dt / tau)
    self.focusPlaneY = self.focusPlaneY + (self.targetPlaneY - self.focusPlaneY) * alphaPlane

    local pitch = Math.Clamp(self.pitch, math.rad(1), math.rad(89))
    local sPitch = math.sin(pitch)
    local radius = (sPitch > 1e-5) and (self.currentHeight / sPitch) or (self.currentHeight / 1e-5)
    local cosPitch = math.cos(pitch)
    local sinYaw = math.sin(self.yaw)
    local cosYaw = math.cos(self.yaw)

    local dirX = cosPitch * sinYaw
    local dirZ = cosPitch * cosYaw

    -- Camera position: X/Z orbit + Y = focusPlaneY + height
    local camX = self.focusPoint.x + dirX * radius
    local camZ = self.focusPoint.z + dirZ * radius
    local camY = self.focusPlaneY + self.currentHeight

    self:setPosition(Position(camX, camY, camZ))

    local lookDir = Vec3f(
        self.focusPoint.x - camX,
        self.focusPlaneY - camY,
        self.focusPoint.z - camZ
    ):normalize()

    local worldUp = Vec3f(0, 1, 0)
    local rot = Quat.FromLook(lookDir, worldUp)
    self:setRotation(rot)
end

return RTSCameraController
