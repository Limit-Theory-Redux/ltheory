local Entity = require("Core.ECS.Entity")
local Economy = require("Modules.Economy.Components")
local NameComponent = require("Modules.Core.Components.NameComponent")
local People = require("Modules.People.Components")

---A person: their name, their own wallet, who they are (`PersonComponent`), their working life
---(`CareerComponent`) and what they remember (`MemoryComponent`). Where they serve is added when they
---are assigned (`AssignmentComponent`); their faction, when they have one (`FactionMembershipComponent`).
---@param givenName string
---@param familyName string
---@param credits number|nil In their wallet
---@param employerId integer|nil
---@return Entity
return function(givenName, familyName, credits, employerId)
    return Entity.Create(
        givenName .. " " .. familyName,
        NameComponent(givenName .. " " .. familyName),
        Economy.PlayerBankAccount(credits or 0),
        People.Person(givenName, familyName),
        People.Career(employerId),
        People.Memory()
    )
end
