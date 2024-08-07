local GlobalStorage = require("_ECS_WIP_TEMP.Systems.GlobalStorage") --!temp path
local Universe = require("_ECS_WIP_TEMP.Systems.Universe")           --!temp path

local EntityComponentSystemTest = require('States.Application')

---@diagnostic disable-next-line: duplicate-set-field
function EntityComponentSystemTest:onInit()
    ---@type GlobalStorage
    self.globalStorage = GlobalStorage() --* decide where to store this later
    ---@type Universe
    self.universe = Universe(0)          --* decide where to store this later

    -- Create a random star system with an economy
    self.universe:createStarSystem(true)

    ---@param archetype EntityArchetype
    ---@param entities table<Entity>
    for archetype, entities in ipairs(GlobalStorage:getEntities()) do
        ---@param entity Entity
        for _, entity in pairs(entities) do
            local nameComponent = entity:findComponentByName("Name")
            ---@cast nameComponent NameComponent
            print(nameComponent:getName() .. " (" .. Enums.EntityArchetype:getName(archetype) .. ")")

            for component in entity:iterComponents() do
                if component:getComponentName() ~= "NameComponent" then
                    print(" - " .. component:getComponentName())
                end
            end
        end
    end
end

return EntityComponentSystemTest
