-- Threshold for considering analog "down"/"pressed"/etc.
-- Set to a non-zero decimal to avoid noise from slight axis movement (e.g. joystick drift)
local analogBiasThreshold = 0.001

-- Threshold time for considering a digital input as "held"
local digitalHoldThreshold = 0.3

local digitalBindTypes = {
    Single = true,
    Combo = true,
}

local analogBindTypes = {
    Pair = true,
    Axis = true,
    MouseX = true,
    MouseY = true,
    MouseDX = true,
    MouseDY = true,
    MouseWheel = true,
}

---@param def DeviceBindings
---@return ControlCategory
local function getCategory(def)
    -- Check all device bindings, consider the entirely analog if any bind is analog
    for _, deviceBinds in pairs(def) do
        if type(deviceBinds) == "table" then
            for _, bind in ipairs(deviceBinds) do
                if type(bind) == "number" then
                elseif type(bind) == "table" and bind.type then
                    if analogBindTypes[bind.type] then
                        return Enums.ControlCategory.Analog
                    end
                end
            end
        end
    end
    return Enums.ControlCategory.Digital
end

---------------------------
--- ActionBinding Class
---------------------------

---@class DeviceBindings
---@field keyboard? table[]
---@field mouse? table[]
---@field gamepad? table[]

---@class ActionBinding
---@field deviceBindings DeviceBindings
---@field category ControlCategory
---@field value number
---@field prevValue number
---@field consumed boolean
---@field downTime number
---@field holdThreshold number
local ActionBinding = Class("ActionBinding", function(self, deviceBindings)
    self.deviceBindings = deviceBindings
    self.category = getCategory(deviceBindings)
    self.value = 0.0
    self.prevValue = 0.0
    self.consumed = false
    self.downTime = 0.0
    self.holdThreshold = digitalHoldThreshold
end)

---@param value number
---@param bind table
---@return number mod
local function applyModifiers(value, bind)
    local mult = bind.mult or 1.0 -- Multiplier to scale final value
    local bias = bind.bias or 0.0 -- Deadzone (bias) to prevent drift
    local expn = bind.expn or 1.0 -- Exponent to apply response curve
    
    local sign = value >= 0 and 1 or -1
    local absVal = math.abs(value)
    
    -- Check to see if within deadzone
    if absVal < bias then
        return 0.0
    end
    
    -- Normalize after deadzone bias
    local normalized = (absVal - bias) / (1.0 - bias)
    
    -- Apply exponent
    local curved = math.pow(math.max(0, normalized), expn)
    
    -- Apply multiplier and restore sign
    return mult * sign * curved
end

---@param bind number|table
---@return number
function ActionBinding:readBind(bind)    
    local t = bind.type

    if t == Enums.ControlType.Single then
        local k = Input:getValue(bind.key)
        return k
    end
    
    if t == Enums.ControlType.Combo then
        local keys = bind.keys
        for i = 1, #keys - 1 do
            local key = keys[i]
            if type(key) == "number" then
                if Input:getValue(key) == 0 then
                    return 0.0
                end
            elseif type(key) == "table" then
                if self:readBind(key) == 0 then -- recurse
                    return 0.0
                end
            end
        end

        return self:readBind(keys[#keys])
    end
    
    if t == Enums.ControlType.Pair then
        local pos = Input:getValue(bind.positive)
        local neg = Input:getValue(bind.negative)
        return pos - neg
    end
    
    if t == Enums.ControlType.GamepadAxis then
        local raw = Input:getValue(bind.button)
        return applyModifiers(raw, bind)
    end
    
    if t == Enums.ControlType.MouseX then
        local pos = Input:mouse():position()
        return pos.x * (bind.mult or 1.0)
    end
    
    if t == Enums.ControlType.MouseY then
        local pos = Input:mouse():position()
        return pos.y * (bind.mult or 1.0)
    end
    
    if t == Enums.ControlType.MouseDX then
        local delta = Input:mouse():delta()
        return delta.x * (bind.mult or 1.0)
    end
    
    if t == Enums.ControlType.MouseDY then
        local delta = Input:mouse():delta()
        return delta.y * (bind.mult or 1.0)
    end
    
    if t == Enums.ControlType.MouseWheel then
        return Input:getValue(Button.MouseScrollY) * (bind.mult or 1.0)
    end
    
    return 0.0
end

---@param bindings table[]|nil
---@return number
function ActionBinding:readDeviceBindings(bindings)
    if not bindings then return 0.0 end
    
    local value = 0.0
    for i = 1, #bindings do
        local v = self:readBind(bindings[i])
        if math.abs(v) > math.abs(value) then
            value = v
        end
    end
    return value
end

function ActionBinding:update(dt)
    self.prevValue = self.value
    self.consumed = false
    
    local kbValue = self:readDeviceBindings(self.deviceBindings.keyboard)
    local mouseValue = self:readDeviceBindings(self.deviceBindings.mouse)
    local gamepadValue = self:readDeviceBindings(self.deviceBindings.gamepad)
    
    local value = 0.0
    
    -- Determine the dominant input value
    if math.abs(kbValue) > math.abs(value) then
        value = kbValue
    end
    
    if math.abs(mouseValue) > math.abs(value) then
        value = mouseValue
    end
    
    if math.abs(gamepadValue) > math.abs(value) then
        value = gamepadValue
    end

    -- Track hold duration
    if self.category == Enums.ControlCategory.Digital and value > 0 then
        self.downTime = self.downTime + dt
    else
        self.downTime = 0.0
    end
    
    self.value = value
end

function ActionBinding:clear()
    self.prevValue = self.value
    self.value = 0.0
    self.consumed = false
    self.downTime = 0.0
end

---@return number
function ActionBinding:consume()
    if self.consumed then return 0.0 end
    self.consumed = true
    return self.value
end

---@return number
function ActionBinding:get()
    if self.consumed then return 0.0 end
    return self.value
end

---@return number
function ActionBinding:getHoldTime()
    if self.consumed then return 0.0 end
    if not self:isDown() then return 0.0 end
    return self.downTime
end

---@return boolean
function ActionBinding:isDown()
    if self.consumed then return false end
    
    if self.category == Enums.ControlCategory.Digital then
        return self.value > 0
    else
        return math.abs(self.value) > analogBiasThreshold
    end
end

---@return boolean
function ActionBinding:isPressed()
    if self.consumed then return false end
    
    if self.category == Enums.ControlCategory.Digital then
        return self.value > 0 and self.prevValue == 0
    else
        return math.abs(self.value) > analogBiasThreshold 
           and math.abs(self.prevValue) <= analogBiasThreshold
    end
end

---@return boolean
function ActionBinding:isReleased()
    if self.consumed then return false end
    
    if self.category == Enums.ControlCategory.Digital then
        return self.value == 0 and self.prevValue > 0
    else
        return math.abs(self.value) <= analogBiasThreshold 
           and math.abs(self.prevValue) > analogBiasThreshold
    end
end

---@param threshold? number
---@return boolean
function ActionBinding:isHeld(threshold)
    if self.consumed then return false end
    threshold = threshold or self.holdThreshold
    return self:isDown() and self.downTime >= threshold
end

---@return boolean
function ActionBinding:isTapped()
    if self.consumed then return false end
    return self:isReleased() and self.downTime < self.holdThreshold
end

---@return boolean
function ActionBinding:consumeDown()
    local down = self:isDown()
    if down then self.consumed = true end
    return down
end

---@return boolean
function ActionBinding:consumePressed()
    local pressed = self:isPressed()
    if pressed then self.consumed = true end
    return pressed
end

---@return boolean
function ActionBinding:consumeReleased()
    local released = self:isReleased()
    if released then self.consumed = true end
    return released
end

---@param threshold? number
---@return boolean
function ActionBinding:consumeHeld(threshold)
    local held = self:isHeld(threshold)
    if held then self.consumed = true end
    return held
end

---@return boolean
function ActionBinding:consumeTapped()
    local tapped = self:isTapped()
    if tapped then self.consumed = true end
    return tapped
end

return ActionBinding