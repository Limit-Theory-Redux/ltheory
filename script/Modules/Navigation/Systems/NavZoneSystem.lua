local Registry          = require("Core.ECS.Registry")
local SpatialComponents = require("Modules.Spatial.Components")
local Components        = require("Modules.Navigation.Components")
local OrderGoals        = require("Shared.Helpers.OrderGoals")
local ZoneSphere        = require("Shared.Helpers.ZoneSphere")

local ZONE_EVENTS = { entered = "NavZoneEntered", left = "NavZoneLeft" }

---The zones that are spheres, in id order so that events come in the same order for the same scene.
---@return table<integer, { zone: Entity, radius: gameunit }> zones
---@return table<EntityId, true> live The ids of those zones
local function sphereZones()
    local zones, live = {}, {}
    for zone in Registry:view(SpatialComponents.Zone) do
        local radius = ZoneSphere.radius(zone)
        if radius then zones[#zones + 1] = { zone = zone, radius = radius } end
    end
    table.sort(zones, function(a, b) return a.zone.id < b.zone.id end)
    for _, entry in ipairs(zones) do live[entry.zone.id] = true end
    return zones, live
end

---@param event string
---@param ship Entity
---@param zoneId EntityId
local function announce(event, ship, zoneId)
    EventBus:dispatch(Event[event], { ship = ship.id, zone = zoneId })
end

--- Navigation zone system: tells, with events, when a ship that has orders enters or leaves a zone (a
--- sphere zone entity). A ship's first check announces the zones it is already in. A ship whose
--- orders have all ended is no longer watched, with no event for the zones it was in. A zone that
--- is destroyed is left. Design: doc/script/nav/navigation.md
---@class NavZoneSystem
---@overload fun(): NavZoneSystem
local NavZoneSystem = Class("NavZoneSystem", function(self)
    local names = {}
    for _, name in pairs(ZONE_EVENTS) do names[#names + 1] = name end
    Event.AddEvents(names)
    for _, name in ipairs(names) do
        EventBus:register(Event[name], name, FrameStage.PostSim)
    end
end)

---Checks one ship against the zones, announcing what changed since its last check.
---@param ship Entity
---@param rb RigidBody
---@param zones table<integer, { zone: Entity, radius: gameunit }>
---@param live table<EntityId, true>
local function checkShip(ship, rb, zones, live)
    local membership = ship:get(Components.ZoneMembership)
    if not membership then
        membership = Components.ZoneMembership()
        ship:add(membership)
    end
    local position = rb:getPos()
    for _, entry in ipairs(zones) do
        local id = entry.zone.id
        local inside = ZoneSphere.contains(entry.zone, entry.radius, position)
        if inside ~= membership:isInside(id) then
            membership:setInside(id, inside)
            announce(inside and ZONE_EVENTS.entered or ZONE_EVENTS.left, ship, id)
        end
    end
    for _, id in ipairs(membership:ids()) do
        if not live[id] then
            membership:setInside(id, false)
            announce(ZONE_EVENTS.left, ship, id)
        end
    end
end

---Announces the zones entered and left since the last update by the ships that have orders.
function NavZoneSystem:update()
    local zones, live = sphereZones()
    for ship, queue in Registry:view(Components.OrderQueue) do
        local rb = OrderGoals.bodyOf(ship)
        if queue:count() > 0 and rb then
            checkShip(ship, rb, zones, live)
        elseif ship:get(Components.ZoneMembership) then
            ship:remove(Components.ZoneMembership)
        end
    end
end

return NavZoneSystem
