local Component = require("Core.ECS.Component")

---@class EconomyJob
---@field kind string "sale", "trade", "mining", "patrol", "raid" or "park"
---@field stage string What the job is at: "buy", "mine", "sell", "fight" or "park"
---@field itemId integer|nil
---@field quantity integer|nil What the job plans to carry
---@field source integer|nil Entity id of the station it buys at, or the asteroid it mines
---@field destination integer|nil Entity id of the station it sells at, guards or parks at
---@field buyPrice integer|nil The price it plans to pay per unit
---@field sellPrice integer|nil The price it plans to get per unit
---@field payout number|nil What a patrol pays when it ends
---@field duration number|nil Seconds a patrol or a raid lasts
---@field score number Profit per second, which the AI chose the job by
---@field claims table[] The market, asteroid and patrol shares it holds against other agents

---An AI ship that earns its living in the economy. It has no fixed role: its equipment (its hold, a
---mining laser, weapons) decides which jobs it can do, and `EconomyAgentSystem` chooses among them
---by profit per second each time it is free. `kind` is the ship's kind, for display.
---@class EconomyAgentComponent: Component
---@field kind string A key of `ShipKindDefs`, or any name
---@field state EconomyAgentState
---@field job EconomyJob|nil
---@field order Entity|nil The navigation order the agent flies, outside docking and fights
---@field waitUntil number|nil Simulated seconds, while waiting, docked or fighting
---@field resume boolean Whether the job goes on after an undocking or a wait
---@field trips integer Jobs finished
---@field earned number Credits earned, all told
---@field spent number Credits spent on purchases, all told
---@field earnedBy table<string, number> Credits earned per job kind
---@field activity string What the agent does now, for display
---@overload fun(self: EconomyAgentComponent, kind: string|nil): EconomyAgentComponent subclass internal
---@overload fun(kind: string|nil): EconomyAgentComponent subclass external
local EconomyAgentComponent = Subclass("EconomyAgentComponent", Component, function(self, kind)
    self:setComponentName("EconomyAgent")

    self.kind = kind or "Ship"
    self.state = Enums.EconomyAgentState.Idle
    self.job = nil
    self.order = nil
    self.waitUntil = nil
    self.resume = false
    self.trips = 0
    self.earned = 0
    self.spent = 0
    self.earnedBy = {}
    self.activity = "idle"
end)

---@return string kind
function EconomyAgentComponent:getKind()
    return self.kind
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

---Books credits the agent earned on a job of `kind`.
---@param kind string
---@param credits number
function EconomyAgentComponent:addEarnings(kind, credits)
    self.earned = self.earned + credits
    self.earnedBy[kind] = (self.earnedBy[kind] or 0) + credits
end

---@return integer trips, number earned, number spent
function EconomyAgentComponent:getStats()
    return self.trips, self.earned, self.spent
end

return EconomyAgentComponent
