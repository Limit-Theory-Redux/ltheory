local Registry = require("Core.ECS.Registry")
local Physics = require("Modules.Physics.Components")
local Rendering = require("Modules.Rendering.Components")
local LensFlareSystem = require("Modules.Rendering.Systems.LensFlareSystem")
local ExplosionEffectComponent =
    require("Modules.Constructs.Components.ExplosionEffectComponent")

---@class ExplosionEffectSystem
---Advances explosion effect lifetime. Non-looping effects are destroyed when
---their duration elapses; looping effects restart their age each cycle so a
---single persistent entity serves repeating visuals without spawn/destroy
---churn. Also drives the deferred point-light intensity envelope (flash ->
---ember decay) so nearby hulls are lit in sync with the fireball. Frozen
---effects keep their age. drawFlares draws the ignition flash's lens flare
---for the effects that want one.
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
            if not effect.frozen then
                effect.age = effect.age + dt
            end

            local light = entity:get(Rendering.PointLight)
            if light then
                -- Same life envelope the shader applies visually.
                local fade = math.exp(-1.1 * math.max(0, effect.age))
                light:setIntensity(effect.lightIntensity * fade)
            end

            if not effect.frozen and effect.age >= effect.duration then
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

-- The ignition flash's flare (Jack's explosion testbed settings)
local FLARE_FLASH_DECAY = 2.6 -- 1/s
local FLARE_MIN_FLASH = 0.03
local flareOptions = { color = { 1.0, 0.72, 0.42 }, intensity = 1, ghosts = 0.8, halo = 0.7 }

---Lens flares for the effects with `flare` set: the ignition flash, fading
---out within the first second. Call it while drawing the frame, with the
---sun's flare.
function ExplosionEffectSystem:drawFlares()
    for entity, effect in Registry:view(ExplosionEffectComponent) do
        if effect.flare and entity:isValid() then
            local flash = math.exp(-effect.age * FLARE_FLASH_DECAY)
            local transform = entity:get(Physics.Transform)
            if flash > FLARE_MIN_FLASH and transform then
                flareOptions.intensity = 1.4 * flash
                LensFlareSystem:drawAt(transform:getPos(), flareOptions)
            end
        end
    end
end

return ExplosionEffectSystem
