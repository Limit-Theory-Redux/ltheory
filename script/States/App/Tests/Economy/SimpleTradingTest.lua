local Application        = require('States.Application')

---@class SimpleTradingTest: Application
local SimpleTradingTest  = Subclass("SimpleTradingTest", Application)

local ChildrenComponent  = require("Modules.Core.Components.ChildrenComponent")
local TransformComponent = require("Modules.Physics.Components.TransformComponent")
local ItemComponent      = require("Modules.Economy.Components.ItemComponent")
local QuantityComponent  = require("Modules.Economy.Components.QuantityComponent")

local Registry           = require("Core.ECS.Registry")
require("Shared.Definitions.ItemDefs")
local Items           = require("Shared.Registries.Items")
local UniverseManager = require("Modules.CelestialObjects.Managers").UniverseManager
local Rulesets        = require("Config.Gen.Rulesets")


local seed = 1

---@diagnostic disable-next-line: duplicate-set-field
function SimpleTradingTest:onInit()
    local scenario = Rulesets.StandardBinarySolarSystem
    local universe = UniverseManager:createUniverse(scenario, seed)

    --* temporary test of ECS UniverseManager 24.Oct.2025 @IllustrisJack
    if not universe then
        return
    end
    Registry:printHierarchy(universe)

    for starSystem in universe:get(ChildrenComponent):iterChildren() do
        ---@cast starSystem Entity
        for child in starSystem:get(ChildrenComponent):iterChildren() do
            ---@cast child Entity
            Log.Info("%s Position: %s", child, child:get(TransformComponent):getPosition())

            local itemCmp = child:get(ItemComponent)
            local quantityCmp = child:get(QuantityComponent)

            if itemCmp and quantityCmp then
                Log.Info("%s Item: %s, Quantity: %s", child, Items:getDefinition(itemCmp:getItem()).name, quantityCmp:getQuantity())
            end
        end
    end

    --todo: impl AI tasks next

    self:quit()
end

return SimpleTradingTest
