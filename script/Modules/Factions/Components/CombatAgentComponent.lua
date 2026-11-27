local Component = require("Core.ECS.Component")

---A ship on a fighting job (`Enums.CombatTask`): its patrol points, what it engages, and its navigation
---order. The agent system adds it for the job's length; `CombatAgentSystem` flies it.
---@class CombatAgentComponent: Component
---@field task CombatTask
---@field patrol Position[] The points it patrols when it engages nothing
---@field postId integer|nil The station a patrol guards: the ships guarding one fly as a wing
---@field target integer|nil Entity id of the ship it engages
---@field order Entity|nil Its navigation order
---@field orderKind string|nil "patrol" or "follow"
---@field closeSince number|nil Simulated seconds it has been within raid range of its target since
---@field raids integer Cargo taken by raids, in units
---@field activity string What it does now, for display
---@overload fun(self: CombatAgentComponent, task: CombatTask, patrol: Position[], postId: integer|nil): CombatAgentComponent subclass internal
---@overload fun(task: CombatTask, patrol: Position[], postId: integer|nil): CombatAgentComponent subclass external
local CombatAgentComponent = Subclass("CombatAgentComponent", Component, function(self, task, patrol, postId)
    self:setComponentName("CombatAgent")

    self.task = task
    self.patrol = patrol or {}
    self.postId = postId
    self.target = nil
    self.order = nil
    self.orderKind = nil
    self.closeSince = nil
    self.raids = 0
    self.activity = "idle"
end)

---@return CombatTask
function CombatAgentComponent:getTask()
    return self.task
end

---@return string
function CombatAgentComponent:getActivity()
    return self.activity
end

---@return integer|nil targetId
function CombatAgentComponent:getTarget()
    return self.target
end

return CombatAgentComponent
