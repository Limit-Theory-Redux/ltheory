local Component = require("Core.ECS.Component")

---@class EconomyJob
---@field itemId integer
---@field quantity integer What the job plans to carry
---@field source integer|nil Entity id of the station it buys at, or the asteroid it mines
---@field destination integer Entity id of the station it sells at
---@field buyPrice integer|nil The price it plans to pay per unit
---@field sellPrice integer The price it plans to get per unit
---@field claims table[] The market and asteroid quantities it holds against other agents

---An AI ship that earns its living in the economy, as a trader or a miner (`Enums.EconomyAgentRole`).
---It trades with its own inventory and its own account or its owner's. `EconomyAgentSystem` picks
---its jobs from the markets and flies them.
---@class EconomyAgentComponent: Component
---@field role EconomyAgentRole
---@field state EconomyAgentState
---@field job EconomyJob|nil
---@field order Entity|nil The navigation order the agent flies, outside docking
---@field waitUntil number|nil Simulated seconds, while `Waiting` or docked
---@field resume boolean Whether the job goes on after an undocking or a wait
---@field trips integer Jobs finished
---@field earned number Credits earned by sales, all told
---@field spent number Credits spent on purchases, all told
---@field activity string What the agent does now, for display
---@overload fun(self: EconomyAgentComponent, role: EconomyAgentRole): EconomyAgentComponent subclass internal
---@overload fun(role: EconomyAgentRole): EconomyAgentComponent subclass external
local EconomyAgentComponent = Subclass("EconomyAgentComponent", Component, function(self, role)
    self:setComponentName("EconomyAgent")

    self.role = role
    self.state = Enums.EconomyAgentState.Idle
    self.job = nil
    self.order = nil
    self.waitUntil = nil
    self.resume = false
    self.trips = 0
    self.earned = 0
    self.spent = 0
    self.activity = "idle"
end)

---@return EconomyAgentRole
function EconomyAgentComponent:getRole()
    return self.role
end

---@return EconomyAgentState
function EconomyAgentComponent:getState()
    return self.state
end

---@param state EconomyAgentState
function EconomyAgentComponent:setState(state)
    self.state = state
end

---@return EconomyJob|nil
function EconomyAgentComponent:getJob()
    return self.job
end

---@return string
function EconomyAgentComponent:getActivity()
    return self.activity
end

---@return integer trips, number earned, number spent
function EconomyAgentComponent:getStats()
    return self.trips, self.earned, self.spent
end

return EconomyAgentComponent
