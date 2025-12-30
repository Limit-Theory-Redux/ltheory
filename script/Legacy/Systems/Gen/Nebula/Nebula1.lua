local Generator = require('Legacy.Systems.Gen.Generator')
local ColorLUT = require('Legacy.Systems.Gen.ColorLUT')

local function generateNebulaIFS(rng, res, starDir)
    Profiler.Begin('Nebula.Generate.IFS')
    local self = TexCube.Create(res, TexFormat.RGBA16F)
    local shader = Cache.Shader('ui', 'gen/nebula')
    local ss = ShaderState.Create(shader)

    do -- Nebula color
        local h = rng:getUniform()
        local s = rng:getUniformRange(0.2, 0.8)
        local l = rng:getUniformRange(0.2, 0.8)
        local color = Color.FromHSL(h, s, l)
        local r = color.r
        local g = color.g
        local b = color.b
        ss:setFloat3('color', r, g, b)
    end

    ss:setFloat('brightnessScale', GameState.gen.nebulaBrightnessScale)

    local roughness = 0.65 + 0.05 * rng:getSign() * rng:getUniform() ^ 2
    ss:setFloat('roughness', roughness)

    local seed = rng:getUniformRange(1, 1000)
    ss:setFloat('seed', seed)

    local lutR = ColorLUT(rng, 5, 0.30, 0.6)
    local lutG = ColorLUT(rng, 5, 0.30, 0.6)
    local lutB = ColorLUT(rng, 5, 0.30, 0.6)
    ss:setTex1D('lutR', lutR)
    ss:setTex1D('lutG', lutG)
    ss:setTex1D('lutB', lutB)
    ss:setFloat3('genStarDir', starDir.x, starDir.y, starDir.z)

    self:generate(ss)
    self:genMipmap()
    self:setMagFilter(TexFilter.Linear)
    self:setMinFilter(TexFilter.LinearMipLinear)

    Profiler.End()
    return self
end

Generator.Add('Nebula', 1.0, generateNebulaIFS)
