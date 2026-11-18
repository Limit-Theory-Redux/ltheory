local Component = require("Core.ECS.Component")

---@class NavWorldComponent: Component
---@overload fun(self: NavWorldComponent, physics: Physics): NavWorldComponent subclass internal
---@overload fun(physics: Physics): NavWorldComponent subclass external
--- The navigation world that steers every ship that has an order, the ship each of its slots
--- belongs to, and the buffers its results are copied into. The buffers hold at least one entry per
--- slot and grow with the slots.
local NavWorldComponent = Subclass("NavWorldComponent", Component, function(self, physics)
    self:setComponentName("NavWorld")
    self._nav = NavWorld.Create(physics)
    self._physics = physics
    self._capacity = 0
    self._axes = nil
    self._arrived = nil
    ---@type table<integer, EntityId> Slot to the id of the ship whose agent it is
    self._shipOfSlot = {}
end)

---@return NavWorld
function NavWorldComponent:getNav() return self._nav end

---@return Physics The physics world the navigation world reads its bodies from
function NavWorldComponent:getPhysics() return self._physics end

---@return ffi.cdata* double[], `NavWorld.AxisCount()` per slot
function NavWorldComponent:getAxes() return self._axes end

---@return ffi.cdata* bool[], one per slot
function NavWorldComponent:getArrived() return self._arrived end

---Makes the buffers hold at least `slots` slots.
---@param slots integer
function NavWorldComponent:reserve(slots)
    if slots <= self._capacity then return end
    self._capacity = slots
    self._axes = ffi.new("double[?]", slots * NavWorld.AxisCount())
    self._arrived = ffi.new("bool[?]", slots)
end

---Records that a slot is a ship's agent.
---@param slot integer
---@param shipId EntityId
function NavWorldComponent:assign(slot, shipId)
    self._shipOfSlot[slot] = shipId
end

---Forgets a slot, which is free to be handed to another agent.
---@param slot integer
function NavWorldComponent:unassign(slot)
    self._shipOfSlot[slot] = nil
end

---@return fun(): integer|nil, EntityId|nil Iterates the slots in use and their ships' ids
function NavWorldComponent:slots()
    return pairs(self._shipOfSlot)
end

---@return boolean Whether any agent is in the world
function NavWorldComponent:hasAgents()
    return next(self._shipOfSlot) ~= nil
end

return NavWorldComponent
