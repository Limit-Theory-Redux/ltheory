local Registry = require("Core.ECS.Registry")
local Rendering = require("Modules.Rendering.Components")
local ExplosionEffectComponent =
    require("Modules.Constructs.Components.ExplosionEffectComponent")

---@class ExplosionEffectSystem
---Advances explosion effect lifetime. Non-looping effects are destroyed when
---their duration elapses; looping effects restart their age each cycle so a
---single persistent entity serves repeating visuals without spawn/destroy
---churn. Also drives the deferred point-light intensity envelope (flash ->
---ember decay) so nearby hulls are lit in sync with the fireball.
local ExplosionEffectSystem = Class("ExplosionEffectSystem", function() end)

function ExplosionEffectSystem:update(dt)
    if not dt or dt <= 0 then
        return
    end

    -- Two-phase like PointLightSystem: destroying inside Registry:view
    -- would swap-remove from the dense storage being iterated.
    local expired = {}

    for entity, effect in Registry:view(ExplosionEffectComponent) do
        if entity:isValid() then
            effect.age = effect.age + dt

            local light = entity:get(Rendering.PointLight)
            if light then
                -- Same life envelope the shader applies visually.
                local fade = math.exp(-1.1 * math.max(0, effect.age))
                light:setIntensity(effect.lightIntensity * fade)
            end

            if effect.age >= effect.duration then
                if effect.loop then
                    effect.age = effect.age % effect.duration
                else
                    expired[#expired + 1] = entity
                end
            end
        end
    end

    for i = 1, #expired do
        if expired[i]:isValid() then
            Registry:destroyEntity(expired[i], Registry.DESTROY_MODE.DESTROY_CHILDREN)
        end
    end
end

return ExplosionEffectSystem
