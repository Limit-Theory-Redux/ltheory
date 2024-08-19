local Camera = require('Systems.Camera.Camera')
local Control = {}

local ControlT = Class(function(self)
    self.mult = 1.0
    self.expn = 1.0
    self.bias = 0.0
end)

local gamepadAxisIcon = {
    [Button.GamepadLeftStickX]   = 'icon/lstick',
    [Button.GamepadLeftStickY]   = 'icon/lstick',
    [Button.GamepadRightStickX]  = 'icon/rstick',
    [Button.GamepadRightStickY]  = 'icon/rstick',
    [Button.GamepadLeftTrigger]  = 'icon/ltrigger',
    [Button.GamepadRightTrigger] = 'icon/rtrigger',
}

local gamepadButtonIcon = {
    [Button.GamepadSouth]         = 'icon/a',
    [Button.GamepadEast]          = 'icon/b',
    [Button.GamepadWest]          = 'icon/x',
    [Button.GamepadNorth]         = 'icon/y',
    [Button.GamepadSelect]        = 'icon/snap',
    [Button.GamepadStart]         = 'icon/menu',
    [Button.GamepadLeftThumb]     = 'icon/lstick',
    [Button.GamepadRightThumb]    = 'icon/rstick',
    [Button.GamepadLeftTrigger2]  = 'icon/lbumper',
    [Button.GamepadRightTrigger2] = 'icon/rbumper',
    [Button.GamepadDPadUp]        = 'icon/dpad_up',
    [Button.GamepadDPadDown]      = 'icon/dpad_down',
    [Button.GamepadDPadLeft]      = 'icon/dpad_left',
    [Button.GamepadDPadRight]     = 'icon/dpad_right',
}

function ControlT:delta()
    return Control.Delta(self)
end

function ControlT:get()
    local v = self:getRaw()
    return self.mult * Math.Sign(v) * pow(
        max(0.0, (abs(v) - self.bias) / (1.0 - self.bias)),
        self.expn)
end

function ControlT:getIcon()
    local path = self:getIconPath()
    return path and Cache.Texture(path, true)
end

function ControlT:invert()
    self.mult = -self.mult
    return self
end

function ControlT:setDeadzone(deadzone)
    self.bias = deadzone
    return self
end

function ControlT:setMult(mult)
    self.mult = mult
    return self
end

function ControlT:setExponent(expn)
    self.expn = expn
    return self
end

--[[
TODO:   Integrate disabled devices by implementing :isActive and dropping
        inactive devices from consideration in And/Or.
--]]
Control.And = Subclass(ControlT, function(self, ...)
    if not ... then error() end
    self.controls = { ... }
end)

function Control.And:getRaw()
    local value = 1.0
    for i = 1, #self.controls do value = value * self.controls[i]:get() end
    return value
end

Control.Delta = Subclass(ControlT, function(self, control)
    self.control = control
    self.last = control:get()
end)

function Control.Delta:getIconPath()
    return self.control:getIconPath()
end

--[[
TODO:   This must be split into an update & cached value, such that the
        control can be retrieved multiple times in one frame without affecting
        the delta value. In general, deltas require extra handling. This is
        also where flattening bindings will come into play and potentially
        get tricky. After PAX, having used this control system a bit, we'll
        need to come back and assess what we've learned (and, in particular,
        if deltas are the natural splitting point where we move to events
        rather than 'continuous' controls, which deltas are not.)
--]]
function Control.Delta:getRaw()
    local curr = self.control:get()
    local last = self.last
    self.last = curr
    return curr - last
end

Control.GamepadAxis = Subclass(ControlT, function(self, axis)
    self.axis = axis
end)

function Control.GamepadAxis:getIconPath()
    return gamepadAxisIcon[self.axis]
end

function Control.GamepadAxis:getRaw()
    return Input:getValue(self.axis)
end

Control.GamepadButton = Subclass(ControlT, function(self, button)
    if not button then error() end
    self.button = button
end)

function Control.GamepadButton:getIconPath()
    return gamepadButtonIcon[self.button]
end

function Control.GamepadButton:getRaw()
    return Input:getValue(self.button)
end

Control.GamepadButtonPressed = Subclass(ControlT, function(self, button)
    if not button then error() end
    self.button = button
end)

function Control.GamepadButtonPressed:getIconPath()
    return gamepadButtonIcon[self.button]
end

function Control.GamepadButtonPressed:getRaw()
    return Input:isPressed(self.button) and 1.0 or 0.0
end

Control.GamepadButtonReleased = Subclass(ControlT, function(self, button)
    if not button then error() end
    self.button = button
end)

function Control.GamepadButtonReleased:getIconPath()
    return gamepadButtonIcon[self.button]
end

function Control.GamepadButtonReleased:getRaw()
    return Input:isReleased(self.button) and 1.0 or 0.0
end

Control.Key = Subclass(ControlT, function(self, key)
    if not key then error() end
    self.key = key
end)

function Control.Key:getRaw()
    return Input:getValue(self.key)
end

Control.Alt    = function() return Control.Or(Control.Key(Button.KeyboardAltLeft), Control.Key(Button.KeyboardAltRight)) end
Control.Ctrl   = function()
    return Control.Or(Control.Key(Button.KeyboardControlLeft),
        Control.Key(Button.KeyboardControlRight))
end
Control.Shift  = function()
    return Control.Or(Control.Key(Button.KeyboardShiftLeft),
        Control.Key(Button.KeyboardShiftRight))
end

Control.MouseX = Subclass(ControlT, function(self) end)
Control.MouseY = Subclass(ControlT, function(self) end)

function Control.MouseX:getRaw()
    local c = Camera.get()
    local m = Input:mouse():position().x
    return Math.Clamp(2.0 * (m - c.x) / c.sx - 1.0, -1.0, 1.0)
end

function Control.MouseY:getRaw()
    local c = Camera.get()
    local m = Input:mouse():position().y
    return Math.Clamp(2.0 * (m - c.y) / c.sy - 1.0, -1.0, 1.0)
end

--[[
TODO:   Really a delta. Unify with MouseX/Y + think about out how 'mouse
        relative to center' best fits into this architecture.
--]]
Control.MouseDX = Subclass(ControlT, function(self) end)
Control.MouseDY = Subclass(ControlT, function(self) end)

function Control.MouseDX:getRaw()
    local md = Input:mouse():delta()
    return md.x
end

function Control.MouseDY:getRaw()
    local md = Input:mouse():delta()
    return md.y
end

Control.MouseButton = Subclass(ControlT, function(self, button)
    self.button = button
end)

function Control.MouseButton:getRaw()
    return Input:getValue(self.button)
end

Control.MouseWheel = Subclass(ControlT, function(self) end)

--[[
TODO:   Unlike other signals, this won't be clamped to [-1, 1]. Problem?
NOTE:   In reality, this is a delta.
NOTE:   Yes, this has already caused problems in the form of dt-dependence
--]]
function Control.MouseWheel:getRaw()
    return Input:getValue(Button.MouseScrollY)
end

Control.Null = Subclass(ControlT, function(self) end)

function Control.Null:getRaw()
    return 0
end

Control.Pair = Subclass(ControlT, function(self, pos, neg)
    self.pos = pos
    self.neg = neg
end)

function Control.Pair:getRaw()
    return self.pos:get() - self.neg:get()
end

Control.Or = Subclass(ControlT, function(self, ...)
    self.controls = { ... }
end)

function Control.Or:getRaw()
    local value = 0.0
    for i = 1, #self.controls do
        local v = self.controls[i]:get()
        if abs(v) > abs(value) then value = v end
    end
    return value
end

return Control

-- TODO: Don't trigger bindings without modifiers when a modifier is pressed.
-- TODO: It should be possible to get the delta from a non-delta control.
