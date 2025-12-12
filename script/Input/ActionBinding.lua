--[[
    Control category definitions:
    we separate controls into two categories for the purpose of handling isDown/isPressed/isReleased 
    logic differently based on type.
    - Binary controls (keys, buttons) where value is either a fixed 0.0 or 1.0
    - Analog controls (axes, mouse movement) where value can vary between -1.0 and 1.0
]]
---@alias ControlCategory "Binary" | "Analog"

-- threshold for considering analog "down"/"pressed"/etc.
-- set to a non-zero decimal to avoid noise from slight axis movement (e.g. joystick drift)
local analogThreshold = 0.001

local binaryBindTypes = {
    Key = true,
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
    -- check all device bindings
    -- if any analog bind found, consider it an Analog
    for _, deviceBinds in pairs(def) do
        if type(deviceBinds) == "table" then
            for _, bind in ipairs(deviceBinds) do
                -- raw button is Binary
                if type(bind) == "number" then
                    -- continue
                elseif type(bind) == "table" and bind.type then
                    if analogBindTypes[bind.type] then
                        return "Analog"
                    end
                end
            end
        end
    end
    return "Binary"
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
local ActionBinding = Class("ActionBinding", function(self, deviceBindings)
    self.deviceBindings = deviceBindings
    self.category = getCategory(deviceBindings)
    self.value = 0.0
    self.prevValue = 0.0
    self.consumed = false
end)

---@param value number
---@param bind table
---@return number mod
local function applyModifiers(value, bind)
    local mult = bind.mult or 1.0 -- multiplier to scale final value
    local bias = bind.bias or 0.0 -- deadzone (bias) to prevent drift
    local expn = bind.expn or 1.0 -- exponent to apply response curve
    
    local sign = value >= 0 and 1 or -1
    local absVal = math.abs(value)
    
    -- check to see if within deadzone
    if absVal < bias then
        return 0.0
    end
    
    -- normalize after deadzone bias
    local normalized = (absVal - bias) / (1.0 - bias)
    
    -- apply exponent
    local curved = math.pow(math.max(0, normalized), expn)
    
    -- apply multiplier and restore sign
    return mult * sign * curved
end

---@param bind number|table
---@return number
function ActionBinding:readBind(bind)
    if type(bind) == "number" then
        return Input:getValue(bind)
    end
    
    local t = bind.type

    -- single key/button
    if t == "Key" then
        local k = Input:getValue(bind.key)
        return k
    end
    
    -- combo - all keys must be held
    if t == "Combo" then
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
    
    -- pair: positive - negative
    if t == "Pair" then
        local pos = Input:getValue(bind.positive)
        local neg = Input:getValue(bind.negative)
        return pos - neg
    end
    
    -- axis: applies special axes modifiers
    if t == "Axis" then
        local raw = Input:getValue(bind.button)
        return applyModifiers(raw, bind)
    end
    
    -- mouse positions and deltas
    if t == "MouseX" then
        local pos = Input:mouse():position()
        return pos.x * (bind.mult or 1.0)
    end
    
    if t == "MouseY" then
        local pos = Input:mouse():position()
        return pos.y * (bind.mult or 1.0)
    end
    
    if t == "MouseDX" then
        local delta = Input:mouse():delta()
        return delta.x * (bind.mult or 1.0)
    end
    
    if t == "MouseDY" then
        local delta = Input:mouse():delta()
        return delta.y * (bind.mult or 1.0)
    end
    
    if t == "MouseWheel" then
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

function ActionBinding:update()
    self.prevValue = self.value
    self.consumed = false
    
    -- read from all devices, take highest absolute value
    local kbValue = self:readDeviceBindings(self.deviceBindings.keyboard)
    local mouseValue = self:readDeviceBindings(self.deviceBindings.mouse)
    local gamepadValue = self:readDeviceBindings(self.deviceBindings.gamepad)
    
    -- find the value with the highest magnitude
    local value = 0.0
    
    if math.abs(kbValue) > math.abs(value) then
        value = kbValue
    end
    
    if math.abs(mouseValue) > math.abs(value) then
        value = mouseValue
    end
    
    if math.abs(gamepadValue) > math.abs(value) then
        value = gamepadValue
    end
    
    self.value = value
end

function ActionBinding:clear()
    self.prevValue = self.value
    self.value = 0.0
    self.consumed = false
end

---@return number
function ActionBinding:get()
    if self.consumed then return 0.0 end
    return self.value
end

---@return number
function ActionBinding:consume()
    if self.consumed then return 0.0 end
    self.consumed = true
    return self.value
end

---@return boolean
function ActionBinding:isDown()
    if self.consumed then return false end
    
    if self.category == "Binary" then
        return self.value > 0
    else
        return math.abs(self.value) > analogThreshold
    end
end

---@return boolean
function ActionBinding:isPressed()
    if self.consumed then return false end
    
    if self.category == "Binary" then
        return self.value > 0 and self.prevValue == 0
    else
        return math.abs(self.value) > analogThreshold 
           and math.abs(self.prevValue) <= analogThreshold
    end
end

---@return boolean
function ActionBinding:isReleased()
    if self.consumed then return false end
    
    if self.category == "Binary" then
        return self.value == 0 and self.prevValue > 0
    else
        return math.abs(self.value) <= analogThreshold 
           and math.abs(self.prevValue) > analogThreshold
    end
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

return ActionBinding