--local istype = ffi.istype
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

---Creates a Tex3D from a Shader
---@param shaderState ShaderState
---@param res integer
---@param fmt TexFormat
---@return Tex3D
function GenUtil.ShaderToTex3D(shaderState, res, fmt)
    -- Create Tex3D, Resolution res, TexFormat fmt
    local tex3D = Tex3D.Create(res, res, res, fmt)
    -- Set RenderState to Default
    RenderState.PushAllDefaults()
    -- Start ShaderState and get it's Shader
    shaderState:start()
    local shader = shaderState:shader()
    -- Set Shader Uniforms
    shader:setFloat3('du', 2, 0, 0)
    shader:setFloat3('dv', 0, 2, 0)

    --TODO: Figure this out
    for i = 0, res - 1 do
        local z = (2.0 * (i / (res - 1)) - 1.0)
        shader:setFloat3('origin', -1, -1, z)
        RenderTarget.PushTex3D(tex3D, i)
        Draw.Rect(-1, -1, 2, 2)
        Draw.Flush()
        RenderTarget.Pop()
    end

    -- Stop ShaderState
    shaderState:stop()
    -- Return RenderState to previous state
    RenderState.PopAll()

    -- Return Tex3D
    return tex3D
end

--[[
--TODO: From Systems/Gen/GenUtil Not Currently Used

function GenUtil.ShaderToTexCube(res, fmt, fragShader, args)
    --Profiler.Begin('Gen.ShaderToTexCube')
    local shader = Cache.Shader('ui', fragShader)
    local state = ShaderState.Create(shader)
    for k, v in pairs(args) do
        local t = type(v)
        if t == 'number' then
            state:setFloat(k, v)
        elseif istype('Vec2f', v) or istype('Vec2d', v) then
            state:setFloat2(k, v.x, v.y)
        elseif istype('Vec3f', v) or istype('Vec3d', v) then
            state:setFloat3(k, v.x, v.y, v.z)
        elseif istype('Vec4f', v) or istype('Vec4d', v) then
            state:setFloat4(k, v.x, v.y, v.z, v.w)
        elseif istype('Tex1D', v) then
            state:setTex1D(k, v)
        elseif istype('Tex2D', v) then
            state:setTex2D(k, v)
        elseif istype('Tex3D', v) then
            state:setTex3D(k, v)
        elseif istype('TexCube', v) then
            state:setTexCube(k, v)
        elseif istype('Matrix', v) then
            state:setMatrix(k, v)
        else
            Log.Error('Argument <%s> has incompatible type', k)
        end
    end

    local self = TexCube.Create(res, fmt)
    self:generate(state)
    self:genMipmap()
    self:setMagFilter(TexFilter.Linear)
    self:setMinFilter(TexFilter.LinearMipLinear)
    --Profiler.End()
    return self
end
--]]
return GenUtil
