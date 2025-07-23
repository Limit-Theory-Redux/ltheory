-- TODO: add caching

local function loadAllElements(group, ...)
    local elements = {}
    for _, module in ipairs({ ... }) do
        local moduleElements = requireAll("Modules." .. module .. "." .. group)
        if moduleElements ~= nil then
            for k, v in pairs(moduleElements) do
                -- if elements[k] ~= nil then
                elements[k] = v
                -- else
                --     error("Duplicate element " .. group .. "/" .. k .. " in module '" .. module .. "'")
                -- end
            end
        end
    end
    return elements
end

local function loadModuleElements(module, group, ...)
    local args = { ... }
    if #args == 1 then
        return require("Modules." .. module .. "." .. group .. "." .. args[1])
    end
    local elements = {}
    for _, element in ipairs(args) do
        local moduleElement = require("Modules." .. module .. "." .. group .. "." .. element)
        -- if elements[element] ~= nil then
        elements[element] = moduleElement
        -- else
        --     error("Duplicate element " .. group .. "/" .. element .. " in module '" .. module .. "'")
        -- end
    end
    return elements
end

-- Load components of the specified modules in a single list.
-- Returns an error if 2 modules have components with the same name.
function loadComponents(...)
    return loadAllElements("Components", ...)
end

-- Load entity of the specified modules in a single list.
-- Returns an error if 2 modules have entities with the same name.
function loadEntities(...)
    return loadAllElements("Entities", ...)
end

-- Load system of the specified modules in a single list.
-- Returns an error if 2 modules have systems with the same name.
function loadSystems(...)
    return loadAllElements("Systems", ...)
end

-- Load components of the specified modules in a single list.
-- Returns an error if 2 modules have components with the same name.
function loadModuleComponents(module, ...)
    return loadModuleElements(module, "Components", ...)
end

-- Load entity of the specified modules in a single list.
-- Returns an error if 2 modules have entities with the same name.
function loadModuleEntities(module, ...)
    return loadModuleElements(module, "Entities", ...)
end

-- Load system of the specified modules in a single list.
-- Returns an error if 2 modules have systems with the same name.
function loadModuleSystems(module, ...)
    return loadModuleElements(module, "Systems", ...)
end
