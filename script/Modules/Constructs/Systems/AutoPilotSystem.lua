local Registry            = require("Core.ECS.Registry")
local ConstructComponents = require("Modules.Constructs.Components")
local ControlOwnerSystem  = require("Modules.Constructs.Systems.ControlOwnerSystem")
local Nav                 = require("Modules.Navigation")
local EntityPosition      = require("Shared.Helpers.EntityPosition")
local FrameOffset         = require("Shared.Helpers.FrameOffset")

local Status = Enums.NavOrderStatus

--- AutoPilot System: the player's autopilot is a MoveTo order given to the player's ship, at
--- `Urgent` priority. The order system flies it; this system issues it, ends it, and hands the
--- controls between the player (manual input) and the order. Taking the controls back by hand
--- cancels the order. Design: doc/script/nav/navigation.md
---@class AutoPilotSystem
local AutoPilotSystem = {}

-- Defined below, where the hand-back it uses is.
local settle

---@param target NavTarget
---@return boolean
local function isEntity(target)
    return rawtype(target) == "table" and target.isValid ~= nil
end

---The order of the ship's autopilot, if it has one.
---@param shipEntity Entity
---@return Entity|nil order
---@return AutoPilotComponent|nil ap
local function orderOf(shipEntity)
    local ap = shipEntity:get(ConstructComponents.AutoPilot)
    return ap and ap:getOrder() or nil, ap
end

--- Engage the autopilot: fly to an entity (to its stand-off), or to a point. An engagement while
--- the autopilot flies gives the order the new target.
---@param shipEntity Entity The player's ship, with an `AutoPilotComponent`
---@param target NavTarget
---@param frame Entity|nil For a point: an entity with a rigid body that the point keeps its offset from
function AutoPilotSystem:engage(shipEntity, target, frame)
    local ap = shipEntity:get(ConstructComponents.AutoPilot)
    if not ap then return end
    settle(shipEntity, ap)

    local targetEntity = isEntity(target) and target or nil
    local order = ap:getOrder()
    if order then
        Nav.retarget(order, target, frame)
    else
        order = Nav.order(shipEntity, Nav.Orders.MoveTo {
            target = target, frame = frame, priority = Nav.Priority.Urgent,
        })
        -- The order flies the ship once the player's manual input has let go of it.
        ControlOwnerSystem:setManual(shipEntity, false)
        Log.Info("AutoPilot: engaged to %s", targetEntity and tostring(targetEntity) or "position")
    end
    ap:setEngagement(order, targetEntity)
end

--- Disengage the autopilot: the order is cancelled and the player has the controls at once. The
--- order system ends the order at its next update.
---@param shipEntity Entity
function AutoPilotSystem:disengage(shipEntity)
    local order, ap = orderOf(shipEntity)
    if not order then return end
    Log.Info("AutoPilot: disengaged")
    Nav.cancel(order)
    self:handBack(shipEntity, ap)
end

--- Settle the autopilot of a ship: an order that has ended (arrived or failed) gives the player the
--- controls back, and every ended order is released.
---@param shipEntity Entity
---@param ap AutoPilotComponent
function settle(shipEntity, ap)
    local order = ap:getOrder()
    if order and order:get(Nav.Components.OrderStatus):hasEnded() then
        if Nav.status(order) == Status.Done then Log.Info("AutoPilot: arrived at destination") end
        AutoPilotSystem:handBack(shipEntity, ap)
    end
    local ending = ap:getEnding()
    for i = #ending, 1, -1 do
        if ending[i]:get(Nav.Components.OrderStatus):hasEnded() then
            Nav.release(ending[i])
            table.remove(ending, i)
        end
    end
end

--- Settle the autopilot of every ship, once per frame.
function AutoPilotSystem:update()
    for shipEntity, ap in Registry:iterEntities(ConstructComponents.AutoPilot) do
        settle(shipEntity, ap)
    end
end

--- The player has the controls: the engagement is forgotten (its order is released once it has
--- ended), and the ownership system resets the input (no axes left over, the travel drive stopped
--- and its cap lifted).
---@param shipEntity Entity
---@param ap AutoPilotComponent
function AutoPilotSystem:handBack(shipEntity, ap)
    ap:endEngagement()
    ControlOwnerSystem:setManual(shipEntity, true)
end

--- Is autopilot currently active for this ship?
---@param shipEntity Entity
---@return boolean
function AutoPilotSystem:isActive(shipEntity)
    local ap = shipEntity:get(ConstructComponents.AutoPilot)
    return ap ~= nil and ap:isActive()
end

--- Is the ship's path still being planned? The ship steers straight to the target until the plan has come.
---@param shipEntity Entity
---@return boolean
function AutoPilotSystem:isPlanning(shipEntity)
    local order = orderOf(shipEntity)
    return order ~= nil and Nav.status(order) == Status.Planning
end

--- Get target name for HUD display
---@param shipEntity Entity
---@return string
function AutoPilotSystem:getTargetName(shipEntity)
    local ap = shipEntity:get(ConstructComponents.AutoPilot)
    return ap and ap:getTargetName() or ""
end

--- How far the flight is from its end: the distance left to the order's goal, the ETA and the next
--- waypoint (see `OrderProgressComponent`).
---@param shipEntity Entity
---@return OrderProgressComponent|nil progress Nil while the autopilot is off
function AutoPilotSystem:getProgress(shipEntity)
    local order = orderOf(shipEntity)
    return order and Nav.progress(order) or nil
end

--- Where the autopilot's target is now: the entity's position, or the point (which moves with its
--- frame entity, if it has one).
---@param shipEntity Entity
---@return Position|nil position Nil while the autopilot is off, or when the frame entity is gone
function AutoPilotSystem:getTargetPosition(shipEntity)
    local order, ap = orderOf(shipEntity)
    if not order then return nil end
    local targetEntity = ap:getTargetEntity()
    if targetEntity then return EntityPosition.Get(targetEntity) end
    local moveTo = order:get(Nav.Components.MoveToOrder)
    local frameId = moveTo:getFrameEntityId()
    if not frameId then return moveTo:getTargetPos() end
    local frame = Registry:getEntityById(frameId)
    return frame and FrameOffset.point(frame, moveTo:getFrameOffset()) or nil
end

return AutoPilotSystem
