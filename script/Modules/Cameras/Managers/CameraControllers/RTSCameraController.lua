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
---@field rotationSpeed number Keyboard rotation speed (Q/E)
---@field mouseSensitivity number Mouse rotation sensitivity
---@field moveSpeed number Base camera pan speed
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
    self.maxHeight = cfg.maxHeight or 50000.0
    self.heightSpeed = cfg.heightSpeed or 15.0

    -- Plane change
    self.planeChangeSpeed = cfg.planeChangeSpeed or 15.0
    self.minPlaneY = cfg.minPlaneY or -50000.0
    self.maxPlaneY = cfg.maxPlaneY or 50000.0

    -- Rotation
    self.rotationSpeed = cfg.rotationSpeed or 2.0
    self.mouseSensitivity = cfg.mouseSensitivity or 1

    -- Movement
    self.moveSpeed = cfg.moveSpeed or 50.0
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

    -- Store defaults for reset
    self.defaultYaw = self.yaw
    self.defaultPitch = self.pitch
    self.basePitch = self.pitch -- Store base pitch for clamping
    self.defaultHeight = self.height
    self.defaultFocusPoint = self.focusPoint
    self.defaultPlaneY = self.targetPlaneY or 0.0

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

function RTSCameraController:resetCamera()
    -- Reset to default/initial camera position
    self.yaw = self.defaultYaw or 0.0
    self.pitch = self.defaultPitch or self.basePitch or self.pitch
    self.height = self.defaultHeight or (self.minHeight + self.maxHeight) * 0.5
    self.focusPoint = self.defaultFocusPoint or Vec3f(0, 0, 0)
    self.targetPlaneY = self.defaultPlaneY or 0.0

    -- Reset base pitch reference
    self.basePitch = self.pitch
end

---@param dt number
function RTSCameraController:onInput(dt)
    if not self.enabled or not Window:isFocused() then
        if self._wasMouseCaptured then
            self._wasMouseCaptured = false
            Input:setCursorVisible(true)
            self._cursorVisible = true
        end
        return
    end

    -- Backspace to reset camera
    if Input:keyboard():isPressed(KeyboardButton.Backspace) then
        self:resetCamera()
    end

    -- Right-click hold for rotation
    self.mouseCaptured = Input:mouse():isDown(MouseControl.Right)

    if self.mouseCaptured then
        GameState.render.gameWindow:cursor():setGrabMode(CursorGrabMode.Locked)
        Input:setCursorVisible(false)

        local delta = Input:mouse():delta()
        if delta:length() > 0.001 then
            -- Apply mouse sensitivity for yaw (horizontal rotation)
            self.yaw = self.yaw - delta.x * self.mouseSensitivity * dt

            -- Apply pitch adjustment (vertical camera angle)
            local pitchSensitivity = self.mouseSensitivity * 0.5
            local pitchDelta = delta.y * pitchSensitivity * dt

            -- Adjust pitch with clamping relative to base pitch
            local basePitch = self.basePitch or self.pitch -- Store base if not set
            if not self.basePitch then
                self.basePitch = self.pitch
            end

            self.pitch = self.pitch + pitchDelta

            -- Clamp to ±30 degrees from base angle
            local maxPitchOffset = math.rad(30) -- 30 degrees in radians
            self.pitch = Math.Clamp(self.pitch, self.basePitch - maxPitchOffset, self.basePitch + maxPitchOffset)
        end
    else
        GameState.render.gameWindow:cursor():setGrabMode(CursorGrabMode.None)
        Input:setCursorVisible(true)
    end

    -- Keyboard rotation alternative
    if Input:keyboard():isDown(KeyboardButton.Q) then
        self.yaw = self.yaw - self.rotationSpeed * dt
    end
    if Input:keyboard():isDown(KeyboardButton.E) then
        self.yaw = self.yaw + self.rotationSpeed * dt
    end

    -- Calculate exponential multiplier for solar system scale
    local normalizedHeight = (self.height - self.minHeight) / (self.maxHeight - self.minHeight)

    -- Much more aggressive exponential for solar system scales
    local exponent = 4.0        -- Steeper growth for massive scale differences
    local maxMultiplier = 100.0 -- Much higher for solar system distances

    -- Smooth exponential curve
    local expMultiplier = 1.0 + (maxMultiplier - 1.0) * (1.0 - math.exp(-exponent * normalizedHeight))

    -- WASD movement (camera-relative) with exponential scaling
    local inputX, inputZ = 0.0, 0.0
    if Input:keyboard():isDown(KeyboardButton.W) then inputZ = inputZ + 1.0 end
    if Input:keyboard():isDown(KeyboardButton.S) then inputZ = inputZ - 1.0 end
    if Input:keyboard():isDown(KeyboardButton.A) then inputX = inputX - 1.0 end
    if Input:keyboard():isDown(KeyboardButton.D) then inputX = inputX + 1.0 end

    -- Edge panning (mouse near screen edges)
    if not self.mouseCaptured then
        local mousePos = Input:mouse():position()
        local screenW = Window:width()
        local screenH = Window:height()
        local edgeThreshold = 20 -- pixels from edge to trigger panning
        local edgePanSpeed = 1.0 -- multiplier for edge panning

        if mousePos.x < edgeThreshold then
            inputX = inputX - 1.0 * edgePanSpeed
        elseif mousePos.x > screenW - edgeThreshold then
            inputX = inputX + 1.0 * edgePanSpeed
        end

        if mousePos.y < edgeThreshold then
            inputZ = inputZ + 1.0 * edgePanSpeed
        elseif mousePos.y > screenH - edgeThreshold then
            inputZ = inputZ - 1.0 * edgePanSpeed
        end
    end

    -- Middle mouse button drag panning
    local middleMousePanning = Input:mouse():isDown(MouseControl.Middle)
    if middleMousePanning then
        local delta = Input:mouse():delta()
        if delta:length() > 0.001 then
            -- Convert screen space delta to world space panning
            local panSensitivity = 0.5
            inputX = inputX - delta.x * panSensitivity
            inputZ = inputZ + delta.y * panSensitivity
        end
    end

    if math.abs(inputX) > 1e-4 or math.abs(inputZ) > 1e-4 then
        -- Normalize if from keyboard, but allow higher values from mouse drag
        if math.abs(inputX) <= 1.0 and math.abs(inputZ) <= 1.0 then
            local invlen = 1.0 / math.sqrt(inputX * inputX + inputZ * inputZ)
            inputX = inputX * invlen
            inputZ = inputZ * invlen
        end

        local moveSpeed = self.moveSpeed * expMultiplier

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

    -- Mouse wheel zoom / plane adjustment with exponential scaling
    local scroll = Input:mouse():value(MouseControl.ScrollY)
    if math.abs(scroll) > 1e-4 then
        local inverted = -scroll
        local shiftDown = Input:keyboard():isDown(KeyboardButton.ShiftLeft) or Input:keyboard():isDown(KeyboardButton.ShiftRight)
        local ctrlDown = Input:keyboard():isDown(KeyboardButton.ControlLeft) or Input:keyboard():isDown(KeyboardButton.ControlRight)

        -- Ctrl+Scroll for faster zoom (useful for solar system scale)
        local scrollMultiplier = ctrlDown and 3.0 or 1.0

        if shiftDown then
            -- Blend between height-proportional (close) and exponential (far) for plane change too
            local heightProportionalPlaneSpeed = self.height * 0.1 -- 10% of current height per scroll
            local exponentialPlaneSpeed = self.planeChangeSpeed * expMultiplier

            -- Use more height-proportional speed when close, more exponential when far
            local blendFactor = math.min(normalizedHeight * 2.0, 1.0)
            local planeSpeed = heightProportionalPlaneSpeed * (1.0 - blendFactor) + exponentialPlaneSpeed * blendFactor

            self.targetPlaneY = Math.Clamp(self.targetPlaneY + inverted * planeSpeed, self.minPlaneY, self.maxPlaneY)
        else
            -- Blend between height-proportional (close) and exponential (far)
            local heightProportionalSpeed = self.height * 0.1 -- 10% of current height per scroll
            local exponentialSpeed = self.heightSpeed * expMultiplier

            -- Use more height-proportional speed when close, more exponential when far
            local blendFactor = math.min(normalizedHeight * 2.0, 1.0) -- 0 to 1 over first 50% of range
            local heightSpeed = heightProportionalSpeed * (1.0 - blendFactor) + exponentialSpeed * blendFactor

            heightSpeed = heightSpeed * scrollMultiplier
            self.height = Math.Clamp(self.height + inverted * heightSpeed, self.minHeight, self.maxHeight)
        end
    end

    -- Keyboard shortcuts for zoom (Page Up/Down or +/-)
    local keyboardZoom = 0.0
    if Input:keyboard():isDown(KeyboardButton.PageUp) or Input:keyboard():isDown(KeyboardButton.Equal) then
        keyboardZoom = -1.0
    end
    if Input:keyboard():isDown(KeyboardButton.PageDown) or Input:keyboard():isDown(KeyboardButton.Minus) then
        keyboardZoom = 1.0
    end

    if math.abs(keyboardZoom) > 1e-4 then
        -- Same blended approach for keyboard zoom
        local heightProportionalSpeed = self.height * 0.1
        local exponentialSpeed = self.heightSpeed * expMultiplier
        local blendFactor = math.min(normalizedHeight * 2.0, 1.0)
        local heightSpeed = heightProportionalSpeed * (1.0 - blendFactor) + exponentialSpeed * blendFactor

        heightSpeed = heightSpeed * 2.0 -- Keyboard zoom a bit faster
        self.height = Math.Clamp(self.height + keyboardZoom * heightSpeed * dt, self.minHeight, self.maxHeight)
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
