local Component = require('Components.Component')

---@class EffectComponent: Component
---@overload fun(self: EffectComponent, effect: Effect): EffectComponent subclass internal
---@overload fun(effect: Effect): EffectComponent subclass external
local EffectComponent = Subclass("EffectComponent", Component, function(self, effect)
    ---@cast self EffectComponent
    self:setComponentName("RenderingEffect")
end)

return EffectComponent
