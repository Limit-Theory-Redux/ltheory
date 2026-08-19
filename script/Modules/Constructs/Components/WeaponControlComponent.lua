local Component = require("Core.ECS.Component")

---@class WeaponControlComponent: Component
---@overload fun(mode?: string, sequence?: string[]): WeaponControlComponent
local WeaponControlComponent = Subclass("WeaponControlComponent", Component, function(self, mode, sequence)
    self:setComponentName("WeaponControl")
    local fireModes = Enums.Weapon.FireMode
    self.mode = mode or fireModes.Volley
    assert(self.mode == fireModes.Volley or self.mode == fireModes.Sequence)

    self.sequence = {}
    for _, mountId in ipairs(sequence or {}) do
        table.insert(self.sequence, mountId)
    end

    self.sequenceIndex = 1
    self.triggerHeld = false
    self.active = false
    self.interShotGap = 0
    self.shotSerial = 0
end)

function WeaponControlComponent:getMode()
    return self.mode
end

---@param mode string
function WeaponControlComponent:setMode(mode)
    local fireModes = Enums.Weapon.FireMode
    assert(mode == fireModes.Volley or mode == fireModes.Sequence)
    self.mode = mode
end

function WeaponControlComponent:getSequence()
    return self.sequence
end

---@param sequence string[]
function WeaponControlComponent:setSequence(sequence)
    self.sequence = {}
    for _, mountId in ipairs(sequence) do
        table.insert(self.sequence, mountId)
    end
    self.sequenceIndex = 1
end

function WeaponControlComponent:getSequenceIndex()
    return self.sequenceIndex
end

---@param index integer
function WeaponControlComponent:setSequenceIndex(index)
    self.sequenceIndex = math.max(1, index)
end

function WeaponControlComponent:isTriggerHeld()
    return self.triggerHeld
end

---@param held boolean
function WeaponControlComponent:setTriggerHeld(held)
    self.triggerHeld = held == true
end

function WeaponControlComponent:isActive()
    return self.active
end

---@param active boolean
function WeaponControlComponent:setActive(active)
    self.active = active == true
end

return WeaponControlComponent
