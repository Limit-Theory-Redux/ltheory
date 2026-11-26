local Component         = require("Core.ECS.Component")
local PhysicsComponents = require("Modules.Physics.Components")
local ScaleConfig       = require("Config.Gen.UniverseScaleConfig")
local NavTarget         = require("Shared.Helpers.NavTarget")
local GoalDrift         = require("Shared.Helpers.GoalDrift")

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
    self._drift = GoalDrift.new()
end)

---@return EntityId
function FollowOrderComponent:getLeaderEntityId() return self._leaderEntityId end

---@return { x: number, y: number, z: number } Game units: right, up, forward in the leader's axes
function FollowOrderComponent:getOffset() return self._offset end

---@return GoalDriftState The drift of the slot, whose key names the plan made for it (`GoalDrift.key`)
function FollowOrderComponent:getDrift() return self._drift end

return FollowOrderComponent
