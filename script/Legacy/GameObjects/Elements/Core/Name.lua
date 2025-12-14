local Entity = require('Legacy.GameObjects.Entity')
local NameComponent = require("Modules.Core.Components.NameComponent")

function Entity:getName()
    local nameComponent = self.entity:get(NameComponent)
    return nameComponent and nameComponent:getName() or format('%s @ %p', type(self), self)
end

function Entity:setName(name)
    local nameComponent = self.entity:get(NameComponent)
    if nameComponent then
        nameComponent:setName(name)
    else
        self.entity:add(NameComponent(name))
    end
end

function Entity:__tostring()
    return self:getName()
end
