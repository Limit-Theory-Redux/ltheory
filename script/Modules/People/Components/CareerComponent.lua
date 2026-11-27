local Component = require("Core.ECS.Component")

---@class CareerEntry
---@field postId integer Where they served
---@field role string A `CrewRoleDefs` id
---@field employerId integer|nil Who paid them; nil while self-employed
---@field from number Simulated seconds
---@field to number|nil Simulated seconds; nil while it goes on

---A person's working life: who employs them (an entity with an account, such as a faction's leader;
---none for a self-employed owner), the posts they held, what they have done (experience, by activity:
---"trade", "mining", "patrol"...) and what they were paid.
---@class CareerComponent: Component
---@field employerId integer|nil
---@field history CareerEntry[]
---@field experience table<string, number>
---@field earned number
---@overload fun(self: CareerComponent, employerId: integer|nil): CareerComponent subclass internal
---@overload fun(employerId: integer|nil): CareerComponent subclass external
local CareerComponent = Subclass("CareerComponent", Component, function(self, employerId)
    self:setComponentName("Career")

    self.employerId = employerId
    self.history = {}
    self.experience = {}
    self.earned = 0
end)

---@return integer|nil
function CareerComponent:getEmployer()
    return self.employerId
end

---@param employerId integer|nil
function CareerComponent:setEmployer(employerId)
    self.employerId = employerId
end

---@return CareerEntry[]
function CareerComponent:getHistory()
    return self.history
end

---@return CareerEntry|nil
function CareerComponent:getCurrent()
    local last = self.history[#self.history]
    return last and not last.to and last or nil
end

---@param activity string
---@return number
function CareerComponent:getExperience(activity)
    return self.experience[activity] or 0
end

---@param activity string
---@param amount number
function CareerComponent:addExperience(activity, amount)
    self.experience[activity] = (self.experience[activity] or 0) + amount
end

---@return number
function CareerComponent:getEarned()
    return self.earned
end

---@param credits number
function CareerComponent:addEarned(credits)
    self.earned = self.earned + credits
end

return CareerComponent
