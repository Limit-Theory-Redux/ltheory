local Application      = require('States.Application')

local InputTest        = Subclass("InputTest", Application)

local Cache            = require("Render.Cache")
local CameraManager    = require("Modules.Cameras.Managers.CameraManager")
local CameraEntity     = require("Modules.Cameras.Entities").Camera
local DeltaTimer       = require("Shared.Tools.DeltaTimer")
local DrawEx           = require("UI.DrawEx")
local GC               = require("Core.Util.GC")
local QuickProfiler    = require("Shared.Tools.QuickProfiler")
local RenderCoreSystem = require("Modules.Rendering.Systems.RenderCoreSystem")

-- ActionBinding system
local ShipActions           = require('Input.ActionBindings.ShipActions')
local GamepadCursorActions  = require('Input.ActionBindings.GamepadCursorActions')

local function drawLabelValue(label, value, x, y, font, fontSize, r, g, b, a, blendMode)
    local fontName = font or 'Unageo-Medium'
    local size = fontSize or 12
    local cr, cg, cb, ca = r or 0.9, g or 0.9, b or 0.9, a or 1.0
    local mode = blendMode or "additive"
    local drawFunc = mode == "alpha" and DrawEx.TextAlpha or DrawEx.TextAdditive
    local text = string.format("%s: %s", label, value)
    drawFunc(fontName, text, size, x, y, 400, size + 4, cr, cg, cb, ca, 0.0, 0.5)
end

local function drawSectionTitle(title, x, y, font, fontSize, r, g, b, a, blendMode)
    local fontName = font or 'Unageo-Medium'
    local size = fontSize or 14
    local cr, cg, cb, ca = r or 1.0, g or 0.8, b or 0.2, a or 1.0
    local mode = blendMode or "additive"
    local drawFunc = mode == "alpha" and DrawEx.TextAlpha or DrawEx.TextAdditive
    drawFunc(fontName, "[ " .. title .. " ]", size, x, y, 400, size + 4, cr, cg, cb, ca, 0.0, 0.5)
end

local function drawAxisBar(label, value, x, y, width, height, font, fontSize)
    local fontName = font or 'Unageo-Medium'
    local size = fontSize or 11

    DrawEx.TextAdditive(fontName, label, size, x, y, 100, height, 0.9, 0.9, 0.9, 1.0, 0.0, 0.5)

    local barX = x + 100
    local barWidth = width - 100
    local halfWidth = barWidth / 2
    local centerX = barX + halfWidth

    RenderState.PushBlendMode(BlendMode.Alpha)
    DrawEx.Rect(barX, y, barWidth, height, Color(0.2, 0.2, 0.2, 0.8))
    DrawEx.Rect(centerX - 1, y, 2, height, Color(0.5, 0.5, 0.5, 1.0))

    local fillWidth = math.abs(value) * halfWidth
    if value >= 0 then
        DrawEx.Rect(centerX, y + 2, fillWidth, height - 4, Color(0.2, 0.8, 0.2, 1.0))
    else
        DrawEx.Rect(centerX - fillWidth, y + 2, fillWidth, height - 4, Color(0.8, 0.2, 0.2, 1.0))
    end
    RenderState.PopBlendMode()

    local valueStr = string.format("%.2f", value)
    DrawEx.TextAdditive(fontName, valueStr, size - 1, barX + barWidth + 10, y, 60, height, 0.7, 0.7, 0.7, 1.0, 0.0, 0.5)
end

local function drawProgressBar(label, value, x, y, width, height, font, fontSize)
    local fontName = font or 'Unageo-Medium'
    local size = fontSize or 11

    DrawEx.TextAdditive(fontName, label, size, x, y, 100, height, 0.9, 0.9, 0.9, 1.0, 0.0, 0.5)

    local barX = x + 100
    local barWidth = width - 100

    RenderState.PushBlendMode(BlendMode.Alpha)
    DrawEx.Rect(barX, y, barWidth, height, Color(0.2, 0.2, 0.2, 0.8))
    
    local clampedValue = math.max(0, math.min(1, value))
    local fillWidth = clampedValue * barWidth
    if fillWidth > 0 then
        DrawEx.Rect(barX, y + 2, fillWidth, height - 4, Color(1.0, 1.0, 0.2, 1.0))
    end
    
    RenderState.PopBlendMode()

    local valueStr = string.format("%.2f", clampedValue)
    DrawEx.TextAdditive(fontName, valueStr, size - 1, barX + barWidth + 10, y, 60, height, 0.7, 0.7, 0.7, 1.0, 0.0, 0.5)
end

local function drawStickVisualizer(label, xValue, yValue, x, y, size, font, fontSize)
    local fontName = font or 'Unageo-Medium'
    local textSize = fontSize or 11
    local centerX = x + size / 2
    local centerY = y + size / 2
    local radius = size / 2

    DrawEx.TextAdditive(fontName, label, textSize, x, y - textSize - 4, size, textSize + 4, 0.9, 0.9, 0.9, 1.0, 0.5, 0.5)
    DrawEx.Circle(centerX, centerY, radius, Color(0.15, 0.15, 0.15, 0.9))
    local deadzoneRadius = radius * 0.1
    DrawEx.Circle(centerX, centerY, deadzoneRadius, Color(0.3, 0.3, 0.3, 0.8))
    DrawEx.Line(centerX - radius, centerY, centerX + radius, centerY, Color(0.4, 0.4, 0.4, 1.0))
    DrawEx.Line(centerX, centerY - radius, centerX, centerY + radius, Color(0.4, 0.4, 0.4, 1.0))

    local stickX = centerX + xValue * radius
    local stickY = centerY - yValue * radius
    local dotSize = 8
    DrawEx.Rect(stickX - dotSize / 2, stickY - dotSize / 2, dotSize, dotSize,
        Color(0.2 + math.abs(xValue) * 0.6, 0.8 - math.abs(yValue) * 0.4, 0.2, 1.0))
end

local function drawDebugLines(lineDefs, startX, startY, lineHeight, font, fontSize, r, g, b, a, alignX, alignY, blendMode)
    local x = startX or 40
    local y = startY or 40
    local spacing = lineHeight or 20
    local fontName = font or 'Unageo-Medium'
    local size = fontSize or 12
    local cr, cg, cb, ca = r or 0.0, g or 0.0, b or 0.0, a or 0.0
    local ax, ay = alignX or 0.0, alignY or 0.0
    local mode = blendMode or "additive"
    local drawFunc = mode == "alpha" and DrawEx.TextAlpha or DrawEx.TextAdditive

    for _, item in ipairs(lineDefs) do
        local line
        if type(item) == "table" and type(item.label) == "string" and type(item.getValue) == "function" then
            line = string.format("%s: %s", item.label, tostring(item.getValue()))
        elseif type(item) == "string" then
            line = item
        end
        if line then
            drawFunc(fontName, line, size, x, y, 100, spacing, cr, cg, cb, ca, ax, ay)
            y = y + spacing
        end
    end
end

local function buildButtonNameLookup()
    local lookup = {}
    for name, value in pairs(Button) do
        if type(value) == "number" then
            lookup[value] = name
        end
    end
    return lookup
end

local function computeTimeAndFrameCounts(count, accumulatedTime, deltaTime, smoothFPS, timeScale)
    count = count + 1
    accumulatedTime = accumulatedTime + deltaTime
    local fpsInterval = 0.1
    if accumulatedTime >= fpsInterval then
        local instantFPS = count / accumulatedTime * (timeScale or 1)
        smoothFPS = smoothFPS * 0.3 + instantFPS * 0.7
        count = 0
        accumulatedTime = 0
    end
    return count, accumulatedTime, smoothFPS
end

function InputTest:drawInputState()
    local x = 40
    local y = 120
    local lineHeight = 22
    local sectionGap = 30

    self.activeDevice = Input:activeDeviceType()
    local deviceText = "Active Device: " .. (self.deviceNames[self.activeDevice] or "UNKNOWN")
    local isGamepad = self.activeDevice == InputDeviceType.Gamepad
    local deviceColor = isGamepad and { 0.2, 0.8, 0.2 } or { 0.2, 0.6, 0.9 }
    DrawEx.TextAdditive('Unageo-Medium', deviceText, 14, x, y - 30, 400, 20,
        deviceColor[1], deviceColor[2], deviceColor[3], 1.0, 0.0, 0.5)

    drawSectionTitle("Mouse", x, y)
    y = y + lineHeight + 4

    local mousePos = Input:mouse():position()
    local mouseDelta = Input:mouse():delta()
    local mouseScroll = Input:mouse():scroll()
    local mouseInWindow = Input:mouse():inWindow()

    drawLabelValue("Position", string.format("%.0f, %.0f", mousePos.x, mousePos.y), x, y)
    y = y + lineHeight
    drawLabelValue("Delta", string.format("%.1f, %.1f", mouseDelta.x, mouseDelta.y), x, y)
    y = y + lineHeight
    drawLabelValue("Scroll", string.format("%.1f, %.1f", mouseScroll.x, mouseScroll.y), x, y)
    y = y + lineHeight
    drawLabelValue("In Window", mouseInWindow and "Yes" or "No", x, y)
    y = y + lineHeight
    drawLabelValue("Buttons Held", self.heldMouseStr, x, y)
    y = y + lineHeight

    local recentMouseX = x + 110
    DrawEx.TextAdditive('Unageo-Medium', "Recent:", 12, x, y, 100, lineHeight, 0.9, 0.9, 0.9, 1.0, 0.0, 0.5)
    if #self.lastPressedMouseButtons == 0 then
        DrawEx.TextAdditive('Unageo-Medium', "(press any)", 10, recentMouseX, y, 200, lineHeight, 0.5, 0.5, 0.5, 0.7, 0.0, 0.5)
    else
        for i, entry in ipairs(self.lastPressedMouseButtons) do
            local age = self.time - entry.time
            local alpha = math.max(0.2, 1.0 - (age / self.keyHistoryDuration))
            DrawEx.TextAdditive('Unageo-Medium', entry.name, 10, recentMouseX, y, 80, lineHeight, 0.9, 0.7, 0.7, alpha, 0.0, 0.5)
            recentMouseX = recentMouseX + 25 + #entry.name * 4
        end
    end
    y = y + sectionGap

    drawSectionTitle("Keyboard", x, y)
    y = y + lineHeight + 4

    local modifiers = {}
    if Input:isKeyboardCtrlDown() then table.insert(modifiers, "Ctrl") end
    if Input:isKeyboardAltDown() then table.insert(modifiers, "Alt") end
    if Input:isKeyboardShiftDown() then table.insert(modifiers, "Shift") end
    local modifiersStr = #modifiers > 0 and table.concat(modifiers, " + ") or "None"
    drawLabelValue("Modifiers", modifiersStr, x, y)
    y = y + lineHeight
    drawLabelValue("Keys Held", self.heldKeysStr, x, y)
    y = y + lineHeight

    local recentKbX = x + 110
    DrawEx.TextAdditive('Unageo-Medium', "Recent:", 12, x, y, 100, lineHeight, 0.9, 0.9, 0.9, 1.0, 0.0, 0.5)
    if #self.lastPressedKbKeys == 0 then
        DrawEx.TextAdditive('Unageo-Medium', "(press any)", 10, recentKbX, y, 200, lineHeight, 0.5, 0.5, 0.5, 0.7, 0.0, 0.5)
    else
        for i, entry in ipairs(self.lastPressedKbKeys) do
            local age = self.time - entry.time
            local alpha = math.max(0.2, 1.0 - (age / self.keyHistoryDuration))
            DrawEx.TextAdditive('Unageo-Medium', entry.name, 10, recentKbX, y, 80, lineHeight, 0.7, 0.7, 0.9, alpha, 0.0, 0.5)
            recentKbX = recentKbX + 25 + #entry.name * 4
        end
    end
    y = y + sectionGap

    drawSectionTitle("Gamepad", x, y)
    y = y + lineHeight + 4

    local gamepad = Input:gamepad()
    local gamepadCount = gamepad:gamepadsCount()
    drawLabelValue("Connected", tostring(gamepadCount), x, y)
    y = y + lineHeight

    if gamepadCount > 0 then
        local lStickX = Input:getValue(Button.GamepadLeftStickX)
        local lStickY = Input:getValue(Button.GamepadLeftStickY)
        local rStickX = Input:getValue(Button.GamepadRightStickX)
        local rStickY = Input:getValue(Button.GamepadRightStickY)

        local lStickXDelta = Input:gamepad():delta(GamepadAxis.LeftStickX)
        local lStickYDelta = Input:gamepad():delta(GamepadAxis.LeftStickY)
        local rStickXDelta = Input:gamepad():delta(GamepadAxis.RightStickX)
        local rStickYDelta = Input:gamepad():delta(GamepadAxis.RightStickY)

        y = y + lineHeight
        drawStickVisualizer("L Stick", lStickX, lStickY, x, y - 10, 100, 'Unageo-Medium', 11)
        drawStickVisualizer("R Stick", rStickX, rStickY, x + 120, y - 10, 100, 'Unageo-Medium', 11)
        y = y + 110

        drawLabelValue("Buttons Held", self.heldGpButtonsStr, x, y)
        y = y + lineHeight

        local recentGpX = x + 110
        DrawEx.TextAdditive('Unageo-Medium', "Recent:", 12, x, y, 100, lineHeight, 0.9, 0.9, 0.9, 1.0, 0.0, 0.5)
        if #self.lastPressedGpButtons == 0 then
            DrawEx.TextAdditive('Unageo-Medium', "(press any)", 10, recentGpX, y, 200, lineHeight, 0.5, 0.5, 0.5, 0.7, 0.0, 0.5)
        else
            for i, entry in ipairs(self.lastPressedGpButtons) do
                local age = self.time - entry.time
                local alpha = math.max(0.2, 1.0 - (age / self.keyHistoryDuration))
                DrawEx.TextAdditive('Unageo-Medium', entry.name, 10, recentGpX, y, 80, lineHeight, 0.7, 0.9, 0.7, alpha, 0.0, 0.5)
                recentGpX = recentGpX + 25 + #entry.name * 4
            end
        end
    else
        DrawEx.TextAdditive('Unageo-Medium', "  (no gamepad detected)", 10, x + 20, y, 300, 18, 0.5, 0.5, 0.5, 0.7, 0.0, 0.5)
    end
end

function InputTest:drawShipActions()
    local screenW = Window:width()
    local x = screenW - 400
    local y = 120
    local lineHeight = 22
    local sectionGap = 20
    local barWidth = 250

    drawSectionTitle("ShipActions Test", x, y)
    y = y + lineHeight + 4

    drawLabelValue("ThrustX (A/D, LStick X)", string.format("%.3f", ShipActions.ThrustX:get()), x, y)
    y = y + lineHeight
    drawLabelValue("ThrustZ (W/S, LStick Y)", string.format("%.3f", ShipActions.ThrustZ:get()), x, y)
    y = y + lineHeight
    drawLabelValue("ThrustY (Space/Ctrl, DPad)", string.format("%.3f", ShipActions.ThrustY:get()), x, y)
    y = y + lineHeight
    drawLabelValue("Roll (Q/E, LT/RT)", string.format("%.3f", ShipActions.Roll:get()), x, y)
    y = y + lineHeight
    drawLabelValue("Yaw (Mouse X, RStick X)", string.format("%.3f", ShipActions.Yaw:get()), x, y)
    y = y + lineHeight
    drawLabelValue("Pitch (Mouse Y, RStick Y)", string.format("%.3f", ShipActions.Pitch:get()), x, y)
    y = y + lineHeight
    drawLabelValue("Boost (Shift, LB)", string.format("%.3f", ShipActions.Boost:get()), x, y)
    y = y + sectionGap + 10

    drawSectionTitle("Simulated Ship State", x, y)
    y = y + lineHeight + 8

    DrawEx.TextAdditive('Unageo-Medium', "Thrust:", 11, x, y, 100, lineHeight, 0.8, 0.8, 0.8, 1.0, 0.0, 0.5)
    y = y + lineHeight
    drawAxisBar("  X (Strafe)", self.shipThrust.x, x, y, barWidth, 16)
    y = y + 22
    drawAxisBar("  Y (Vertical)", self.shipThrust.y, x, y, barWidth, 16)
    y = y + 22
    drawAxisBar("  Z (Forward)", self.shipThrust.z, x, y, barWidth, 16)
    y = y + sectionGap

    DrawEx.TextAdditive('Unageo-Medium', "Rotation:", 11, x, y, 100, lineHeight, 0.8, 0.8, 0.8, 1.0, 0.0, 0.5)
    y = y + lineHeight
    drawAxisBar("  Roll", self.shipRoll, x, y, barWidth, 16)
    y = y + 22
    drawAxisBar("  Yaw", self.shipYaw, x, y, barWidth, 16)
    y = y + 22
    drawAxisBar("  Pitch", self.shipPitch, x, y, barWidth, 16)
    y = y + sectionGap

    DrawEx.TextAdditive('Unageo-Medium', "Docked:", 11, x, y, 100, lineHeight, 0.8, 0.8, 0.8, 1.0, 0.0, 0.5)
    y = y + lineHeight
    drawProgressBar(string.format("  %s", self.inputDocked and "Yes" or "No"), self.inputDockedProgress, x, y, barWidth, 16)
    y = y + sectionGap + 10

    drawSectionTitle("Controls", x, y)
    y = y + lineHeight + 4

    local hints = {
        "W/S or LStick Y: Forward/Back thrust",
        "A/D or LStick X: Strafe left/right",
        "Space/Ctrl or DPad: Up/Down thrust",
        "Q/E or LT/RT: Roll",
        "Mouse or RStick: Yaw/Pitch",
        "Shift or LB: Boost",
    }
    for _, hint in ipairs(hints) do
        DrawEx.TextAdditive('Unageo-Medium', hint, 10, x, y, 350, lineHeight, 0.6, 0.6, 0.6, 0.9, 0.0, 0.5)
        y = y + 16
    end
end

function InputTest:drawGamepadCursor()
    if self.activeDevice ~= InputDeviceType.Gamepad then return end
    local size = 64
    local halfSize = size / 2
    RenderState.PushBlendMode(BlendMode.Alpha)
    local color = Color(1.0, 1.0, 1.0, 0.5)
    DrawEx.Ring(self.gamepadCursorX, self.gamepadCursorY, halfSize - 4, color, 1)
    DrawEx.Circle(self.gamepadCursorX, self.gamepadCursorY, halfSize - 33, color)
    RenderState.PopBlendMode()
end

function InputTest:updateBindings(dt)
    if self.runProfiler then self.profiler:start() end
    for _, binding in pairs(ShipActions) do binding:update(dt) end
    for _, binding in pairs(GamepadCursorActions) do binding:update(dt) end
    if self.runProfiler then self.profiler:stop() end
end

function InputTest:updateInputState()
    -- Ship physics actions
    self.inputShipThrustX       = ShipActions.ThrustX:get()
    self.inputShipThrustY       = ShipActions.ThrustY:get()
    self.inputShipThrustZ       = ShipActions.ThrustZ:get()
    self.inputShipRoll          = ShipActions.Roll:get()
    self.inputShipYaw           = ShipActions.Yaw:get()
    self.inputShipPitch         = ShipActions.Pitch:get()
    self.inputShipBoost         = ShipActions.Boost:get()

    -- Ship docking toggle
    if ShipActions.Dock:isHeld() and ShipActions.Dock:getHoldTime() >= self.dockHoldTime then
        if not self.dockWasHeld then
            self.inputDocked = not self.inputDocked
            self.dockWasHeld = true
        end
    end
    if not ShipActions.Dock:isHeld() then self.dockWasHeld = false end
    ShipActions.Dock:consumeHeld(self.dockHoldTime)
    self.inputDockedProgress    = math.min(1.0, ShipActions.Dock:getHoldTime() / self.dockHoldTime)

    -- Gamepad cursor actions
    self.inputCursorMoveX     = GamepadCursorActions.MoveX:get()
    self.inputCursorMoveY     = GamepadCursorActions.MoveY:get()
    self.inputCursorBoost     = GamepadCursorActions.Boost:get()
    self.inputCursorConfirm   = GamepadCursorActions.Confirm:get()
    self.inputCursorCancel    = GamepadCursorActions.Cancel:get()
end

function InputTest:updateShipPhysics(dt)
    local boostMult = 1.0 + self.inputShipBoost * 1.5
    
    local function updateAxisSmooth(current, input, accel, decay, maxVal)
        if math.abs(input) > 0.001 then
            local target = math.max(-maxVal, math.min(maxVal, input * maxVal))
            local diff = target - current
            local change = accel * dt * (math.abs(diff) > 0.1 and 1.5 or 1.0)
            if math.abs(diff) < change then return target end
            return current + change * (diff > 0 and 1 or -1)
        else
            if math.abs(current) < decay * dt then return 0 end
            return current - decay * dt * (current > 0 and 1 or -1)
        end
    end

    local function updateAxisDirect(current, input, smoothing, maxVal)
        local target = math.max(-maxVal, math.min(maxVal, input))
        local diff = target - current
        local change = smoothing * dt
        if math.abs(diff) < change then return target end
        return current + change * (diff > 0 and 1 or -1)
    end    

    self.shipThrust = Vec3f(
        updateAxisSmooth(self.shipThrust.x, self.inputShipThrustX * boostMult, self.thrustAccel, self.thrustDecay, self.maxThrust),
        updateAxisSmooth(self.shipThrust.y, self.inputShipThrustY * boostMult, self.thrustAccel, self.thrustDecay, self.maxThrust),
        updateAxisSmooth(self.shipThrust.z, self.inputShipThrustZ * boostMult, self.thrustAccel, self.thrustDecay, self.maxThrust)
    )
    self.shipRoll = updateAxisDirect(self.shipRoll, self.inputShipRoll, self.rotationAccel, self.maxRotation)
    self.shipYaw = updateAxisDirect(self.shipYaw, self.inputShipYaw, self.rotationAccel, self.maxRotation)
    self.shipPitch = updateAxisDirect(self.shipPitch, self.inputShipPitch, self.rotationAccel, self.maxRotation)
end

function InputTest:updateGamepadCursor(dt)
    local speed = 400 * dt
    self.gamepadCursorX = self.gamepadCursorX + self.inputCursorMoveX * speed
    self.gamepadCursorY = self.gamepadCursorY - self.inputCursorMoveY * speed
    self.gamepadCursorX = math.max(0, math.min(Window:width(), self.gamepadCursorX))
    self.gamepadCursorY = math.max(0, math.min(Window:height(), self.gamepadCursorY))
end

function InputTest:onInit()
    Window:setFullscreen(false, true)

    self.profiler = QuickProfiler("ActionBindingProfiler", true, false)
    self.runProfiler = true

    self.timer = DeltaTimer("InputTest")
    self.timer:start("fps", 0.1)
    self.accumulatedTime = 0
    self.frameCount = 0
    self.smoothFPS = 0
    self.time = 0

    self.showDebugLines = true
    self.debugLineDefs = {
        { label = "FPS",             getValue = function() return string.format("%d", math.floor(self.smoothFPS + 0.5)) end },
        { label = "Lua Memory (KB)", getValue = function() return string.format("%.2f", GC.GetMemory()) end }
    }

    local cam = CameraEntity()
    CameraManager:registerCamera("OrbitCam", cam)
    CameraManager:setActiveCamera("OrbitCam")

    self.buttonNameLookup = buildButtonNameLookup()

    self.deviceNames = {
        [InputDeviceType.Cursor] = "CURSOR",
        [InputDeviceType.Gamepad] = "GAMEPAD",
        [InputDeviceType.Keyboard] = "KEYBOARD",
        [InputDeviceType.Mouse] = "MOUSE",
        [InputDeviceType.Touchpad] = "TOUCHPAD",
        [InputDeviceType.SystemEvent] = "SYSTEM EVENT",
    }

    -- Pre-compute keyboard button names
    self.keyboardButtonNames = {}
    self.keyboardButtons = {
        Button.KeyboardA, Button.KeyboardB, Button.KeyboardC, Button.KeyboardD, Button.KeyboardE,
        Button.KeyboardF, Button.KeyboardG, Button.KeyboardH, Button.KeyboardI, Button.KeyboardJ,
        Button.KeyboardK, Button.KeyboardL, Button.KeyboardM, Button.KeyboardN, Button.KeyboardO,
        Button.KeyboardP, Button.KeyboardQ, Button.KeyboardR, Button.KeyboardS, Button.KeyboardT,
        Button.KeyboardU, Button.KeyboardV, Button.KeyboardW, Button.KeyboardX, Button.KeyboardY,
        Button.KeyboardZ,
        Button.KeyboardKey0, Button.KeyboardKey1, Button.KeyboardKey2, Button.KeyboardKey3,
        Button.KeyboardKey4, Button.KeyboardKey5, Button.KeyboardKey6, Button.KeyboardKey7,
        Button.KeyboardKey8, Button.KeyboardKey9,
        Button.KeyboardF1, Button.KeyboardF2, Button.KeyboardF3, Button.KeyboardF4,
        Button.KeyboardF5, Button.KeyboardF6, Button.KeyboardF7, Button.KeyboardF8,
        Button.KeyboardF9, Button.KeyboardF10, Button.KeyboardF11, Button.KeyboardF12,
        Button.KeyboardSpace, Button.KeyboardEnter, Button.KeyboardTab, Button.KeyboardEscape,
        Button.KeyboardBackspace, Button.KeyboardDelete, Button.KeyboardInsert,
        Button.KeyboardHome, Button.KeyboardEnd, Button.KeyboardPageUp, Button.KeyboardPageDown,
        Button.KeyboardUp, Button.KeyboardDown, Button.KeyboardLeft, Button.KeyboardRight,
        Button.KeyboardShiftLeft, Button.KeyboardShiftRight,
        Button.KeyboardControlLeft, Button.KeyboardControlRight,
        Button.KeyboardAltLeft, Button.KeyboardAltRight,
        Button.KeyboardComma, Button.KeyboardPeriod, Button.KeyboardSlash, Button.KeyboardBackslash,
        Button.KeyboardSemicolon, Button.KeyboardQuote, Button.KeyboardBracketLeft, Button.KeyboardBracketRight,
        Button.KeyboardMinus, Button.KeyboardEqual, Button.KeyboardBackquote,
    }
    for _, btn in ipairs(self.keyboardButtons) do
        local name = self.buttonNameLookup[btn]
        if name then
            self.keyboardButtonNames[btn] = name:gsub("^Keyboard", "")
        end
    end

    self.mouseButtonsToTrack = {
        { MouseControl.Left,    "Left" },
        { MouseControl.Middle,  "Middle" },
        { MouseControl.Right,   "Right" },
        { MouseControl.Forward, "Forward" },
        { MouseControl.Back,    "Back" },
    }

    self.gpButtons = {
        { Button.GamepadSouth,         "A/Cross" },
        { Button.GamepadEast,          "B/Circle" },
        { Button.GamepadNorth,         "Y/Triangle" },
        { Button.GamepadWest,          "X/Square" },
        { Button.GamepadLeftTrigger,   "LB" },
        { Button.GamepadRightTrigger,  "RB" },
        { Button.GamepadLeftTrigger2,  "LT" },
        { Button.GamepadRightTrigger2, "RT" },
        { Button.GamepadSelect,        "Select" },
        { Button.GamepadStart,         "Start" },
        { Button.GamepadLeftThumb,     "L3" },
        { Button.GamepadRightThumb,    "R3" },
        { Button.GamepadDPadUp,        "DPad Up" },
        { Button.GamepadDPadDown,      "DPad Down" },
        { Button.GamepadDPadLeft,      "DPad Left" },
        { Button.GamepadDPadRight,     "DPad Right" },
    }

    -- Device state
    self.activeDevice = nil
    self.gamepadCursorX = Window:width() / 2
    self.gamepadCursorY = Window:height() / 2

    -- Reusable tables for string building
    self._mouseStrParts = {}
    self._kbStrParts = {}
    self._gpStrParts = {}

    -- Cached display strings
    self.heldMouseStr = "None"
    self.heldKeysStr = "None"
    self.heldGpButtonsStr = "None"

    -- History
    self.lastPressedMouseButtons = {}
    self.lastPressedKbKeys = {}
    self.lastPressedGpButtons = {}
    self.maxKeyHistory = 8
    self.keyHistoryDuration = 2.0

    -- Ship state
    self.shipThrust = Vec3f(0, 0, 0)
    self.thrustAccel = 2.0
    self.thrustDecay = 3.0
    self.maxThrust = 1.0
    self.shipRoll = 0
    self.shipYaw = 0
    self.shipPitch = 0
    self.rotationAccel = 3.0
    self.maxRotation = 1.0
    self.shipDocked = 0
    self.dockHoldTime = 0.3
    self.inputShipThrustX = 0
    self.inputShipThrustY = 0
    self.inputShipThrustZ = 0
    self.inputShipRoll = 0
    self.inputShipYaw = 0
    self.inputShipPitch = 0
    self.inputShipBoost = 0
    self.inputDockedProgress = 0
    self.inputDocked = false
    self.dockWasHeld = false
    
    -- Gamepad cursor state
    self.inputCursorMoveX = 0
    self.inputCursorMoveY = 0
    self.inputCursorBoost = 0
    self.inputCursorConfirm = 0
    self.inputCursorCancel = 0

    EventBus:subscribe(Event.PreRender, self, self.onStatePreRender)
    EventBus:subscribe(Event.Input, self, self.onStateInput)
end

function InputTest:onStatePreRender(data)
    local dt = data:deltaTime()
    self.time = self.time + dt
    self.timer:update(dt)

    self.frameCount, self.accumulatedTime, self.smoothFPS = computeTimeAndFrameCounts(
        self.frameCount, self.accumulatedTime, dt, self.smoothFPS, self.timeScale
    )

    self:updateBindings(dt)
    self:updateInputState()
    self:updateShipPhysics(dt)
    self:updateGamepadCursor(dt)

    -- History cleanup (in-place)
    local currentTime = self.time
    local duration = self.keyHistoryDuration
    for i = #self.lastPressedMouseButtons, 1, -1 do
        if currentTime - self.lastPressedMouseButtons[i].time >= duration then
            table.remove(self.lastPressedMouseButtons, i)
        end
    end
    for i = #self.lastPressedKbKeys, 1, -1 do
        if currentTime - self.lastPressedKbKeys[i].time >= duration then
            table.remove(self.lastPressedKbKeys, i)
        end
    end
    for i = #self.lastPressedGpButtons, 1, -1 do
        if currentTime - self.lastPressedGpButtons[i].time >= duration then
            table.remove(self.lastPressedGpButtons, i)
        end
    end
end

function InputTest:onPreRender(data)
end

function InputTest:onRender(data)
    RenderCoreSystem:render(data)
    self:immediateUI(function()
        if self.showDebugLines then
            drawDebugLines(self.debugLineDefs, 40, 40, 25, 'Unageo-Medium', 11, 0.9, 0.9, 0.9, 0.9, 0.0, 0.5, "additive")
        end
        self:drawInputState()
        self:drawShipActions()
        self:drawGamepadCursor()
    end)
end

function InputTest:onStateInput(data)
    local time = self.time
    local maxHist = self.maxKeyHistory

    -- MOUSE BLOCK
    do
        local hist = self.lastPressedMouseButtons
        local parts = self._mouseStrParts
        local count = 0
        local buttons = self.mouseButtonsToTrack

        for i = 1, #buttons do
            local btn = buttons[i]
            local id = btn[1]
            local name = btn[2]

            if Input:mouse():isDown(id) then
                count = count + 1
                parts[count] = name
            end

            if Input:mouse():isPressed(id) then
                table.insert(hist, 1, { name = name, time = time })
                if #hist > maxHist then table.remove(hist) end
            end
        end

        local newStr = count > 0 and table.concat(parts, ", ", 1, count) or "None"
        if newStr ~= self.heldMouseStr then
            self.heldMouseStr = newStr
        end
    end

    -- KEYBOARD BLOCK
    do
        local hist = self.lastPressedKbKeys
        local parts = self._kbStrParts
        local count = 0
        local buttons = self.keyboardButtons
        local names = self.keyboardButtonNames

        for i = 1, #buttons do
            local btn = buttons[i]
            local name = names[btn]
            if name then
                if Input:isDown(btn) then
                    count = count + 1
                    parts[count] = name
                end

                if Input:isPressed(btn) then
                    table.insert(hist, 1, { name = name, time = time })
                    if #hist > maxHist then table.remove(hist) end
                end
            end
        end

        local newStr = count > 0 and table.concat(parts, ", ", 1, count) or "None"
        if newStr ~= self.heldKeysStr then
            self.heldKeysStr = newStr
        end
    end

    -- GAMEPAD BLOCK
    do
        local hist = self.lastPressedGpButtons
        local parts = self._gpStrParts
        local count = 0
        local buttons = self.gpButtons

        for i = 1, #buttons do
            local btn = buttons[i]
            local id = btn[1]
            local name = btn[2]

            if Input:isDown(id) then
                count = count + 1
                parts[count] = name
            end

            if Input:isPressed(id) then
                table.insert(hist, 1, { name = name, time = time })
                if #hist > maxHist then table.remove(hist) end
            end
        end

        local newStr = count > 0 and table.concat(parts, ", ", 1, count) or "None"
        if newStr ~= self.heldGpButtonsStr then
            self.heldGpButtonsStr = newStr
        end
    end
end

return InputTest
