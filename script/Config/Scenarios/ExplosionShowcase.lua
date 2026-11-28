-- Explosion Showcase: one volumetric explosion (the weapons' death explosion)
-- next to the player's ship, to look at and tune. The camera starts orbiting
-- it; Regenerate (B) sets off a new one. Its own keys are on the numpad, so
-- the ship's keys still work: Enter loops it, 5 picks the next size class,
-- + - scale its size, * / its duration, . freezes its clock, 4 and 6 (held)
-- scrub through it, 0 clears the stage. (Was Jack's Explosion testbed.)

local Registry = require("Core.ECS.Registry")
local PhysicsComponents = require("Modules.Physics.Components")
local ExplosionEffectEntity = require("Modules.Constructs.Entities.ExplosionEffectEntity")
local ExplosionEffectComponent = require("Modules.Constructs.Components.ExplosionEffectComponent")
local Helpers = require("Config.Scenarios.Helpers")

-- The size classes, as Config.weapons.deathExplosion has them
local PRESETS = { "small", "medium", "large", "capital" }

local DISTANCE = 6.0 -- from the player's ship (game units, 1 = 10 km)
-- The orbit camera around the explosion (game units): Jack's testbed camera
local CAMERA = { distance = 4.0, minDistance = 0.5, maxDistance = 20.0, zoomRatio = 0.15 }
local SCALE_STEP, MIN_SIZE_SCALE, MAX_SIZE_SCALE = 0.25, 0.25, 4.0
local MIN_DURATION_SCALE, MAX_DURATION_SCALE = 0.5, 3.0
local SCRUB_SPEED = 0.35 -- seconds of explosion per second held

---@return table|nil the explosion's component while there is one
local function current(data)
    local entity = data.explosion
    return entity and entity:isValid() and entity:get(ExplosionEffectComponent) or nil
end

local function clear(data)
    if data.explosion and data.explosion:isValid() then
        Registry:destroyEntity(data.explosion, Registry.DESTROY_MODE.DESTROY_CHILDREN)
    end
    data.explosion = nil
end

-- A new explosion at the anchor, with a seed of its own
local function spawn(game, loop)
    local data = game.scenarioData
    clear(data)
    local preset = Config.weapons.deathExplosion[PRESETS[data.preset]]
    data.serial = data.serial + 1
    data.explosion = ExplosionEffectEntity(data.seed + data.serial * 7919, {
        position = data.anchor:get(PhysicsComponents.Transform):getPos(),
        size = preset.size * data.sizeScale,
        duration = preset.duration * data.durationScale,
        lightIntensity = preset.lightIntensity,
        lightRadius = preset.lightRadius,
        loop = loop,
        flare = true,
    })
    -- A frozen clock stays frozen for the new one
    data.explosion:get(ExplosionEffectComponent).frozen = data.frozen
end

return {
    name = "Explosion Showcase",
    description = "A death explosion up close; B sets off a new one, numpad keys tune it.",
    stations = false,

    setup = function(game)
        local data = game.scenarioData
        data.seed = game.rng:get31()
        data.serial = 0
        data.preset = 2 -- medium
        data.sizeScale, data.durationScale = 1.0, 1.0
        data.frozen = false

        -- What the camera orbits: where the explosions go
        local anchor = Registry:createEntity()
        Registry:add(anchor, PhysicsComponents.Transform())
        anchor:get(PhysicsComponents.Transform):setPos(Helpers.outward(game, DISTANCE))
        game:addSceneEntity(anchor)
        data.anchor = anchor
        game:setOrbitFocus(anchor, CAMERA)

        -- LTHEORY_EFFECT_QUALITY=1..3 draws it at that Effect Quality (for comparing their cost).
        local quality = tonumber(os.getenv("LTHEORY_EFFECT_QUALITY"))
        if quality then GameState.render.effectQuality = quality end
        -- LTHEORY_EXPLOSION_LOOP=1 loops it from the start (for measuring a running explosion).
        spawn(game, os.getenv("LTHEORY_EXPLOSION_LOOP") == "1")
    end,

    regenerate = function(game)
        local effect = current(game.scenarioData)
        spawn(game, effect ~= nil and effect.loop)
    end,

    onInput = function(game, dt)
        local data = game.scenarioData
        local effect = current(data)

        if Input:isPressed(Button.KeyboardNumpadEnter) then
            spawn(game, not (effect and effect.loop))
        elseif Input:isPressed(Button.KeyboardNumpad5) then
            data.preset = data.preset % #PRESETS + 1
            spawn(game, effect ~= nil and effect.loop)
        elseif Input:isPressed(Button.KeyboardNumpad0) then
            clear(data)
        end
        effect = current(data)

        if Input:isPressed(Button.KeyboardNumpadDecimal) then
            data.frozen = not data.frozen
        end
        local scrub = 0
        if Input:isDown(Button.KeyboardNumpad6) then scrub = scrub + SCRUB_SPEED end
        if Input:isDown(Button.KeyboardNumpad4) then scrub = scrub - SCRUB_SPEED end
        if scrub ~= 0 then data.frozen = true end

        -- Size and duration change the running explosion at once
        local resized, retimed = false, false
        if Input:isPressed(Button.KeyboardNumpadAdd) then
            data.sizeScale = math.min(data.sizeScale + SCALE_STEP, MAX_SIZE_SCALE)
            resized = true
        elseif Input:isPressed(Button.KeyboardNumpadSubtract) then
            data.sizeScale = math.max(data.sizeScale - SCALE_STEP, MIN_SIZE_SCALE)
            resized = true
        end
        if Input:isPressed(Button.KeyboardNumpadMultiply) then
            data.durationScale = math.min(data.durationScale + SCALE_STEP, MAX_DURATION_SCALE)
            retimed = true
        elseif Input:isPressed(Button.KeyboardNumpadDivide) then
            data.durationScale = math.max(data.durationScale - SCALE_STEP, MIN_DURATION_SCALE)
            retimed = true
        end

        if effect then
            local preset = Config.weapons.deathExplosion[PRESETS[data.preset]]
            effect.frozen = data.frozen
            if data.frozen then
                effect.age = math.max(0, math.min(effect.age + scrub * dt, effect.duration))
            end
            if resized then
                effect.size = preset.size * data.sizeScale
            end
            if retimed then
                effect.duration = preset.duration * data.durationScale
                if not effect.loop then
                    effect.age = math.min(effect.age, effect.duration * 0.5)
                end
            end
        end
    end,

    controls = {
        { keys = "B", action = "New explosion" },
        { keys = "Numpad Enter", action = "Loop on / off" },
        { keys = "Numpad 5", action = "Next size class (small, medium, large, capital)" },
        { keys = "Numpad + -", action = "Bigger / smaller" },
        { keys = "Numpad * /", action = "Longer / shorter" },
        { keys = "Numpad .", action = "Freeze / unfreeze" },
        { keys = "Numpad 4 6 (held)", action = "Scrub back / forward (freezes)" },
        { keys = "Numpad 0", action = "Clear the stage" },
    },

    overlayLines = function(game)
        local data = game.scenarioData
        local effect = current(data)
        local status
        if effect then
            status = string.format("%s   size %.2f   duration %.2f s   age %.2f s%s%s",
                PRESETS[data.preset], effect.size, effect.duration, effect.age,
                effect.loop and "   LOOP" or "", data.frozen and "   FROZEN" or "")
        else
            status = PRESETS[data.preset] .. "   (stage empty)" .. (data.frozen and "   FROZEN" or "")
        end
        return {
            "Explosion Showcase",
            status,
            "B: new explosion    Numpad: Enter loop    5 size class    + - size    * / duration",
            "Numpad: . freeze    4 6 scrub (hold)    0 clear    F5: reload shaders",
        }
    end,

    teardown = function(game)
        clear(game.scenarioData)
    end,
}
