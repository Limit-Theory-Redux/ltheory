local Component = require('Component')

---@class EffectComponent: Component
---@overload fun(effect: Effect): EffectComponent subclass external
local EffectComponent = Subclass(Component, function(self, effect)
    ---@cast self EffectComponent
    self:setComponentName("RenderingEffect")
end)

return EffectComponent
