local Component = require("Core.ECS.Component")

---A ship's docking at a station, from its request until it has left the port: which port, which
---phase (`Enums.DockingPhase`), and the glide in or out. `DockingSystem` adds, runs and removes it.
---@class DockingComponent: Component
---@field station integer Entity id of the station
---@field port integer Index of the port in the station's `DockingPortsComponent`
---@field phase DockingPhase
---@field order Entity|nil The navigation order that flies the ship to the approach point
---@field progress number 0 to 1 through the glide in or out
---@field duration number Seconds the glide takes
---@field from table|nil Where the glide starts: `{ right, up, forward }` in game units along the station's axes
---@field fromRot Quat|nil The ship's orientation, relative to the station's, where the glide starts
---@overload fun(self: DockingComponent, station: integer, port: integer): DockingComponent subclass internal
---@overload fun(station: integer, port: integer): DockingComponent subclass external
local DockingComponent = Subclass("DockingComponent", Component, function(self, station, port)
    self:setComponentName("Docking")

    self.station = station
    self.port = port
    self.phase = Enums.DockingPhase.Approaching
    self.order = nil
    self.progress = 0
    self.duration = 0
end)

---@return integer stationId
function DockingComponent:getStation()
    return self.station
end

---@return integer portIndex
function DockingComponent:getPort()
    return self.port
end

---@return DockingPhase
function DockingComponent:getPhase()
    return self.phase
end

---@param phase DockingPhase
function DockingComponent:setPhase(phase)
    self.phase = phase
    self.progress = 0
end

return DockingComponent
