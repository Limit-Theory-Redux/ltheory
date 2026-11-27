---The speed caps of the navigation world's agents, by slot, from two sources: the cap a flight sets
---for a while (a plan's corner, `PlanFollower`) and the limit a ship keeps for as long as it has
---one (`Nav.setMaxSpeed`, a formation's leader held to its wing's pace). The agent is capped at the
---lower of the two. Writers of an agent's cap go through here, so neither lifts the other.
local NavSpeedCaps = {}

local caps, limits = {}, {}

---@param nav NavWorld
---@param slot integer
local function apply(nav, slot)
    nav:setSpeedCap(slot, math.min(caps[slot] or math.huge, limits[slot] or math.huge))
end

---Sets the flight's cap of the agent (math.huge lifts it).
---@param nav NavWorld
---@param slot integer
---@param cap number Game units per second
function NavSpeedCaps.cap(nav, slot, cap)
    caps[slot] = cap ~= math.huge and cap or nil
    apply(nav, slot)
end

---Sets the ship's own limit on the agent (nil lifts it).
---@param nav NavWorld
---@param slot integer
---@param limit number|nil Game units per second
function NavSpeedCaps.limit(nav, slot, limit)
    limits[slot] = limit
    apply(nav, slot)
end

---Forgets a slot, when its agent is removed (the slot may be handed to another agent).
---@param slot integer
function NavSpeedCaps.forget(slot)
    caps[slot], limits[slot] = nil, nil
end

---Forgets every slot, for a new navigation world.
function NavSpeedCaps.reset()
    caps, limits = {}, {}
end

return NavSpeedCaps
