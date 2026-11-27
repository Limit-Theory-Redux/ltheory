local Component = require("Core.ECS.Component")

---The people who man a post (a ship, later a station): each member's role (`CrewRoleDefs`), and who
---commands. The commander carries the post's decision-making (its AI components), so a change of
---command, by promotion, death or mutiny, hands the decisions to someone else.
---@class CrewComponent: Component
---@field members table<integer, string> Person entity id -> role id
---@field commanderId integer|nil
---@overload fun(self: CrewComponent): CrewComponent subclass internal
---@overload fun(): CrewComponent subclass external
local CrewComponent = Subclass("CrewComponent", Component, function(self)
    self:setComponentName("Crew")

    self.members = {}
    self.commanderId = nil
end)

---@return table<integer, string>
function CrewComponent:getMembers()
    return self.members
end

---@param personId integer
---@param role string|nil Nil takes them off the crew
function CrewComponent:setMember(personId, role)
    self.members[personId] = role
end

---@return integer|nil
function CrewComponent:getCommander()
    return self.commanderId
end

---@param personId integer|nil
function CrewComponent:setCommander(personId)
    self.commanderId = personId
end

---@return integer
function CrewComponent:getSize()
    local n = 0
    for _ in pairs(self.members) do n = n + 1 end
    return n
end

return CrewComponent
