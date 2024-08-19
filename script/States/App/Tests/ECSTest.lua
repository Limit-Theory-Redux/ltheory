---@type Entity
local Entity = require("Entity")
---@type NameComponent
local NameComponent = require("GameObjects.Components.EntityName")
---@type TypeComponent
local TypeComponent = require("GameObjects.Components.EntityType")

EntityComponentSystemTest = require('States.Application')

---@diagnostic disable-next-line: duplicate-set-field
function EntityComponentSystemTest:onInit()
    self:printEntityComponents()
end

---@class TestEntity: Entity
local TestEntity = Subclass(Entity, function(self)
    self:addComponent(NameComponent("TestEntity"))
    self:addComponent(TypeComponent(Enums.EntityType.Player))
end)

function EntityComponentSystemTest:printEntityComponents()
    for index, component in self:iterComponents() do
        print(index, component.name, component)
    end
end

return EntityComponentSystemTest
