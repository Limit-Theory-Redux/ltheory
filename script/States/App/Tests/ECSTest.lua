local GlobalStorage = require("_ECS_WIP_TEMP.Systems.GlobalStorage")                              --!temp path
local UniverseSystem = require("_ECS_WIP_TEMP.Systems.CelestialObjects.UniverseGenerationSystem") --!temp path

local EntityComponentSystemTest = require('States.Application')

---@diagnostic disable-next-line: duplicate-set-field
function EntityComponentSystemTest:onInit()
    -- Create universes
    UniverseSystem:createUniverse(0)
    UniverseSystem:createUniverse(1) -- test uniqueness
    UniverseSystem:createUniverse(0) -- test sameness

    -- Validate universes
    local universes = GlobalStorage:getEntitiesFromArchetype(Enums.EntityArchetype.UniverseEntity)

    ---@param universe UniverseEntity
    for universe in Iterator(universes) do
        self:printHierarchy(universe)
    end

    self:quit()
end

function EntityComponentSystemTest:printHierarchy(rootEntity)
    local stack = { { entity = rootEntity, depth = 0 } }
    local processedEntities = {}

    while #stack > 0 do
        local item = table.remove(stack)
        local current = item.entity
        local depth = item.depth

        if not processedEntities[current] then
            processedEntities[current] = true

            local indent = string.rep("│   ", depth)
            local branch = depth > 0 and "├─ " or ""

            ---@type SeedComponent
            local seedComponent = current:findComponentByArchetype(Enums.ComponentArchetype.SeedComponent)
            Log.Debug("%s%s%s with seed %s", indent, branch, current, seedComponent and seedComponent:getSeed() or "No Seed Component")

            ---@type EntityHierarchyComponent
            local hierarchyComponent = current:findComponentByArchetype(Enums.ComponentArchetype.HierarchyComponent)
            if hierarchyComponent then
                for child in hierarchyComponent:iterChildren() do
                    table.insert(stack, { entity = child, depth = depth + 1 })
                end
            end
        end
    end
end

return EntityComponentSystemTest
