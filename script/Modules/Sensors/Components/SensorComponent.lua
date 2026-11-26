local Component = require("Core.ECS.Component")
local SensorModel = require("Modules.Sensors.SensorModel")

--- A ship's sensor system: its dials and what it has made of its
--- surroundings. The state (`SensorState`) holds the dials, the objects as
--- this sensor sees them, pulse echoes, scan reads and its own heat; the
--- readings are the last update's `SensorModel.evaluate`.
---
--- The objects are views of the `SensorSignature`s around it: tables with this
--- sensor's own `pos` (metres from it) and `entity`, reading everything else
--- through from the signature.
---@class SensorComponent: Component
---@overload fun(dials: SensorDials): SensorComponent
local SensorComponent = Subclass("SensorComponent", Component, function(self, dials)
    self:setComponentName("Sensor")
    self.state = SensorModel.newState(SensorModel.copyDials(dials))
    ---@type table<SensorSignatureComponent, SensorObject>
    self.views = {}
    ---@type table<SensorObject, SensorResult>
    self.readings = {}
    self.pulseRequested = false
end)

---@return SensorState
function SensorComponent:getState()
    return self.state
end

---@return SensorDials
function SensorComponent:getDials()
    return self.state.dials
end

--- Replaces the dials (a copy of them).
---@param dials SensorDials
function SensorComponent:setDials(dials)
    self.state.dials = SensorModel.copyDials(dials)
end

---@return table<SensorObject, SensorResult>
function SensorComponent:getReadings()
    return self.readings
end

---@param readings table<SensorObject, SensorResult>
function SensorComponent:setReadings(readings)
    self.readings = readings
end

--- The objects this sensor sees, in a fixed order.
---@return SensorObject[]
function SensorComponent:getObjects()
    return self.state.objects
end

---@param signature SensorSignatureComponent
---@return SensorObject|nil
function SensorComponent:getView(signature)
    return self.views[signature]
end

---@param signature SensorSignatureComponent
---@param view SensorObject|nil
function SensorComponent:setView(signature, view)
    self.views[signature] = view
end

---@return table<SensorSignatureComponent, SensorObject>
function SensorComponent:getViews()
    return self.views
end

---@return SensorObject|nil
function SensorComponent:getTarget()
    return self.state.target
end

---@param target SensorObject|nil
function SensorComponent:setTarget(target)
    self.state.target = target
end

---@return boolean
function SensorComponent:isScanning()
    return self.state.scanning
end

---@param scanning boolean
function SensorComponent:setScanning(scanning)
    self.state.scanning = scanning == true
end

--- Asks for a pulse on the next update.
function SensorComponent:requestPulse()
    self.pulseRequested = true
end

---@return boolean
function SensorComponent:isPulseRequested()
    return self.pulseRequested
end

function SensorComponent:clearPulseRequest()
    self.pulseRequested = false
end

---@return string
function SensorComponent:getSurroundings()
    return self.state.surroundings
end

---@param surroundings string open, nebula, debris or radiation
function SensorComponent:setSurroundings(surroundings)
    self.state.surroundings = surroundings
end

---@return string
function SensorComponent:getResolutionRule()
    return self.state.resolutionRule
end

---@param rule string background or dim
function SensorComponent:setResolutionRule(rule)
    self.state.resolutionRule = rule
end

---@return boolean
function SensorComponent:isMerging()
    return self.state.merge
end

---@param merge boolean
function SensorComponent:setMerging(merge)
    self.state.merge = merge == true
end

--- Forgets everything this sensor learned: echoes, reads, scan progress, its
--- memory of contacts, its heat and its target.
function SensorComponent:reset()
    local state = self.state
    state.echoes, state.reads, state.progress, state.memory = {}, {}, {}, {}
    state.target, state.scanning, state.heat = nil, false, 0
    state.pulseAge, state.pulseHeardBy = math.huge, 0
    self.pulseRequested = false
end

return SensorComponent
