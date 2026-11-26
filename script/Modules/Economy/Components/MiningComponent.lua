local Component = require("Core.ECS.Component")

---A ship mining an asteroid, from when it reaches the rock until its run ends: the rock, the circle
---it flies around it, the laser and what it has taken. `MiningSystem` adds, runs and removes it.
---@class MiningComponent: Component
---@field rock integer Entity id of the asteroid
---@field itemId integer
---@field quantity integer The most the run takes
---@field untilTime number Simulated seconds the run ends at, at the latest
---@field taken integer Units taken in this run
---@field extracted number The part of a unit extracted since the last whole one
---@field radius number The circle's radius, in game units
---@field angle number Where on the circle the ship is, in radians
---@field height number The circle's height over the rock's centre, in game units
---@field settle number 0 to 1 through the glide from where the ship arrived onto the circle
---@field from Position Where the ship arrived
---@field fromRot Quat The ship's orientation when it arrived
---@field beam Entity|nil The laser, while it is on
---@field lasing boolean Whether the laser is on an ore spot this frame
---@overload fun(self: MiningComponent, rock: integer, itemId: integer, quantity: integer, untilTime: number): MiningComponent subclass internal
---@overload fun(rock: integer, itemId: integer, quantity: integer, untilTime: number): MiningComponent subclass external
local MiningComponent = Subclass("MiningComponent", Component, function(self, rock, itemId, quantity, untilTime)
    self:setComponentName("EconomyMining")

    self.rock = rock
    self.itemId = itemId
    self.quantity = quantity
    self.untilTime = untilTime
    self.taken = 0
    self.extracted = 0
    self.settle = 0
    self.lasing = false
    self.done = false
end)

---@return integer units Taken in this run
function MiningComponent:getTaken()
    return self.taken
end

---@return boolean done Whether the run has ended (hold full, rock empty, time up or rock gone)
function MiningComponent:isDone()
    return self.done
end

---@return boolean
function MiningComponent:isLasing()
    return self.lasing
end

return MiningComponent
