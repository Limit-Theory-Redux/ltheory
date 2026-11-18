local Component         = require("Core.ECS.Component")
local PhysicsComponents = require("Modules.Physics.Components")
local ScaleConfig       = require("Config.Gen.UniverseScaleConfig")
local NavTarget         = require("Shared.Helpers.NavTarget")

---@class FollowOrderComponent: Component
---@overload fun(self: FollowOrderComponent, target: Entity, offsetM: Vec3d): FollowOrderComponent subclass internal
---@overload fun(target: Entity, offsetM: Vec3d): FollowOrderComponent subclass external
--- What a Follow order keeps station on: a leader entity, and a slot at `offsetM` (metres: right, up,
--- forward) in the leader's own axes. It also keeps the key of the plan that flies the ship to the
--- slot, which changes when the slot has moved far from where that plan ends.
local FollowOrderComponent = Subclass("FollowOrderComponent", Component, function(self, target, offsetM)
    self:setComponentName("FollowOrder")
    local entityId = NavTarget.Split(target, "FollowOrder")
    assert(entityId, "FollowOrder: the leader is an entity, not a point")
    assert(target:get(PhysicsComponents.RigidBody), "FollowOrder: the leader has no rigid body")
    assert(rawtype(offsetM) == "table" or rawtype(offsetM) == "cdata", "FollowOrder: the offset is a vector in metres")
    for _, value in ipairs({ offsetM.x, offsetM.y, offsetM.z }) do
        assert(rawtype(value) == "number" and value == value and math.abs(value) < math.huge,
            "FollowOrder: the offset is finite")
    end
    self._leaderEntityId = entityId
    self._offset = {
        x = ScaleConfig:toGameUnits(offsetM.x, "starSystem"),
        y = ScaleConfig:toGameUnits(offsetM.y, "starSystem"),
        z = ScaleConfig:toGameUnits(offsetM.z, "starSystem"),
    }
    self._slotKey = 0
    ---@type Position|nil
    self._keySlot = nil
end)

---@return EntityId
function FollowOrderComponent:getLeaderEntityId() return self._leaderEntityId end

---@return { x: number, y: number, z: number } Game units: right, up, forward in the leader's axes
function FollowOrderComponent:getOffset() return self._offset end

---The key of the plan for the slot: it rises by one when `slot` is farther than `range` from the slot
---the current key was made for, so a plan is made again only for a slot that has really moved.
---@param slot Position The slot now
---@param range gameunit
---@return integer
function FollowOrderComponent:slotKey(slot, range)
    if not self._keySlot or slot:distance(self._keySlot) > range then
        self._keySlot = slot
        self._slotKey = self._slotKey + 1
    end
    return self._slotKey
end

return FollowOrderComponent
