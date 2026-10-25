local GenUtil = {}

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

-- Generation moved to Core.ECS.Mesh.Util.GenUtil (render API v2, S5): fullscreen
-- passes through `TexGen`. The ShaderState based `ShaderToTex3D` is gone.
function GenUtil.ShaderToTexCube(res, fmt, fragShader, args)
    return require('Core.ECS.Mesh.Util.GenUtil').ShaderToTexCube(res, fmt, fragShader, args)
end

return GenUtil
