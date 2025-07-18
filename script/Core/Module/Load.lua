-- TODO: add caching

-- Load components of the specified modules in a single list.
-- Returns an error if 2 modules have components with the same name.
function loadComponents(...)
    local components = {}
    for _, module in ipairs({ ... }) do
        local moduleComponents = requireAll("Modules." .. module .. ".Components")
        if moduleComponents ~= nil then
            for k, v in pairs(moduleComponents) do
                -- if components[k] ~= nil then
                components[k] = v
                -- else
                --     error("Duplicate component '" .. k .. "' in module '" .. module .. "'")
                -- end
            end
        end
    end
    return components
end

-- Load entity of the specified modules in a single list.
-- Returns an error if 2 modules have entities with the same name.
function loadEntities(...)
    local entities = {}
    for _, module in ipairs({ ... }) do
        local moduleEntities = requireAll("Modules." .. module .. ".Entities")
        if moduleEntities ~= nil then
            for k, v in pairs(moduleEntities) do
                -- if entities[k] ~= nil then
                entities[k] = v
                -- else
                --     error("Duplicate entity '" .. k .. "' in module '" .. module .. "'")
                -- end
            end
        end
    end
    return entities
end

-- Load system of the specified modules in a single list.
-- Returns an error if 2 modules have systems with the same name.
function loadSystems(...)
    local systems = {}
    for _, module in ipairs({ ... }) do
        local moduleSystems = requireAll("Modules." .. module .. ".Systems")
        if moduleSystems ~= nil then
            for k, v in pairs(moduleSystems) do
                -- if systems[k] ~= nil then
                systems[k] = v
                -- else
                --     error("Duplicate system '" .. k .. "' in module '" .. module .. "'")
                -- end
            end
        end
    end
    return systems
end
