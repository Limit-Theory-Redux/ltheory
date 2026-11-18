local ConstructComponents = require("Modules.Constructs.Components")

local Owner = Enums.ControlOwner

-- From the strongest owner down.
local PRECEDENCE = { Owner.Manual, Owner.Claim, Owner.Orders }

--- Control ownership: decides, for every ship, which writer owns its `ControlInputComponent`
--- (manual input, then a direct-control claim, then orders), and resets the input when the owner
--- changes. The owner is recomputed on each change of what the writers want, not once per frame:
--- subscribers of one event stage have no order, so a per-frame decider could run after the new owner
--- has written. A writer that isn't the owner must not write the ship's input.
---@class ControlOwnerSystem
local ControlOwnerSystem = {}

---@param wants ControlOwnerComponent
---@return ControlOwnerId
local function strongest(wants)
    for _, owner in ipairs(PRECEDENCE) do
        if wants:wants(owner) then return owner end
    end
    return Owner.None
end

--- What the new owner starts from: no axes, the travel drive told to stop and its cap lifted.
---@param ship Entity
local function resetInput(ship)
    local input = ship:get(ConstructComponents.ControlInput)
    if not input then return end
    input:clearAxes()
    input:setDriveRequest(Enums.TravelDriveRequest.Disengage)
    input:setDriveSpeedCap(nil)
end

---@param ship Entity
---@param owner ControlOwnerId
---@param on boolean
local function setWant(ship, owner, on)
    local component = ship:get(ConstructComponents.ControlOwner)
    if not component then
        component = ship:add(ConstructComponents.ControlOwner())
    end
    component:setWants(owner, on)
    local now = strongest(component)
    if now == component:getOwner() then return end
    component:setOwner(now)
    resetInput(ship)
end

---The owner of the ship's control input; `None` for a ship nobody has asked for.
---@param ship Entity
---@return ControlOwnerId
function ControlOwnerSystem:ownerOf(ship)
    local component = ship:get(ConstructComponents.ControlOwner)
    return component and component:getOwner() or Owner.None
end

---Marks the ship as piloted by the player (or not): manual input owns it while it is.
---@param ship Entity
---@param on boolean
function ControlOwnerSystem:setManual(ship, on)
    setWant(ship, Owner.Manual, on)
end

---Marks the ship as flown by orders (or not): orders own it when nothing stronger does.
---@param ship Entity
---@param on boolean
function ControlOwnerSystem:setOrders(ship, on)
    setWant(ship, Owner.Orders, on)
end

---Takes the ship's controls for the caller, who then writes the input directly. A claimed ship has
---no orders flown, and manual input still overrides the claim.
---@param ship Entity
function ControlOwnerSystem:claim(ship)
    assert(not self:isClaimed(ship), "ControlOwnerSystem.claim: the ship is claimed already")
    setWant(ship, Owner.Claim, true)
end

---Gives back the controls taken with `claim`.
---@param ship Entity
function ControlOwnerSystem:release(ship)
    assert(self:isClaimed(ship), "ControlOwnerSystem.release: the ship is not claimed")
    setWant(ship, Owner.Claim, false)
end

---@param ship Entity
---@return boolean
function ControlOwnerSystem:isClaimed(ship)
    local component = ship:get(ConstructComponents.ControlOwner)
    return component ~= nil and component:wants(Owner.Claim)
end

return ControlOwnerSystem
