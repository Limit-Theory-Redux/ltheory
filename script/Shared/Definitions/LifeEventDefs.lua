---What can happen to a person, and how it moves them: `morale` is added to their morale, `opinion` to
---their opinion of the event's subject (who paid them, who commanded the ship they lost, whose faction
---attacked them). Each event is also kept in the person's memory, so later decisions (by their
---traits, their career, their crewmates) can weigh what they lived through. Events are data: a new
---kind is an entry here, reported through `PeopleManager.experience`.

---@class LifeEventDefinition
---@field id string
---@field morale number
---@field opinion number Toward the event's subject
---@field remember boolean Whether it goes into memory (small, frequent events don't)

---@type table<string, LifeEventDefinition>
local LifeEvents = {
    Hired = { id = "Hired", morale = 0.10, opinion = 0.15, remember = true },
    Paid = { id = "Paid", morale = 0.01, opinion = 0.01, remember = false },
    JobDone = { id = "JobDone", morale = 0.005, opinion = 0.005, remember = false },
    Attacked = { id = "Attacked", morale = -0.04, opinion = -0.08, remember = true },
    ShipLost = { id = "ShipLost", morale = -0.25, opinion = -0.20, remember = true },
    Promoted = { id = "Promoted", morale = 0.15, opinion = 0.10, remember = true },
    CommandChanged = { id = "CommandChanged", morale = 0, opinion = 0, remember = true },
}

return LifeEvents
