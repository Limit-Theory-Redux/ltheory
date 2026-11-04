local Generator = require('Shared.Generation.Generator')
local ColorLUT = require('Shared.Generation.ColorLUT')

local function generateNebulaIFS(rng, res, starDir)
    Profiler.Begin('Nebula.Generate.IFS')
    local shader = Cache.Shader('fullscreen_ndc', 'gen/nebula')
    local P = shader:blockType('Params')
    local p = P()

    do -- Nebula color
        local h = rng:getUniform()
        local s = rng:getUniformRange(0.2, 0.8)
        local l = rng:getUniformRange(0.2, 0.8)
        local color = Color.FromHSL(h, s, l)
        p.color.x, p.color.y, p.color.z = color.r, color.g, color.b
    end

    p.brightnessScale = GameState.gen.nebulaBrightnessScale

    p.roughness = 0.65 + 0.05 * rng:getSign() * rng:getUniform() ^ 2

    p.seed = rng:getUniformRange(1, 1000)

    local lutR = ColorLUT(rng, 5, 0.30, 0.6)
    local lutG = ColorLUT(rng, 5, 0.30, 0.6)
    local lutB = ColorLUT(rng, 5, 0.30, 0.6)
    p.genStarDir.x, p.genStarDir.y, p.genStarDir.z = starDir.x, starDir.y, starDir.z

    local self = TexGen.Cube {
        label  = 'Nebula.IFS',
        shader = shader,
        size   = res,
        format = TexFormat.RGBA16F,
        params = p,
        -- Nearest, like the LUTs always were: uploading a Tex1D's data resets its
        -- filters (the Linear ones ColorLUT sets before the upload never applied).
        inputs = {
            { lutR:view(), Samplers.Point },
            { lutG:view(), Samplers.Point },
            { lutB:view(), Samplers.Point },
        },
        mips   = true,
    }

    Profiler.End()
    return self
end

Generator.Add('Nebula', 1.0, generateNebulaIFS)
