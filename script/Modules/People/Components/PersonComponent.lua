local Component = require("Core.ECS.Component")

---A person (an NPC): their name, their traits and their morale. Traits are values by trait id, for
---the personality that decides how they choose; none are read yet. Morale, 0 to 1, is moved by
---what happens to them (`LifeEventDefs`).
---@class PersonComponent: Component
---@field givenName string
---@field familyName string
---@field traits table<string, number>
---@field morale number
---@overload fun(self: PersonComponent, givenName: string, familyName: string, traits: table<string, number>|nil): PersonComponent subclass internal
---@overload fun(givenName: string, familyName: string, traits: table<string, number>|nil): PersonComponent subclass external
local PersonComponent = Subclass("PersonComponent", Component, function(self, givenName, familyName, traits)
    self:setComponentName("Person")

    self.givenName = givenName
    self.familyName = familyName
    self.traits = traits or {}
    self.morale = 0.6
end)

---@return string
function PersonComponent:getFullName()
    return self.givenName .. " " .. self.familyName
end

---@param trait string
---@return number|nil
function PersonComponent:getTrait(trait)
    return self.traits[trait]
end

---@param trait string
---@param value number
function PersonComponent:setTrait(trait, value)
    self.traits[trait] = value
end

---@return number
function PersonComponent:getMorale()
    return self.morale
end

---@param morale number
function PersonComponent:setMorale(morale)
    self.morale = math.max(0, math.min(1, morale))
end

return PersonComponent
