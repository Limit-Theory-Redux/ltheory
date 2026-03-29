local PhysicsComponents = require("Modules.Physics.Components")
local CoreComponents    = require("Modules.Core.Components")
local CameraManager     = require("Modules.Cameras.Managers.CameraManager")

--- AsteroidBeltRenderer — performant batch renderer for asteroid belts/rings.
--- Pre-generates asteroid transforms from belt parameters, then renders them
--- in a single shader setup with per-asteroid uniform updates.
--- Uses LodMesh for automatic distance-based LOD.
---@class AsteroidBeltRenderer
local AsteroidBeltRenderer = {}

--- Maximum render distance (squared) for individual asteroids
local MAX_RENDER_DIST_SQ = 4e12  -- ~2M units (matches LOD 7 max)

--- Generate asteroid transforms for a belt
---@param params table { orbitRadius, width, count, inclination, seed, minScale, maxScale }
---@return table asteroids Array of { px, py, pz, rotSeed, scale }
function AsteroidBeltRenderer.generateBeltAsteroids(params)
    local rng = RNG.Create(params.seed or 12345)
    local asteroids = {}

    local orbitRadius = params.orbitRadius
    local width = params.width or orbitRadius * 0.1
    local count = params.count or 500
    local inclination = params.inclination or 0
    local minScale = params.minScale or 10
    local maxScale = params.maxScale or 200

    for i = 1, count do
        local angle = rng:getUniform() * math.pi * 2

        -- Gaussian-ish radial distribution within belt width
        local radialOffset = (rng:getUniform() + rng:getUniform() - 1.0) * width * 0.5

        -- Vertical offset
        local vertOffset = (rng:getUniform() - 0.5) * width * 0.3
        vertOffset = vertOffset + math.sin(angle) * math.sin(inclination) * (orbitRadius + radialOffset)

        local r = orbitRadius + radialOffset
        local px = math.cos(angle) * r
        local py = vertOffset
        local pz = math.sin(angle) * r

        -- Bias toward smaller asteroids (cubic)
        local scale = minScale + (maxScale - minScale) * rng:getUniform() * rng:getUniform()

        local rotSeed = rng:get31()

        table.insert(asteroids, {
            px = px, py = py, pz = pz,
            rotSeed = rotSeed,
            scale = scale,
        })
    end

    return asteroids
end

--- Create a render function for a belt entity that batch-draws asteroids
---@param asteroidData table Array from generateBeltAsteroids
---@param lodMesh LodMesh The shared asteroid LOD mesh
---@return function renderFn
function AsteroidBeltRenderer.createRenderFn(asteroidData, lodMesh)
    local asteroid_shader = Cache.Shader('wvp', 'material/asteroid')
    local asteroid_tex = Cache.Texture('rock')
    local rotCache = {}

    -- Pre-compute rotations from seeds
    for i, a in ipairs(asteroidData) do
        local rng = RNG.Create(a.rotSeed)
        local ax = rng:getUniform() - 0.5
        local ay = rng:getUniform() - 0.5
        local az = rng:getUniform() - 0.5
        local len = math.sqrt(ax*ax + ay*ay + az*az)
        if len > 0.001 then ax, ay, az = ax/len, ay/len, az/len end
        local angle = rng:getUniform() * math.pi * 2
        rotCache[i] = Quat.FromAxisAngle(Vec3f(ax, ay, az), angle)
    end

    return function(entity, blendMode)
        if blendMode ~= BlendMode.Disabled then return end

        local eye = CameraManager:getEye()
        if not eye then return end

        -- Setup shader once for all asteroids
        asteroid_shader:start()
        asteroid_shader:setTex2D('texDiffuse', asteroid_tex)

        for i, a in ipairs(asteroidData) do
            -- Camera-relative position
            local rx = a.px - eye.x
            local ry = a.py - eye.y
            local rz = a.pz - eye.z
            local distSq = rx*rx + ry*ry + rz*rz

            -- Distance culling; skip if spawned as entity
            if distSq < MAX_RENDER_DIST_SQ and not a.spawned then
                local relPos = Vec3f(rx, ry, rz)
                local mat = Matrix.FromPosRotScale(relPos, rotCache[i], a.scale)
                local matIT = mat:inverse()

                -- Set uniforms directly on the bound shader (not autovar stack)
                asteroid_shader:setMatrix('mWorld', mat)
                asteroid_shader:setMatrixT('mWorldIT', matIT)
                asteroid_shader:setFloat('scale', a.scale)

                -- Draw with LOD based on distance
                lodMesh:draw(distSq)
            end
        end

        asteroid_shader:stop()
    end
end

return AsteroidBeltRenderer
