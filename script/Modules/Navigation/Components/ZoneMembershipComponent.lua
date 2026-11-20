local Component = require("Core.ECS.Component")

---@class ZoneMembershipComponent: Component
---@overload fun(self: ZoneMembershipComponent): ZoneMembershipComponent subclass internal
---@overload fun(): ZoneMembershipComponent subclass external
--- The zones a ship was inside at the last check, as a set of zone entity ids. It is on a ship while the
--- ship has orders, and the zone events are the changes of it.
local ZoneMembershipComponent = Subclass("ZoneMembershipComponent", Component, function(self)
    self:setComponentName("ZoneMembership")
    ---@type table<EntityId, true>
    self._inside = {}
end)

---@param zoneId EntityId
---@return boolean
function ZoneMembershipComponent:isInside(zoneId)
    return self._inside[zoneId] == true
end

---@param zoneId EntityId
---@param inside boolean
function ZoneMembershipComponent:setInside(zoneId, inside)
    self._inside[zoneId] = inside or nil
end

---@return EntityId[] The ids of the zones the ship is inside, in increasing order
function ZoneMembershipComponent:ids()
    local ids = {}
    for id in pairs(self._inside) do ids[#ids + 1] = id end
    table.sort(ids)
    return ids
end

return ZoneMembershipComponent
