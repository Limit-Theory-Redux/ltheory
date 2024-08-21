local Component = require('_ECS_WIP_TEMP.Components.Component')

---@class RNGComponent: Component
---@overload fun(self:RNGComponent, storageInfo: rngInfo): RNGComponent subclass internal
---@overload fun(seed: integer): RNGComponent subclass external
local RNGComponent = Subclass(Component, function(self)
    self:setComponentName("GenerationRNGComponent")

    -- Set Component Archetype
    self:setArchetype(Enums.ComponentArchetype.RNGComponent)
end)

---@private
function RNGComponent:setRNG(rngInfo)
    self.rngInfo = rngInfo
end

---@return integer
function RNGComponent:getRNG()
    return self.rngInfo
end

return RNGComponent
