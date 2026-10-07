-- Types --
---@type MaterialType
local MaterialType = require("Shared.Types.MaterialType")

local CelestialComponents = require("Modules.CelestialObjects.Components")

-- Every material is a `MaterialType`: the shader, the fixed-function state
-- (the blend mode picks the scene bucket; cull and depth default to those of
-- the scene pass of the bucket), the `MaterialParams` defaults, default
-- textures and an optional `perDraw` callback.
--
--   * `MaterialParams` (shader group 1) is written once per instance:
--     `local p = mat:params(); p.heightMult = 2; mat:commit()`.
--   * `perDraw(entity, user)` fills `drawUser` (a `Vec4f[7]`, shader group 2)
--     of each drawn mesh, for values that change from frame to frame or from
--     entity to entity. It runs only for the meshes that survive culling.
--   * `mWorld`, `mWorldIT` and the body's scale (`drawScale.x`) are in the
--     draw block for every mesh, no callback needed.
--
-- See res/shader/include/draw_block.glsl.

---@class Materials
---@field Asteroid MaterialType
MaterialType {
    name     = "Asteroid",
    shader   = { "wvp", "material/asteroid" },
    state    = { blend = BlendMode.Disabled },
    textures = {
        texDiffuse = { tex = Cache.Texture('rock') },
    },
}

---@class Materials
---@field Metal MaterialType
MaterialType {
    name     = "Metal",
    shader   = { "wvp", "material/metal" },
    state    = { blend = BlendMode.Disabled },
    -- The paint stripes (`paintAttrib`, `paintColor`) were never set from Lua;
    -- zero is what the shader has always run with.
    textures = {
        texDiffuse = { tex = Cache.Texture('metal/01_d') },
        texNormal  = { tex = Cache.Texture('metal/01_n') },
        texSpec    = { tex = Cache.Texture('metal/01_s') },
    },
}

---@class Materials
---@field DebugColor MaterialType
MaterialType {
    name     = "DebugColor",
    shader   = { "wvp", "material/solidcolor" },
    state    = { blend = BlendMode.Disabled },
    defaults = { color = Vec3f(1.0, 0.0, 1.0) },
}

---@class Materials
---@field PlanetSurface MaterialType
-- Per instance: color1..4, oceanLevel and atmoScale (from the planet's gen options;
-- the shaders derive rPlanet/rAtmo from the draw's scale, so a rescaled body stays right).
MaterialType {
    name     = "PlanetSurface",
    shader   = { "wvp", "material/planet" },
    state    = { blend = BlendMode.Disabled },
    defaults = { heightMult = 1.0 },
    textures = {
        surface = { sampler = Samplers.LinearMipClamp }, -- set per planet
    },
    -- drawUser[0].x: the time of the cloud motion
    perDraw  = function(entity, user)
        ---@cast entity Entity
        user[0].x = entity:get(CelestialComponents.Simulation.CloudMotion):getTime()
    end,
}

---@class Materials
---@field PlanetAtmosphere MaterialType
-- Per instance: atmoScale.
MaterialType {
    name   = "PlanetAtmosphere",
    shader = { "wvp", "material/atmosphere" },
    state  = { blend = BlendMode.Alpha },
}

---@class Materials
---@field PlanetRing MaterialType
-- Per instance: rMin, rMax and seed; enableDebug/debugMode for tests.
MaterialType {
    name     = "PlanetRing",
    shader   = { "wvp", "material/planetring" },
    state    = { blend = BlendMode.Alpha },
    defaults = {
        ringHeight    = 50,
        rotationSpeed = 2.0,
        twistFactor   = 0.25,
        enableDebug   = 0,
        debugMode     = 0,
    },
    -- drawUser[0].x: the time of the ring rotation
    perDraw  = function(entity, user)
        ---@cast entity Entity
        user[0].x = entity:get(CelestialComponents.Simulation.PlanetaryRingMotion):getTime()
    end,
}

---@class Materials
---@field MoonSurface MaterialType
-- Per instance: highlandColor and mariaColor.
MaterialType {
    name     = "MoonSurface",
    shader   = { "wvp", "material/moon" },
    state    = { blend = BlendMode.Disabled },
    defaults = { heightMult = 0.03, enableAtmosphere = 0.0 },
    textures = {
        surface = { sampler = Samplers.LinearMipClamp }, -- set per moon
    },
}

---@class Materials
---@field Star MaterialType
MaterialType {
    name     = "Star",
    shader   = { "wvp", "material/star" },
    state    = { blend = BlendMode.Disabled },
    textures = {
        sunTex = { tex = Cache.Texture('surface/2k_sun') },
    },
    -- drawUser[0].x: time, [0].y: starTemp, [1].xyz: starTint
    perDraw  = function(entity, user)
        ---@cast entity Entity
        local CoreComponents = require("Modules.Core.Components")
        user[0].x = Engine:getTime()

        local lumCmp = entity:get(CelestialComponents.Luminosity)
        local luminosity = lumCmp and lumCmp:getLuminosity() or 1.0
        user[0].y = 1.0 + math.log(math.max(0.1, luminosity)) * 0.3

        local typeCmp = entity:get(CoreComponents.Type)
        local starType = typeCmp and typeCmp:getSubtype() or "MainSequence"
        if starType == "RedGiant" then
            user[1].x, user[1].y, user[1].z = 1.0, 0.3, 0.1
        elseif starType == "WhiteDwarf" then
            user[1].x, user[1].y, user[1].z = 0.8, 0.85, 1.0
        else
            user[1].x, user[1].y, user[1].z = 1.0, 0.85, 0.6
        end
    end,
}

---@class Materials
---@field TravelDrive MaterialType
MaterialType {
    name     = "TravelDrive",
    shader   = { "traveldrive", "material/traveldrive" },
    state    = { blend = BlendMode.Additive },
    defaults = { effectScale = 1.5 }, -- inflation distance along normals
    -- drawUser[0].x: time, .y: intensity, .z: driveSpeed
    perDraw  = function(_, user)
        local TDS = require("Modules.Constructs.Systems.TravelDriveSystem")
        user[0].x = Engine:getTime()

        local intensity = 0
        local state = TDS:getState()
        if state == "charging" then
            intensity = TDS:getChargeProgress()
        elseif state == "active" then
            intensity = 0.7 + 0.3 * math.min(1, TDS:getMultiplier() / 50)
        elseif state == "decelerating" then
            intensity = math.max(0, (TDS:getMultiplier() - 1) / 50)
        end
        user[0].y = intensity
        user[0].z = TDS:getMultiplier()
    end,
}
