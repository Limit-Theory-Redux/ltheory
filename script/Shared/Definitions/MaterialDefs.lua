-- Types --
---@type MaterialDefinition
local MaterialDefinition = require("Shared.Types.MaterialDefinition")
-- Definitions --
local ShaderVarFuncs = require("Shared.Definitions.ShaderVarFuncs")

local CelestialComponents = require("Modules.CelestialObjects.Components")
local PhysicsComponents = require("Modules.Physics.Components")

-- Helper
local function genField(name, type)
    return {
        uniformName = name,
        uniformType = type,
        callbackFn = function(_, entity)
            local gen = entity:get(CelestialComponents.Gen.Planet)
            if type == Enums.UniformType.Float3 then
                local v = gen[name]
                return v.x, v.y, v.z
            else
                return gen[name]
            end
        end
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
        { texName = "texDiffuse", tex = Cache.Texture('rock'), texType = Enums.UniformType.Tex2D, texSettings = nil }
    },
    autoShaderVars = {
        { uniformName = "mWorld",   uniformType = Enums.UniformType.Matrix,  callbackFn = ShaderVarFuncs.mWorldFunc },
        { uniformName = "mWorldIT", uniformType = Enums.UniformType.MatrixT, callbackFn = ShaderVarFuncs.mWorldITFunc },
        { uniformName = "scale",    uniformType = Enums.UniformType.Float,   callbackFn = ShaderVarFuncs.scaleFunc }
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
        { texName = "texDiffuse", tex = Cache.Texture('metal/01_d'), texType = Enums.UniformType.Tex2D, texSettings = nil },
        { texName = "texNormal",  tex = Cache.Texture('metal/01_n'), texType = Enums.UniformType.Tex2D, texSettings = nil },
        { texName = "texSpec",    tex = Cache.Texture('metal/01_s'), texType = Enums.UniformType.Tex2D, texSettings = nil }
    },
    autoShaderVars = {
        { uniformName = "mWorld",   uniformType = Enums.UniformType.Matrix,  callbackFn = ShaderVarFuncs.mWorldFunc },
        { uniformName = "mWorldIT", uniformType = Enums.UniformType.MatrixT, callbackFn = ShaderVarFuncs.mWorldITFunc },
        { uniformName = "scale",    uniformType = Enums.UniformType.Float,   callbackFn = ShaderVarFuncs.scaleFunc }
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
        { uniformName = "color", uniformType = Enums.UniformType.Float3, callbackFn = function() return 1.0, 0.0, 1.0 end }
    },
    autoShaderVars = {
        { uniformName = "mWorld", uniformType = Enums.UniformType.Matrix, callbackFn = ShaderVarFuncs.mWorldFunc },
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
        { uniformName = "heightMult", uniformType = Enums.UniformType.Float,  callbackFn = function() return 1.0 end },
        { uniformName = "starColor",  uniformType = Enums.UniformType.Float3, callbackFn = function() return 1.0, 0.5, 0.1 end },
    },
    autoShaderVars = {
        { uniformName = "mWorld",   uniformType = Enums.UniformType.Matrix,  callbackFn = ShaderVarFuncs.mWorldFunc },
        { uniformName = "mWorldIT", uniformType = Enums.UniformType.MatrixT, callbackFn = ShaderVarFuncs.mWorldITFunc },
        { uniformName = "scale",    uniformType = Enums.UniformType.Float,   callbackFn = ShaderVarFuncs.scaleFunc },

        genField("oceanLevel", Enums.UniformType.Float),
        genField("color1", Enums.UniformType.Float3),
        genField("color2", Enums.UniformType.Float3),
        genField("color3", Enums.UniformType.Float3),
        genField("color4", Enums.UniformType.Float3),

        { uniformName = "origin", uniformType = Enums.UniformType.Float3,
            callbackFn = function(eye, entity)
                local rb = entity:get(PhysicsComponents.RigidBody):getRigidBody()
                local o = rb:getPos():relativeTo(eye)
                return o.x, o.y, o.z
            end },
        { uniformName = "rPlanet", uniformType = Enums.UniformType.Float,
            callbackFn = function(_, e) return e:get(PhysicsComponents.RigidBody):getRigidBody():getScale() end },
        { uniformName = "rAtmo", uniformType = Enums.UniformType.Float,
            callbackFn = function(_, e)
                local rb = e:get(PhysicsComponents.RigidBody):getRigidBody()
                local gen = e:get(CelestialComponents.Gen.Planet)
                return rb:getScale() * gen.atmoScale
            end },
    }
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
        { uniformName = "starColor", uniformType = Enums.UniformType.Float3, callbackFn = function() return 1.0, 0.5, 0.1 end },
    },
    autoShaderVars = {
        { uniformName = "mWorld",   uniformType = Enums.UniformType.Matrix,  callbackFn = ShaderVarFuncs.mWorldFunc },
        { uniformName = "mWorldIT", uniformType = Enums.UniformType.MatrixT, callbackFn = ShaderVarFuncs.mWorldITFunc },
        { uniformName = "scale",    uniformType = Enums.UniformType.Float,   callbackFn = ShaderVarFuncs.scaleFunc },

        { uniformName = "origin", uniformType = Enums.UniformType.Float3,
            callbackFn = function(eye, entity)
                local rb = entity:get(PhysicsComponents.RigidBody):getRigidBody()
                local o = rb:getPos():relativeTo(eye)
                return o.x, o.y, o.z
            end },
        { uniformName = "rPlanet", uniformType = Enums.UniformType.Float,
            callbackFn = function(_, e) return e:get(PhysicsComponents.RigidBody):getRigidBody():getScale() end },
        { uniformName = "rAtmo", uniformType = Enums.UniformType.Float,
            callbackFn = function(_, e)
                local rb = e:get(PhysicsComponents.RigidBody):getRigidBody()
                local gen = e:get(CelestialComponents.Gen.Planet)
                return rb:getScale() * gen.atmoScale
            end },
        { uniformName = "scaleVec", uniformType = Enums.UniformType.Float3,
            callbackFn = function(_, e)
                local s = e:get(PhysicsComponents.RigidBody):getRigidBody():getScale()
                return s, s, s
            end },
    }
}
