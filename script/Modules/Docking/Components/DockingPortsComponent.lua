local Component = require("Core.ECS.Component")

---@class DockingPort
---@field offsetRadii ApproachOffset Where a docked ship sits, in station radii along the station's axes
---@field approachRadii ApproachOffset Where a ship waits before it glides in, outside the port
---@field occupant integer|nil The entity id of the ship that holds the port, from its reservation on

---The docking ports of a station: places around its hull, each held by one ship at a time from
---when the ship asks to dock until it has left. `DockingSystem` assigns them.
---@class DockingPortsComponent: Component
---@field ports DockingPort[]
---@overload fun(self: DockingPortsComponent, count: integer|nil, portRadii: number|nil, approachRadii: number|nil): DockingPortsComponent subclass internal
---@overload fun(count: integer|nil, portRadii: number|nil, approachRadii: number|nil): DockingPortsComponent subclass external
local DockingPortsComponent = Subclass("DockingPortsComponent", Component, function(self, count, portRadii, approachRadii)
    self:setComponentName("DockingPorts")

    self:init(count or 4, portRadii or 1.05, approachRadii or 1.6)
end)

---Spreads the ports evenly around the station's equator (its right-forward plane).
---@param count integer
---@param portRadii number The ports' distance from the centre, in station radii
---@param approachRadii number The approach points' distance from the centre, in station radii
function DockingPortsComponent:init(count, portRadii, approachRadii)
    self.ports = {}
    for i = 1, count do
        local angle = (i - 1) * 2 * math.pi / count
        local right, forward = math.sin(angle), math.cos(angle)
        self.ports[i] = {
            offsetRadii = { right = right * portRadii, up = 0, forward = forward * portRadii },
            approachRadii = { right = right * approachRadii, up = 0, forward = forward * approachRadii },
            occupant = nil,
        }
    end
end

---@return DockingPort[]
function DockingPortsComponent:getPorts()
    return self.ports
end

---@param index integer
---@return DockingPort|nil
function DockingPortsComponent:getPort(index)
    return self.ports[index]
end

return DockingPortsComponent
