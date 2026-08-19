local Component = require("Core.ECS.Component")

---@class LightEffectComponent: Component
---@overload fun(duration: number, fadeOutDuration: number|nil, kind: string|nil): LightEffectComponent
local LightEffectComponent = Subclass("LightEffectComponent", Component, function(self, duration, fadeOutDuration, kind)
    self:setComponentName("LightEffect")
    self.duration = math.max(0, duration or 0)
    self.remaining = self.duration
    self.fadeOutDuration = math.max(0, fadeOutDuration or self.duration)
    self.kind = kind or "transient"
    self.baseIntensity = nil
end)

---@return number
function LightEffectComponent:getRemaining()
    return self.remaining
end

---@return number
function LightEffectComponent:getDuration()
    return self.duration
end

return LightEffectComponent
