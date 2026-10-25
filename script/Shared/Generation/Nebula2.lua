local Generator = require('Shared.Generation.Generator')

local function generateNebulaLightTransport(rng, res, starDir)
    Profiler.Begin('Nebula.Generate.LightTransport')
    -- Either cube ends up as the result, so both get a mip chain.
    local buffDst = TexCube.Create(res, TexFormat.RGBA16F, { mips = true })
    local buffSrc = TexCube.Create(res, TexFormat.RGBA16F, { mips = true })
    buffSrc:clear(0.05, 0.05, 0.05, 0)

    local emit   = Cache.Shader('fullscreen_ndc', 'gen/nebula_emit')
    local absorb = Cache.Shader('fullscreen_ndc', 'gen/nebula_absorb')
    local EmitParams = emit:blockType('Params')
    local AbsorbParams = absorb:blockType('Params')

    for i = 1, 8 do
        for j = 1, rng:getInt(4, 8) do -- Emission
            local p = EmitParams()
            local rot = rng:getQuat()
            local dir = rng:getDir3()
            local T = rng:getUniform()
            local K = Math.Lerp(1600.0, 15000.0, T)
            local C = Color.FromTemperature(K, 2.5):toVec3():normalize():scale(1.0 + rng:getExp())
            p.color.x, p.color.y, p.color.z = C.x, C.y, C.z
            p.rot.x, p.rot.y, p.rot.z, p.rot.w = rot.x, rot.y, rot.z, rot.w
            TexGen.CubeInto(buffDst, {
                label  = 'Nebula.Emit',
                shader = emit,
                params = p,
                inputs = { { buffSrc:view(), Samplers.LinearClamp } },
            })
            buffSrc, buffDst = buffDst, buffSrc
        end

        for j = 1, rng:getInt(2, 4) do -- Extinction
            local p = AbsorbParams()
            local rot = rng:getQuat()
            p.density = 1.0 + rng:getExp()
            p.seed = rng:getUniform()
            p.rot.x, p.rot.y, p.rot.z, p.rot.w = rot.x, rot.y, rot.z, rot.w
            TexGen.CubeInto(buffDst, {
                label  = 'Nebula.Absorb',
                shader = absorb,
                params = p,
                inputs = { { buffSrc:view(), Samplers.LinearClamp } },
            })
            buffSrc, buffDst = buffDst, buffSrc
        end
    end

    buffSrc:genMipmap()
    Profiler.End()
    return buffSrc
end

Generator.Add('Nebula', 0.1, generateNebulaLightTransport)
