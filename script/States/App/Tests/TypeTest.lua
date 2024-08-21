local TypeTest = require('States.Application')

local EntityInfo = require('_ECS_WIP_TEMP.Shared.Types.EntityInfo')
local ComponentInfo = require('_ECS_WIP_TEMP.Shared.Types.ComponentInfo')

---@diagnostic disable-next-line: duplicate-set-field
function TypeTest:onInit()
    -- Custom Types
    local entityInfo = EntityInfo { id = 0, archetype = 0 }
    print(entityInfo, type(entityInfo), Enums.Type:getName(type(entityInfo)))

    local componentInfo = ComponentInfo { id = 0, archetype = 0 }
    print(componentInfo, type(componentInfo), Enums.Type:getName(type(componentInfo)))

    -- Make sure vanilla type still works
    print(type(), type(0), type(""), type(true), type(function() end))
end

return TypeTest
