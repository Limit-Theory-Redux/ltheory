local ffi = require('ffi')
local libphx = require('libphx').lib

-- `TexGen` fills cube maps and volumes with a fullscreen shader (see
-- doc/engine/render-api-v2.md, section 3e). The generating shader declares a
-- group-2 `Params` block whose first three members carry the face or slice
-- (render/gpu/gen.rs); the rest is `shader:blockType('Params')`.
--
--     local P = shader:blockType('Params')
--     local p = P()
--     p.seed, p.freq = s, f
--     local tex = TexGen.Cube { size = 2048, format = TexFormat.RGBA16F, shader = shader, params = p, mips = true }
--     local vol = TexGen.Volume { size = 96, format = TexFormat.R32F, shader = shader, params = p }
--
-- Fields: `shader` (required), `size`, `format`, `params` (a `Params` struct),
-- `inputs` (a list of `{ view, sampler }`, group-3 slots 0..3), `label`,
-- `mips` (cubes: generate the mip chain). `TexGen.CubeInto` fills an existing
-- cube: `TexGen.CubeInto(cube, { shader = ..., params = ..., inputs = ... })`.

local function describe(opts)
    local desc = GenDesc.Create(opts.shader)
    if opts.label then desc:label(opts.label) end
    local params = opts.params
    if params ~= nil then
        desc:params(ffi.cast('const uint8_t*', params), ffi.sizeof(params))
    end
    local inputs = opts.inputs
    if inputs then
        for slot, input in ipairs(inputs) do
            desc:input(slot - 1, input[1], input[2])
        end
    end
    return desc
end

function onDef_TexGen(t, mt)
    t.Cube = function(opts)
        local desc = describe(opts)
        local _instance = libphx.TexGen_Cube(Renderer, desc, opts.size, opts.format)
        local cube = Core.ManagedObject(_instance, libphx.TexCube_Free)
        if opts.mips then cube:genMipmap() end
        return cube
    end

    t.CubeInto = function(cube, opts)
        libphx.TexGen_CubeInto(Renderer, describe(opts), cube)
    end

    t.Volume = function(opts)
        local desc = describe(opts)
        local _instance = libphx.TexGen_Volume(Renderer, desc, opts.size, opts.format)
        return Core.ManagedObject(_instance, libphx.Tex3D_Free)
    end
end
