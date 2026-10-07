local ffi = require('ffi')
local GenUtil = {}

--[[
--TODO: From Systems/Gen/GenUtil.lua Not Used Currently

-- Find a suitable point on the given mesh for mounting a module.
-- Normal gives the module's desired surface normal direction, while facing
-- gives the direction in which the module should be free of obstruction
function GenUtil.FindMountPoint(mesh, bsp, rng, normal, facing, maxTries)
    local radius = mesh:getRadius()
    local center = mesh:getCenter()
    --Log.Debug("@@@ GenUtil.FindMountPoint - radius = %s, center = %s", radius, center)
    local e2, e3 = Math.OrthoBasis(normal)
    local t = ffi.new('float[1]')
    for i = 1, maxTries do
        local ortho = rng:getDisc():scale(radius)
        local p1 = center + normal:scale(radius) + e2:scale(ortho.x) + e3:scale(ortho.y)
        local p2 = p1 - normal:scale(2.0 * radius)
        local ray = Ray(p1.x, p1.y, p1.z, p2.x - p1.x, p2.y - p1.y, p2.z - p1.z, 0, 1)
        if bsp:intersectRay(ray, t) then
            local p3 = ray:getPoint(t[0])
            local p4 = p3 + facing:scale(0.01)
            local p5 = p4 + facing:scale(radius)
            local ray2 = Ray(p4.x, p4.y, p4.z, p5.x - p4.x, p5.y - p4.y, p5.z - p4.z, 0, 1)
            if not bsp:intersectRay(ray2, t) then
                return p3
            end
        end
    end
    return nil
end
--]]

--- Write `value` into the `Params` field `p[name]`: numbers and booleans
--- directly, vectors component by component (`Vec3f` into a `vec3`, ...).
local function assignField(p, name, value)
    local t = type(value)
    if t == 'number' then
        p[name] = value
    elseif t == 'boolean' then
        p[name] = value and 1.0 or 0.0
    else
        local field = p[name]
        local components = ffi.sizeof(field) / 4 -- Vec2f/Vec3f/Vec4f
        field.x = value.x
        if components >= 2 then field.y = value.y end
        if components >= 3 then field.z = value.z end
        if components >= 4 then field.w = value.w end
    end
end

--- The `Params` struct of `shader` with `args` written into it. Arguments the
--- shader does not declare are ignored.
---@param shader Shader
---@param args table<string, number|boolean|Vec2f|Vec3f|Vec4f>|nil
---@return ffi.cdata*
local function buildParams(shader, args)
    local T = shader:blockType('Params')
    local p = T()
    for k, v in pairs(args or {}) do
        if ffi.offsetof(T, k) ~= nil then
            assignField(p, k, v)
        end
    end
    return p
end

-- A one texel black texture (alpha 1): what a sampler without a bound texture reads.
local blackTex
local function black()
    if not blackTex then
        blackTex = Tex2D.Create(1, 1, TexFormat.RGBA16F)
        blackTex:clear(0, 0, 0, 1)
    end
    return blackTex
end

--- Sampler inputs of generating shaders that declare any, by fragment shader.
--- `gen/moon` blends a photographic base texture (`baseMoonTex`) that is not
--- shipped; the sampler always read black, and still does.
local defaultInputs = {
    ['gen/moon'] = function() return { { black():view(), Samplers.Point } } end,
}

---Creates a Tex3D from a fullscreen generating shader (`TexGen.Volume`): one
---pass per z-slice over [-1,1]^3.
---@param fragShader string fragment shader name, e.g. 'sdf/asteroid'
---@param res integer
---@param fmt TexFormat
---@param args table<string, number|boolean|Vec2f|Vec3f|Vec4f>|nil `Params` fields
---@return Tex3D
function GenUtil.ShaderToTex3D(fragShader, res, fmt, args)
    local shader = Cache.Shader('fullscreen_ndc', fragShader)
    return TexGen.Volume {
        label  = 'GenUtil.ShaderToTex3D',
        shader = shader,
        size   = res,
        format = fmt,
        params = buildParams(shader, args),
    }
end

---Creates a TexCube from a fullscreen generating shader (`TexGen.Cube`),
---with its mip chain.
---@param res integer
---@param fmt TexFormat
---@param fragShader string fragment shader name, e.g. 'gen/planet'
---@param args table<string, number|boolean|Vec2f|Vec3f|Vec4f>|nil `Params` fields
---@return TexCube
function GenUtil.ShaderToTexCube(res, fmt, fragShader, args)
    Profiler.Begin('Gen.ShaderToTexCube')
    local shader = Cache.Shader('fullscreen_ndc', fragShader)
    local self = TexGen.Cube {
        label  = 'GenUtil.ShaderToTexCube',
        shader = shader,
        size   = res,
        format = fmt,
        params = buildParams(shader, args),
        mips   = true,
        inputs = defaultInputs[fragShader] and defaultInputs[fragShader](),
    }
    self:setMagFilter(TexFilter.Linear)
    self:setMinFilter(TexFilter.LinearMipLinear)
    Profiler.End()
    return self
end

GenUtil.buildParams = buildParams

return GenUtil
