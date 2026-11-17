local Component = require("Core.ECS.Component")

---@class ExplosionEffectComponent: Component
---@overload fun(config?: table): ExplosionEffectComponent
---Data-only parameters and runtime state for a volumetric explosion effect.
---Behavior (aging, looping, teardown, light envelope) lives in
---ExplosionEffectSystem; rendering reads this component in the additive pass.
---
---Loop mode: when `loop` is true the system restarts the effect every
---`duration` seconds instead of destroying the entity - one persistent
---entity replaces periodic spawn/destroy churn for repeating effects.
---
---Frozen: while `frozen` is true the system leaves `age` alone (and never
---removes the effect), so whoever froze it can scrub through it.
---
---Flare: with `flare` true, ExplosionEffectSystem:drawFlares draws a lens
---flare for the ignition flash.
local ExplosionEffectComponent = Subclass("ExplosionEffectComponent", Component, function(self, config)
    config = config or {}
    self:setComponentName("ExplosionEffect")

    -- Static visual parameters.
    ---Deterministic shader seed (per-instance noise variation).
    self.seed = config.seed or 0
    ---World-unit fireball radius (billboard half-extent).
    self.size = config.size or 10.0

    -- Lifetime state (owned by ExplosionEffectSystem).
    self.age = 0.0
    self.duration = config.duration or 2.5
    self.loop = config.loop == true
    self.frozen = false
    self.flare = config.flare == true

    -- Deferred point-light parameters (illumination of nearby hulls).
    self.lightColor = config.lightColor or Color(1.0, 0.55, 0.25, 1.0)
    self.lightIntensity = config.lightIntensity or 3.0
    self.lightRadius = config.lightRadius or 1.0
end)

return ExplosionEffectComponent
