-- Types --
---@type MaterialDefinition
local MaterialDefinition = require("Shared.Types.MaterialDefinition")
-- Definitions --
local ShaderVarFuncs = require("Shared.Definitions.ShaderVarFuncs")

local CoreComponents = require("Modules.Core.Components")
local CelestialComponents = require("Modules.CelestialObjects.Components")
local PhysicsComponents = require("Modules.Physics.Components")

-- Helper
local function genField(name, type, cmpType)
    return {
        type = type,
        value = function(_, entity)
            local gen = entity:get(cmpType)
            if type == Enums.UniformType.Float3 then
                local v = gen[name]
                return v.x, v.y, v.z
            else
                return gen[name]
            end
        end,
        perInstance = true -- every gen field is per-planet
    }
end

---@class Materials
---@field Asteroid Material
MaterialDefinition {
    name = "Asteroid",
    vs_name = "wvp",
    fs_name = "material/asteroid",
    blendMode = BlendMode.Disabled,
    textures = {
        texDiffuse = { tex = Cache.Texture('rock'), type = Enums.UniformType.Tex2D, settings = nil }
    },
    autoShaderVars = {
        mWorld   = { type = Enums.UniformType.Matrix, value = ShaderVarFuncs.mWorldFunc },
        mWorldIT = { type = Enums.UniformType.MatrixT, value = ShaderVarFuncs.mWorldITFunc },
        scale    = { type = Enums.UniformType.Float, value = ShaderVarFuncs.scaleFunc }
    }
}

---@class Materials
---@field Metal Material
MaterialDefinition {
    name = "Metal",
    vs_name = "wvp",
    fs_name = "material/metal",
    blendMode = BlendMode.Disabled,
    textures = {
        texDiffuse = { tex = Cache.Texture('metal/01_d'), type = Enums.UniformType.Tex2D, settings = nil },
        texNormal  = { tex = Cache.Texture('metal/01_n'), type = Enums.UniformType.Tex2D, settings = nil },
        texSpec    = { tex = Cache.Texture('metal/01_s'), type = Enums.UniformType.Tex2D, settings = nil }
    },
    autoShaderVars = {
        mWorld   = { type = Enums.UniformType.Matrix, value = ShaderVarFuncs.mWorldFunc },
        mWorldIT = { type = Enums.UniformType.MatrixT, value = ShaderVarFuncs.mWorldITFunc },
        scale    = { type = Enums.UniformType.Float, value = ShaderVarFuncs.scaleFunc }
    }
}

---@class Materials
---@field DebugColor Material
MaterialDefinition {
    name = "DebugColor",
    vs_name = "wvp",
    fs_name = "material/solidcolor",
    blendMode = BlendMode.Disabled,
    constShaderVars = {
        color = { type = Enums.UniformType.Float3, value = { 1.0, 0.0, 1.0 } }
    },
    autoShaderVars = {
        mWorld = { type = Enums.UniformType.Matrix, value = ShaderVarFuncs.mWorldFunc },
    }
}

---@class Materials
---@field DebugDeferred Material - Deferred-compatible debug material for testing deferred lighting
MaterialDefinition {
    name = "DebugDeferred",
    vs_name = "wvp",
    fs_name = "material/solidcolor_deferred",
    blendMode = BlendMode.Disabled,
    constShaderVars = {
        color = { type = Enums.UniformType.Float3, value = { 0.8, 0.8, 0.8 } }
    },
    autoShaderVars = {
        mWorld   = { type = Enums.UniformType.Matrix, value = ShaderVarFuncs.mWorldFunc },
        mWorldIT = { type = Enums.UniformType.MatrixT, value = ShaderVarFuncs.mWorldITFunc },
    }
}

---@class Materials
---@field PlanetSurface Material
MaterialDefinition {
    name = "PlanetSurface",
    vs_name = "wvp",
    fs_name = "material/planet",
    blendMode = BlendMode.Disabled,
    textures = nil, -- set at runtime
    constShaderVars = {
        heightMult = { type = Enums.UniformType.Float, value = 1.0 },
        starColor  = { type = Enums.UniformType.Float3, value = { 1.0, 0.5, 0.1 } },
    },
    autoShaderVars = {
        mWorld     = { type = Enums.UniformType.Matrix, value = ShaderVarFuncs.mWorldFunc, perInstance = true },
        mWorldIT   = { type = Enums.UniformType.MatrixT, value = ShaderVarFuncs.mWorldITFunc, perInstance = true },
        scale      = { type = Enums.UniformType.Float, value = ShaderVarFuncs.scaleFunc, perInstance = true },

        time       = { type = Enums.UniformType.Float,
            value = function(_, e)
                ---@cast e Entity
                local time = e:get(CelestialComponents.Simulation.CloudMotion):getTime()
                return time
            end,
            perInstance = false
        },

        oceanLevel = genField("oceanLevel", Enums.UniformType.Float, CelestialComponents.Gen.Planet),
        color1     = genField("color1", Enums.UniformType.Float3, CelestialComponents.Gen.Planet),
        color2     = genField("color2", Enums.UniformType.Float3, CelestialComponents.Gen.Planet),
        color3     = genField("color3", Enums.UniformType.Float3, CelestialComponents.Gen.Planet),
        color4     = genField("color4", Enums.UniformType.Float3, CelestialComponents.Gen.Planet),

        origin     = { type = Enums.UniformType.Float3,
            value = function(eye, entity)
                local rb = entity:get(PhysicsComponents.RigidBody):getRigidBody()
                local o = rb:getPos():relativeTo(eye)
                return o.x, o.y, o.z
            end, perInstance = true },
        rPlanet    = { type = Enums.UniformType.Float,
            value = function(_, e) return e:get(PhysicsComponents.RigidBody):getRigidBody():getScale() end, perInstance = true },
        rAtmo      = { type = Enums.UniformType.Float,
            value = function(_, e)
                local rb = e:get(PhysicsComponents.RigidBody):getRigidBody()
                local gen = e:get(CelestialComponents.Gen.Planet)
                return rb:getScale() * gen.atmoScale
            end, perInstance = false }, -- shared across all planets
    },
}

---@class Materials
---@field PlanetAtmosphere Material
MaterialDefinition {
    name = "PlanetAtmosphere",
    vs_name = "wvp",
    fs_name = "material/atmosphere",
    blendMode = BlendMode.Alpha,
    textures = nil, -- set at runtime
    constShaderVars = {
        starColor = { type = Enums.UniformType.Float3, value = { 1.0, 0.5, 0.1 } },
    },
    autoShaderVars = {
        mWorld   = { type = Enums.UniformType.Matrix, value = ShaderVarFuncs.mWorldFunc, perInstance = true },
        mWorldIT = { type = Enums.UniformType.MatrixT, value = ShaderVarFuncs.mWorldITFunc, perInstance = true },
        scale    = { type = Enums.UniformType.Float, value = ShaderVarFuncs.scaleFunc, perInstance = true },

        origin   = { type = Enums.UniformType.Float3,
            value = function(eye, entity)
                local rb = entity:get(PhysicsComponents.RigidBody):getRigidBody()
                local o = rb:getPos():relativeTo(eye)
                return o.x, o.y, o.z
            end, perInstance = true },
        rPlanet  = { type = Enums.UniformType.Float,
            value = function(_, e) return e:get(PhysicsComponents.RigidBody):getRigidBody():getScale() end, perInstance = true },
        rAtmo    = { type = Enums.UniformType.Float,
            value = function(_, e)
                local rb = e:get(PhysicsComponents.RigidBody):getRigidBody()
                local gen = e:get(CelestialComponents.Gen.Planet)
                return rb:getScale() * gen.atmoScale
            end, perInstance = false },
        scaleVec = { type = Enums.UniformType.Float3,
            value = function(_, e)
                local s = e:get(PhysicsComponents.RigidBody):getRigidBody():getScale()
                return s, s, s
            end, perInstance = true },
    },
}

---@class Materials
---@field PlanetRing Material
MaterialDefinition {
    name = "PlanetRing",
    vs_name = "wvp",                 -- standard WVP vertex shader
    fs_name = "material/planetring", -- fragment shader
    blendMode = BlendMode.Alpha,     -- enable alpha blending
    textures = nil,
    constShaderVars = {
        nearDist = { uniformType = Enums.UniformType.Float, callbackFn = function() return 10000 end },
        farDist = { uniformType = Enums.UniformType.Float, callbackFn = function() return 100000 end }
    },
    autoShaderVars = {
        -- World transform
        mWorld       = { type = Enums.UniformType.Matrix, value = ShaderVarFuncs.mWorldFunc, perInstance = true },
        mWorldIT     = { type = Enums.UniformType.MatrixT, value = ShaderVarFuncs.mWorldITFunc, perInstance = true },

        -- Time for rotation
        time         = { type = Enums.UniformType.Float,
            value = function(_, e)
                ---@cast e Entity
                local time = e:get(CelestialComponents.Simulation.PlanetaryRingMotion):getTime()
                return time
            end,
            perInstance = false
        },

        -- Planet center and radius (for shadow)
        planetPos    = { type = Enums.UniformType.Float3,
            value = function(eye, e)
                ---@cast e Entity
                local planet = e:get(CoreComponents.Parent)
                    :getParent()
                    :get(PhysicsComponents.RigidBody)
                    :getRigidBody()
                local p = planet:getPos()
                return p.x, p.y, p.z
            end,
            perInstance = true
        },
        planetQuat   = { uniformType = Enums.UniformType.Float4,
            callbackFn = function(eye, e)
                ---@cast e Entity
                local planet = e:get(CoreComponents.Parent)
                    :getParent()
                    :get(PhysicsComponents.RigidBody)
                    :getRigidBody()
                local p = planet:getRot()
                return p.x, p.y, p.z, p.w
            end,
            perInstance = true
        },
        planetRadius = { uniformType = Enums.UniformType.Float4,
            callbackFn = function(eye, e)
                ---@cast e Entity
                local planetRadius = e:get(CoreComponents.Parent)
                    :getParent()
                    :get(PhysicsComponents.RigidBody)
                    :getRadius()
                return planetRadius
            end,
            perInstance = true
        },
        ringQuat     = { uniformType = Enums.UniformType.Float4,
            callbackFn = function(_, e)
                local ringBody = e:get(PhysicsComponents.RigidBody):getRigidBody()
                local q = ringBody:getRot() -- returns a quaternion {x, y, z, w}
                return q.x, q.y, q.z, q.w
            end,
            perInstance = true
        },
        -- Ring procedural parameters
        seed         = { type = Enums.UniformType.Float,
            value = function(_, e)
                local seed = e:get(CoreComponents.Seed):getSeed()
                return seed
            end,
            perInstance = true
        },
    },
}

---@class Materials
---@field MoonSurface Material
MaterialDefinition {
    name = "MoonSurface",
    vs_name = "wvp",
    fs_name = "material/moon",
    blendMode = BlendMode.Disabled,
    textures = nil, -- set at runtime
    constShaderVars = {
        starColor        = { type = Enums.UniformType.Float3, value = { 1.0, 0.5, 0.1 } },
        heightMult       = { type = Enums.UniformType.Float, value = 0.03 },
        craterDepth      = { type = Enums.UniformType.Float, value = 1.0 },
        enableAtmosphere = { type = Enums.UniformType.Float, value = 0.0 },
        mercuryBaseTex   = { type = Enums.UniformType.Tex2D, value = Cache.Texture('surface/2k_mercury') },
        moonBaseTex      = { type = Enums.UniformType.Tex2D, value = Cache.Texture('surface/2k_moon') },
    },
    autoShaderVars = {
        mWorld        = { type = Enums.UniformType.Matrix, value = ShaderVarFuncs.mWorldFunc, perInstance = true },
        mWorldIT      = { type = Enums.UniformType.MatrixT, value = ShaderVarFuncs.mWorldITFunc, perInstance = true },
        scale         = { type = Enums.UniformType.Float, value = ShaderVarFuncs.scaleFunc, perInstance = true },

        highlandColor = genField("highlandColor", Enums.UniformType.Float3, CelestialComponents.Gen.Moon),
        mariaColor    = genField("mariaColor", Enums.UniformType.Float3, CelestialComponents.Gen.Moon),

        origin        = { type = Enums.UniformType.Float3,
            value = function(eye, entity)
                local rb = entity:get(PhysicsComponents.RigidBody):getRigidBody()
                local o = rb:getPos():relativeTo(eye)
                return o.x, o.y, o.z
            end, perInstance = true },
        rPlanet       = { type = Enums.UniformType.Float,
            value = function(_, e) return e:get(PhysicsComponents.RigidBody):getRigidBody():getScale() end, perInstance = true },
    },
}
