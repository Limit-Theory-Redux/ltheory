local Registry = require("Core.ECS.Registry")
local NameComponent = require("Core.ECS.NameComponent")

local function Entity(name, ...)
    local entityId = Registry:createEntity()
    Registry:add(entityId, NameComponent(name or "Entity"))
    for _, component in ipairs({ ... }) do
        Registry:add(entityId, component)
    end
    return entityId
end

return Entity
