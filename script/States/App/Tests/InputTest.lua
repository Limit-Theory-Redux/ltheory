local Application         = require('States.Application')

local InputTest           = Subclass("InputTest", Application)

local Cache               = require("Render.Cache")
local CameraSystem        = require("Modules.Rendering.Systems.CameraSystem")
local CameraEntity        = require("Modules.Rendering.Entities").Camera
local DeltaTimer          = require("Shared.Tools.DeltaTimer")
local DrawEx              = require("UI.DrawEx")
local GC                  = require("Core.Util.GC")
local QuickProfiler       = require("Shared.Tools.QuickProfiler")
local RenderCoreSystem    = require("Modules.Rendering.Systems.RenderCoreSystem")

-- ActionBinding system
local ShipActions         = require('Input.ActionBindings.ShipActions')

local function DrawLabelValue(label, value, x, y, font, fontSize, r, g, b, a, blendMode)
    local fontName = font or 'Unageo-Medium'
    local size = fontSize or 12
    local cr, cg, cb, ca = r or 0.9, g or 0.9, b or 0.9, a or 1.0
    local mode = blendMode or "additive"
    local drawFunc = mode == "alpha" and DrawEx.TextAlpha or DrawEx.TextAdditive
    local text = string.format("%s: %s", label, value)
    drawFunc(fontName, text, size, x, y, 400, size + 4, cr, cg, cb, ca, 0.0, 0.5)
end

local function DrawSectionTitle(title, x, y, font, fontSize, r, g, b, a, blendMode)
    local fontName = font or 'Unageo-Medium'
    local size = fontSize or 14
    local cr, cg, cb, ca = r or 1.0, g or 0.8, b or 0.2, a or 1.0
    local mode = blendMode or "additive"
    local drawFunc = mode == "alpha" and DrawEx.TextAlpha or DrawEx.TextAdditive
    drawFunc(fontName, "[ " .. title .. " ]", size, x, y, 400, size + 4, cr, cg, cb, ca, 0.0, 0.5)
end

local function DrawAxisBar(label, value, x, y, width, height, font, fontSize)
    local fontName = font or 'Unageo-Medium'
    local size = fontSize or 11
    
    -- Draw label
    DrawEx.TextAdditive(fontName, label, size, x, y, 100, height, 0.9, 0.9, 0.9, 1.0, 0.0, 0.5)
    
    -- Bar background
    local barX = x + 100
    local barWidth = width - 100
    local halfWidth = barWidth / 2
    local centerX = barX + halfWidth
    
    -- Need to set blend mode for Draw.Rect to render properly
    RenderState.PushBlendMode(BlendMode.Alpha)
    
    -- Draw background bar (gray) - Draw.Rect takes (x, y, width, height)
    DrawEx.Rect(barX, y, barWidth, height, Color(0.2, 0.2, 0.2, 0.8))
    
    -- Draw center line
    DrawEx.Rect(centerX - 1, y, 2, height,Color(0.5, 0.5, 0.5, 1.0))
    
    -- Draw value bar
    local fillWidth = math.abs(value) * halfWidth
    if value >= 0 then
        DrawEx.Rect(centerX, y + 2, fillWidth, height - 4, Color(0.2, 0.8, 0.2, 1.0))
    else
        DrawEx.Rect(centerX - fillWidth, y + 2, fillWidth, height - 4, Color(0.8, 0.2, 0.2, 1.0))
    end
    
    RenderState.PopBlendMode()
    
    -- Draw value text
    local valueStr = string.format("%.2f", value)
    DrawEx.TextAdditive(fontName, valueStr, size - 1, barX + barWidth + 10, y, 60, height, 0.7, 0.7, 0.7, 1.0, 0.0, 0.5)
end

local function DrawStickVisualizer(label, xValue, yValue, x, y, size, font, fontSize)
    local fontName = font or 'Unageo-Medium'
    local textSize = fontSize or 11
    local halfSize = size / 2
    local centerX = x + halfSize
    local centerY = y + halfSize
    
    -- Draw label above
    DrawEx.TextAdditive(fontName, label, textSize, x, y - textSize - 4, size, textSize + 4, 0.9, 0.9, 0.9, 1.0, 0.5, 0.5)
    
    -- Draw background box using DrawEx.Rect for proper rendering
    DrawEx.Rect(x, y, size, size, Color(0.15, 0.15, 0.15, 0.9))
    
    -- Draw crosshairs
    DrawEx.Rect(x, centerY - 1, size, 2, Color(0.4, 0.4, 0.4, 1.0))
    DrawEx.Rect(centerX - 1, y, 2, size, Color(0.4, 0.4, 0.4, 1.0))
    
    -- Draw deadzone indicator
    local deadzoneRadius = size * 0.1
    DrawEx.Rect(centerX - deadzoneRadius, centerY - deadzoneRadius, deadzoneRadius * 2, deadzoneRadius * 2, Color(0.3, 0.3, 0.3, 0.8))
    
    -- Draw stick position
    local dotX = centerX + (xValue * halfSize * 0.9) - 6
    local dotY = centerY + (yValue * halfSize * 0.9) - 6
    local dotSize = 12
    
    -- Dot color based on magnitude
    local magnitude = math.sqrt(xValue * xValue + yValue * yValue)
    local intensity = math.min(magnitude, 1.0)
    DrawEx.Rect(dotX, dotY, dotSize, dotSize, Color(0.2 + intensity * 0.6, 0.8 - intensity * 0.4, 0.2, 1.0))
    
    -- Draw values below
    local valuesStr = string.format("X:%.2f Y:%.2f", xValue, yValue)
    DrawEx.TextAdditive(fontName, valuesStr, textSize - 1, x, y + size + 4, size, textSize + 2, 0.6, 0.6, 0.6, 1.0, 0.5, 0.5)
end

local function isLineDef(item)
    return type(item) == "table" 
        and type(item.label) == "string" 
        and type(item.getValue) == "function"
end

---@class DebugLineDef
---@field label string
---@field getValue fun(): string

local function DrawDebugLines(lineDefs, startX, startY, lineHeight, font, fontSize, r, g, b, a, alignX, alignY, blendMode)
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
        if isLineDef(item) then
            ---@cast item DebugLineDef
            local value = item.getValue() -- call the getValue function to get current value
            line = string.format("%s: %s", item.label, tostring(value))
        elseif type(item) == "string" then
            -- static string
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
    
    -- [[ mouse state ]]
    DrawSectionTitle("Mouse", x, y)
    y = y + lineHeight + 4
    
    local mousePos = Input:mouse():position()
    local mouseDelta = Input:mouse():delta()
    local mouseScroll = Input:mouse():scroll()
    local mouseInWindow = Input:mouse():inWindow()
    
    DrawLabelValue("Position", string.format("%.0f, %.0f", mousePos.x, mousePos.y), x, y)
    y = y + lineHeight
    DrawLabelValue("Delta", string.format("%.1f, %.1f", mouseDelta.x, mouseDelta.y), x, y)
    y = y + lineHeight
    DrawLabelValue("Scroll", string.format("%.1f, %.1f", mouseScroll.x, mouseScroll.y), x, y)
    y = y + lineHeight
    DrawLabelValue("In Window", mouseInWindow and "Yes" or "No", x, y)
    y = y + lineHeight
    
    -- mouse buttons
    local mouseButtons = { "Left", "Middle", "Right", "Forward", "Back" }
    local mouseButtonEnums = { MouseControl.Left, MouseControl.Middle, MouseControl.Right, MouseControl.Forward, MouseControl.Back }
    local heldMouseButtons = {}
    for i, btn in ipairs(mouseButtonEnums) do
        if Input:mouse():isDown(btn) then
            table.insert(heldMouseButtons, mouseButtons[i])
        end
    end
    local mouseButtonsStr = #heldMouseButtons > 0 and table.concat(heldMouseButtons, ", ") or "None"
    DrawLabelValue("Buttons Held", mouseButtonsStr, x, y)
    y = y + lineHeight

    -- recent mouse button history
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
    
    -- [[ keyboard state ]]
    DrawSectionTitle("Keyboard", x, y)
    y = y + lineHeight + 4
    
    -- modifiers
    local modifiers = {}
    if Input:isKeyboardCtrlDown() then table.insert(modifiers, "Ctrl") end
    if Input:isKeyboardAltDown() then table.insert(modifiers, "Alt") end
    if Input:isKeyboardShiftDown() then table.insert(modifiers, "Shift") end
    local modifiersStr = #modifiers > 0 and table.concat(modifiers, " + ") or "None"
    DrawLabelValue("Modifiers", modifiersStr, x, y)
    y = y + lineHeight
    
    -- currently held keys
    local heldStr = #self.heldKeys > 0 and table.concat(self.heldKeys, ", ") or "None"
    DrawLabelValue("Keys Held", heldStr, x, y)
    y = y + lineHeight
    
    -- recent key history
    local recentKbX = x + 110
    DrawEx.TextAdditive('Unageo-Medium', "Recent:", 12, x, y, 100, lineHeight, 0.9, 0.9, 0.9, 1.0, 0.0, 0.5)
    if #self.lastPressedKbKeys == 0 then
        DrawEx.TextAdditive('Unageo-Medium', "(press any)", 10, recentKbX, y, 200, lineHeight, 0.5, 0.5, 0.5, 0.7, 0.0, 0.5)
    else
        for i, entry in ipairs(self.lastPressedKbKeys) do
            local age = self.time - entry.time
            local alpha = math.max(0.2, 1.0 - (age / self.keyHistoryDuration))
            DrawEx.TextAdditive('Unageo-Medium', entry.name, 10, recentKbX, y, 80, lineHeight, 0.7, 0.7, 0.9, alpha, 0.0, 0.5)
            recentKbX = recentKbX + 25 + #entry.name * 4  -- adjust spacing based on text length
        end
    end
    y = y + sectionGap
    
    -- [[ gamepad State ]]
    DrawSectionTitle("Gamepad", x, y)
    y = y + lineHeight + 4
    
    local gamepad = Input:gamepad()
    local gamepadCount = gamepad:gamepadsCount()
    DrawLabelValue("Connected", tostring(gamepadCount), x, y)
    y = y + lineHeight
    
    if gamepadCount > 0 then
        local lStickX = Input:getValue(Button.GamepadLeftStickX)
        local lStickY = Input:getValue(Button.GamepadLeftStickY)
        local rStickX = Input:getValue(Button.GamepadRightStickX)
        local rStickY = Input:getValue(Button.GamepadRightStickY)

        y = y + lineHeight
        DrawStickVisualizer("L Stick", lStickX, lStickY, x, y - 10, 100, 'Unageo-Medium', 11)
        DrawStickVisualizer("R Stick", rStickX, rStickY, x + 120, y - 10, 100, 'Unageo-Medium', 11)
        y = y + 110
        
        -- buttons
        local gpButtons = {
            { Button.GamepadSouth, "A/Cross" },
            { Button.GamepadEast, "B/Circle" },
            { Button.GamepadNorth, "Y/Triangle" },
            { Button.GamepadWest, "X/Square" },
            { Button.GamepadLeftTrigger, "RB" },
            { Button.GamepadRightTrigger, "LB" },
            { Button.GamepadLeftTrigger2, "LT" },
            { Button.GamepadRightTrigger2, "RT" },
            { Button.GamepadSelect, "Select" },
            { Button.GamepadStart, "Start" },
            { Button.GamepadLeftThumb, "L3" },
            { Button.GamepadRightThumb, "R3" },
            { Button.GamepadDPadUp, "DPad Up" },
            { Button.GamepadDPadDown, "DPad Down" },
            { Button.GamepadDPadLeft, "DPad Left" },
            { Button.GamepadDPadRight, "DPad Right" },
        }
        local heldGpButtons = {}
        for _, btnInfo in ipairs(gpButtons) do
            if Input:isDown(btnInfo[1]) then
                table.insert(heldGpButtons, btnInfo[2])
            end
        end
        local gpButtonsStr = #heldGpButtons > 0 and table.concat(heldGpButtons, ", ") or "None"
        DrawLabelValue("Buttons Held", gpButtonsStr, x, y)
        y = y + lineHeight
        
        -- recent button history
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

function InputTest:drawShipActionsTest()
    local screenW = Window:width()
    local x = screenW - 400
    local y = 120
    local lineHeight = 22
    local sectionGap = 20
    local barWidth = 250
    
    DrawSectionTitle("ShipActions - Input", x, y)
    y = y + lineHeight + 4
    
    DrawLabelValue("ThrustX (A/D, LStick X)", string.format("%.3f", ShipActions.ThrustX:get()), x, y)
    y = y + lineHeight
    DrawLabelValue("ThrustZ (W/S, LStick Y)", string.format("%.3f", ShipActions.ThrustZ:get()), x, y)
    y = y + lineHeight
    DrawLabelValue("ThrustY (Space/Ctrl, DPad)", string.format("%.3f", ShipActions.ThrustY:get()), x, y)
    y = y + lineHeight
    DrawLabelValue("Roll (Q/E, LT/RT)", string.format("%.3f", ShipActions.Roll:get()), x, y)
    y = y + lineHeight
    DrawLabelValue("Yaw (Mouse X, RStick X)", string.format("%.3f", ShipActions.Yaw:get()), x, y)
    y = y + lineHeight
    DrawLabelValue("Pitch (Mouse Y, RStick Y)", string.format("%.3f", ShipActions.Pitch:get()), x, y)
    y = y + lineHeight
    DrawLabelValue("Boost (Shift, LB)", string.format("%.3f", ShipActions.Boost:get()), x, y)
    y = y + sectionGap + 10
    
    DrawSectionTitle("Ship State (Simulated)", x, y)
    y = y + lineHeight + 8
    
    -- thrust bars
    DrawEx.TextAdditive('Unageo-Medium', "Thrust:", 11, x, y, 100, lineHeight, 0.8, 0.8, 0.8, 1.0, 0.0, 0.5)
    y = y + lineHeight
    
    DrawAxisBar("  X (Strafe)", self.shipThrust.x, x, y, barWidth, 16)
    y = y + 22
    DrawAxisBar("  Y (Vertical)", self.shipThrust.y, x, y, barWidth, 16)
    y = y + 22
    DrawAxisBar("  Z (Forward)", self.shipThrust.z, x, y, barWidth, 16)
    y = y + sectionGap
    
    -- rotation bars
    DrawEx.TextAdditive('Unageo-Medium', "Rotation:", 11, x, y, 100, lineHeight, 0.8, 0.8, 0.8, 1.0, 0.0, 0.5)
    y = y + lineHeight
    
    DrawAxisBar("  Roll", self.shipRoll, x, y, barWidth, 16)
    y = y + 22
    DrawAxisBar("  Yaw", self.shipYaw, x, y, barWidth, 16)
    y = y + 22
    DrawAxisBar("  Pitch", self.shipPitch, x, y, barWidth, 16)
    y = y + sectionGap + 10
    
    -- controls hint
    DrawSectionTitle("Controls", x, y)
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

function InputTest:updateShipPhysics(dt)
    local thrustX = ShipActions.ThrustX:get()
    local thrustY = ShipActions.ThrustY:get()
    local thrustZ = ShipActions.ThrustZ:get()
    local roll = ShipActions.Roll:get()
    local yaw = ShipActions.Yaw:get()
    local pitch = ShipActions.Pitch:get()
    local boost = ShipActions.Boost:get()
    local boostMult = 1.0 + boost * 1.5
    
    local function updateAxisSmooth(current, input, accel, decay, maxVal)
        if math.abs(input) > 0.001 then
            local target = math.max(-maxVal, math.min(maxVal, input * maxVal))
            local diff = target - current
            local change = accel * dt * (math.abs(diff) > 0.1 and 1.5 or 1.0)
            if math.abs(diff) < change then
                return target
            else
                return current + change * (diff > 0 and 1 or -1)
            end
        else
            if math.abs(current) < decay * dt then
                return 0
            else
                return current - decay * dt * (current > 0 and 1 or -1)
            end
        end
    end

    local function updateAxisDirect(current, input, smoothing, maxVal)
        local target = math.max(-maxVal, math.min(maxVal, input))
        local diff = target - current
        local change = smoothing * dt
        if math.abs(diff) < change then
            return target
        else
            return current + change * (diff > 0 and 1 or -1)
        end
    end
    
    -- update thrust axes
    self.shipThrust = Vec3f(
        updateAxisSmooth(self.shipThrust.x, thrustX * boostMult, self.thrustAccel, self.thrustDecay, self.maxThrust),
        updateAxisSmooth(self.shipThrust.y, thrustY * boostMult, self.thrustAccel, self.thrustDecay, self.maxThrust),
        updateAxisSmooth(self.shipThrust.z, thrustZ * boostMult, self.thrustAccel, self.thrustDecay, self.maxThrust)
    )
    
    -- update rotation axes
    self.shipRoll = updateAxisDirect(self.shipRoll, roll, self.rotationAccel, self.maxRotation)
    self.shipYaw = updateAxisDirect(self.shipYaw, yaw, self.rotationAccel, self.maxRotation)
    self.shipPitch = updateAxisDirect(self.shipPitch, pitch, self.rotationAccel, self.maxRotation)
end

function InputTest:onInit()

    -- window setup
    Window:setFullscreen(false, true)

    -- init timer
    self.timer = DeltaTimer("InputTest")
    self.timer:start("fps", 0.1)
    self.accumulatedTime = 0
    self.frameCount = 0
    self.smoothFPS = 0
    self.time = 0

    -- init debug
    self.showDebugLines = true
    self.debugLineDefs = {
        { label = "FPS",                getValue = function() return string.format("%d", math.floor(self.smoothFPS + 0.5)) end },
        { label = "Lua Memory (KB)",    getValue = function() return string.format("%.2f", GC.GetMemory()) end }
    }

    -- init camera
    self.camPos = Vec3f(0, 0, 10)
    self.focusPos = Vec3f(0, 0, 0)
    local cam = CameraEntity()
    CameraSystem:setCamera(cam)
    CameraSystem.currentCameraTransform:setPos(Position(self.camPos.x, self.camPos.y, self.camPos.z))
    CameraSystem.currentCameraTransform:setRot(Quat.LookAt(self.camPos, self.focusPos, Vec3f(0, 1, 0)))

    -- input state tracking
    self.buttonNameLookup = buildButtonNameLookup()
    self.lastPressedMouseButtons = {}
    self.lastPressedKbKeys = {}
    self.lastPressedGpButtons = {}
    self.maxKeyHistory = 8
    self.keyHistoryDuration = 2.0
    self.heldKeys = {}
    
    -- simulated ship state (for ActionBinding testing)
    self.shipThrust = Vec3f(0, 0, 0)
    self.thrustAccel = 2.0
    self.thrustDecay = 3.0
    self.maxThrust = 1.0
    self.shipRoll = 0
    self.shipYaw = 0
    self.shipPitch = 0
    self.rotationAccel = 3.0
    self.rotationDecay = 4.0
    self.maxRotation = 1.0

    -- events
    EventBus:subscribe(Event.PreRender, self, self.onStatePreRender)
    EventBus:subscribe(Event.Input, self, self.onStateInput)
end

function InputTest:onStatePreRender(data)
    -- update timers
    local dt = data:deltaTime()
    local scaledDT = dt * (self.timeScale or 1)
    self.timer:update(dt)
    self.time = self.time + dt

    -- update fps counter state
    self.frameCount, self.accumulatedTime, self.smoothFPS = computeTimeAndFrameCounts(
        self.frameCount,
        self.accumulatedTime,
        dt,
        self.smoothFPS,
        self.timeScale
    )

    -- update all shipActions bindings
    for _, binding in pairs(ShipActions) do
        binding:update()
    end
    
    -- update ship physics based on ActionBindings
    self:updateShipPhysics(dt)

    -- clean up old input history entries
    local currentTime = self.time
    -- clean up old mouse button history
    local newMouseHistory = {}
    for _, entry in ipairs(self.lastPressedMouseButtons) do
        if currentTime - entry.time < self.keyHistoryDuration then
            table.insert(newMouseHistory, entry)
        end
    end
    self.lastPressedMouseButtons = newMouseHistory

    -- clean up old key history
    local newKeyHistory = {}
    for _, entry in ipairs(self.lastPressedKbKeys) do
        if currentTime - entry.time < self.keyHistoryDuration then
            table.insert(newKeyHistory, entry)
        end
    end
    self.lastPressedKbKeys = newKeyHistory
    
    -- clean up old gamepad button history
    local newGpHistory = {}
    for _, entry in ipairs(self.lastPressedGpButtons) do
        if currentTime - entry.time < self.keyHistoryDuration then
            table.insert(newGpHistory, entry)
        end
    end
    self.lastPressedGpButtons = newGpHistory
end

function InputTest:onPreRender(data)
    
end

function InputTest:onRender(data)
    RenderCoreSystem:render(data)
    self:immediateUI(function()
        if self.showDebugLines then
            DrawDebugLines(self.debugLineDefs, 40, 40, 25, 'Unageo-Medium', 11, 0.9, 0.9, 0.9, 0.9, 0.0, 0.5, "additive")
        end
        
        self:drawInputState()
        self:drawShipActionsTest()
    end)
end

function InputTest:onStateInput(data)
    local mouseButtonsToTrack = {
        { MouseControl.Left, "Left" },
        { MouseControl.Middle, "Middle" },
        { MouseControl.Right, "Right" },
        { MouseControl.Forward, "Forward" },
        { MouseControl.Back, "Back" },
    }

    local newHeldMouseBtns = {}
    -- track mouse button presses
    for _, btn in ipairs(mouseButtonsToTrack) do
        if Input:mouse():isDown(btn[1]) then
            table.insert(newHeldMouseBtns, btn[2])
            
            -- check if this is a new press
            if Input:mouse():isPressed(btn[1]) then
                -- add to history
                table.insert(self.lastPressedMouseButtons, 1, { name = btn[2], time = self.time })
                -- trim history
                while #self.lastPressedMouseButtons > self.maxKeyHistory do
                    table.remove(self.lastPressedMouseButtons)
                end
            end
        end
    end
    self.heldMouseButtons = newHeldMouseBtns
    
    local keyboardButtons = {
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
    
    -- update held keys and detect new presses
    local newHeldKeys = {}
    for _, btn in ipairs(keyboardButtons) do
        if Input:isDown(btn) then
            local name = self.buttonNameLookup[btn] or tostring(btn)
            -- clean up the name (remove "Keyboard" prefix)
            name = name:gsub("^Keyboard", "")
            table.insert(newHeldKeys, name)
            
            -- check if this is a new press
            if Input:isPressed(btn) then
                -- add to history
                table.insert(self.lastPressedKbKeys, 1, { name = name, time = self.time })
                -- trim history
                while #self.lastPressedKbKeys > self.maxKeyHistory do
                    table.remove(self.lastPressedKbKeys)
                end
            end
        end
    end
    self.heldKeys = newHeldKeys
    
    -- track gamepad button presses
    local gpButtons = {
        { Button.GamepadSouth, "A/Cross" },
        { Button.GamepadEast, "B/Circle" },
        { Button.GamepadNorth, "Y/Triangle" },
        { Button.GamepadWest, "X/Square" },
        { Button.GamepadLeftTrigger2, "LT" },
        { Button.GamepadRightTrigger2, "RT" },
        { Button.GamepadLeftTrigger, "LB" },
        { Button.GamepadRightTrigger, "RB" },
        { Button.GamepadSelect, "Select" },
        { Button.GamepadStart, "Start" },
        { Button.GamepadLeftThumb, "L3" },
        { Button.GamepadRightThumb, "R3" },
        { Button.GamepadDPadUp, "DPad Up" },
        { Button.GamepadDPadDown, "DPad Down" },
        { Button.GamepadDPadLeft, "DPad Left" },
        { Button.GamepadDPadRight, "DPad Right" },
    }
    
    for _, btnInfo in ipairs(gpButtons) do
        if Input:isPressed(btnInfo[1]) then
            table.insert(self.lastPressedGpButtons, 1, { name = btnInfo[2], time = self.time })
            while #self.lastPressedGpButtons > self.maxKeyHistory do
                table.remove(self.lastPressedGpButtons)
            end
        end
    end
end


return InputTest