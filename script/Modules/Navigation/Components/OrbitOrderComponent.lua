local Component         = require("Core.ECS.Component")
local PhysicsComponents = require("Modules.Physics.Components")
local NavConfig         = require("Config.Navigation.NavConfig")
local NavTarget         = require("Shared.Helpers.NavTarget")

---@param v { x: number, y: number, z: number }
---@return { x: number, y: number, z: number }
local function unitOf(v)
    assert(rawtype(v) == "table" or rawtype(v) == "cdata", "OrbitOrder: the normal is a vector")
    local length = math.sqrt(v.x * v.x + v.y * v.y + v.z * v.z)
    assert(length > 0 and length < math.huge, "OrbitOrder: the normal is finite and not zero")
    return { x = v.x / length, y = v.y / length, z = v.z / length }
end

---@class OrbitOrderComponent: Component
---@overload fun(self: OrbitOrderComponent, target: Entity, altitude: number|nil, normal: Position|Vec3d|nil): OrbitOrderComponent subclass internal
---@overload fun(target: Entity, altitude: number|nil, normal: Position|Vec3d|nil): OrbitOrderComponent subclass external
--- What an Orbit order circles: an entity, at `altitude` target radii from its centre, anticlockwise
--- about a normal. The normal is the one the order names, or, when it names none, the plane the ship
--- is in when it reaches the circle, which the order system records here.
local OrbitOrderComponent = Subclass("OrbitOrderComponent", Component, function(self, target, altitude, normal)
    self:setComponentName("OrbitOrder")
    local entityId = NavTarget.Split(target, "OrbitOrder")
    assert(entityId, "OrbitOrder: the target is an entity, not a point")
    assert(target:get(PhysicsComponents.RigidBody), "OrbitOrder: the target has no rigid body")
    altitude = altitude or NavConfig.standOffRadii
    assert(rawtype(altitude) == "number" and altitude > 1,
        "OrbitOrder: the altitude is a number of target radii from its centre, more than 1")
    self._targetEntityId = entityId
    self._altitude = altitude
    self._normal = normal and unitOf(normal) or nil
    self._chosenNormal = self._normal
end)

---@return EntityId
function OrbitOrderComponent:getTargetEntityId() return self._targetEntityId end

---@return number Target radii from the target's centre
function OrbitOrderComponent:getAltitude() return self._altitude end

---@return { x: number, y: number, z: number }|nil The unit normal the order names; nil to take the ship's plane
function OrbitOrderComponent:getNormal() return self._normal end

---@return { x: number, y: number, z: number }|nil The unit normal the ship circles about; nil until it is on the circle
function OrbitOrderComponent:getChosenNormal() return self._chosenNormal end

---@param normal { x: number, y: number, z: number }|nil A unit vector; nil forgets the choice, for a ship that leaves the circle
function OrbitOrderComponent:setChosenNormal(normal) self._chosenNormal = normal end

return OrbitOrderComponent
