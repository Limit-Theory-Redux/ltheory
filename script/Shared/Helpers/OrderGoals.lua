-- Where a MoveTo-like order flies to and how it counts as arrived: a point, or an approach to an
-- entity (a stand-off from its centre, or a point fixed to its body). The order system flies these goals
-- and navigation estimates price them, so both ask here.
-- Design: doc/script/nav/navigation.md

local PhysicsComponents = require("Modules.Physics.Components")
local NavConfig         = require("Config.Navigation.NavConfig")
local EntityPosition    = require("Shared.Helpers.EntityPosition")
local PlanFlight        = require("Shared.Helpers.PlanFlight")
local ApproachSpecs     = require("Shared.Helpers.ApproachSpecs")
local ApproachSpec      = require("Shared.Types.ApproachSpec")

---Where a goal order flies to, and how it counts as arrived.
---@class OrderGoal
---@field targetPos Position The goal now
---@field frameBody RigidBody|nil The body the goal rides on
---@field frameEntity Entity|nil The entity that body belongs to
---@field safeRange gameunit The stand-off distance from the target's centre; 0 for a point
---@field arrivalRadius gameunit

---@class OrderGoals
local OrderGoals = {}

---@param entity Entity
---@return RigidBody|nil
function OrderGoals.bodyOf(entity)
    local component = entity:get(PhysicsComponents.RigidBody)
    return component and component:getRigidBody() or nil
end

---@param targetPos Position
---@param safeRange gameunit
---@param arrivalRadius gameunit
---@param frame Entity|nil
---@return OrderGoal
function OrderGoals.make(targetPos, safeRange, arrivalRadius, frame)
    return {
        targetPos = targetPos,
        frameEntity = frame,
        frameBody = frame and OrderGoals.bodyOf(frame) or nil,
        safeRange = safeRange,
        arrivalRadius = arrivalRadius,
    }
end

---@param point Position
---@return OrderGoal
function OrderGoals.point(point)
    return OrderGoals.make(point, 0, NavConfig.pointArrival(), nil)
end

---The goal of an approach to an entity: a stand-off from its centre, or a point fixed to its body.
---@param target Entity
---@param approach ApproachSpec
---@return OrderGoal
function OrderGoals.approach(target, approach)
    local radius = PlanFlight.targetRadius(target)
    local targetPos = EntityPosition.Get(target)
    if approach:isPoint() then
        local body = OrderGoals.bodyOf(target)
        local rotation = body and body:getRot() or Quat.Identity()
        return OrderGoals.make(approach:pointOf(targetPos, radius, rotation), 0, NavConfig.pointArrival(), target)
    end
    local gap, radii = approach:getGap(), approach:getRadii()
    return OrderGoals.make(targetPos, PlanFlight.standOff(radius, gap, radii),
        PlanFlight.arrivalRadius(radius, gap, radii), target)
end

---The goal of a MoveTo to a target.
---@param target NavTarget An entity, or a point
---@param approach ApproachSpecId|ApproachSpecConstructor|ApproachSpec|nil For an entity only; its kind's own when nil
---@return OrderGoal
function OrderGoals.moveTo(target, approach)
    if rawtype(target) == "table" and target.isValid ~= nil then
        return OrderGoals.approach(target, ApproachSpecs.For(target, approach ~= nil and ApproachSpec.Of(approach) or nil))
    end
    return OrderGoals.point(Position(target.x, target.y, target.z))
end

return OrderGoals
