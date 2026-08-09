local CameraManager     = require("Modules.Cameras.Managers.CameraManager")

--- AsteroidBeltRenderer — performant batch renderer for asteroid belts/rings.
---@class AsteroidBeltRenderer
local AsteroidBeltRenderer = {}

--- Maximum render distance (squared)
local MAX_RENDER_DIST_SQ = 4e12
--- Maximum drawn asteroids per frame per belt/ring
local MAX_DRAWN_PER_FRAME = 200
--- Render-distance override (benchmark: camera orbits far outside the belt).
--- nil = derive from belt spread (game default).
local benchRenderDistSq = nil

--- Allow benchmarks/tests to raise the per-frame draw cap (module-local).
---@param n number
function AsteroidBeltRenderer.setMaxDrawnPerFrame(n)
    MAX_DRAWN_PER_FRAME = n or 200
end

--- Allow benchmarks/tests to override the render-distance cutoff.
---@param distSq number Squared distance; asteroids beyond this are not drawn
function AsteroidBeltRenderer.setRenderDistSq(distSq)
    benchRenderDistSq = distSq
end

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
        local radialOffset = (rng:getUniform() + rng:getUniform() - 1.0) * width * 0.5
        local vertOffset = (rng:getUniform() - 0.5) * width * 0.3
        vertOffset = vertOffset + math.sin(angle) * math.sin(inclination) * (orbitRadius + radialOffset)

        local r = orbitRadius + radialOffset
        local px = math.cos(angle) * r
        local py = vertOffset
        local pz = math.sin(angle) * r

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

--- Create a render function for a belt entity
---@param asteroidData table
---@param lodMesh LodMesh
---@return function renderFn
function AsteroidBeltRenderer.createRenderFn(asteroidData, lodMesh)
    local asteroid_shader = Cache.Shader('wvp', 'material/asteroid')
    local asteroid_tex = Cache.Texture('rock')

    -- Lazy rotation cache — only compute for asteroids we actually render
    local rotCache = {}

    -- Reusable Vec3f
    local relPos = Vec3f(0, 0, 0)

    -- Render distance proportional to belt spread (capped), or the
    -- benchmark override when set (camera orbits far outside the belt)
    local renderDistSq = benchRenderDistSq
    if not renderDistSq then
        local maxOrbitR = 0
        for i = 1, math.min(100, #asteroidData) do
            local a = asteroidData[i]
            local r = math.sqrt(a.px * a.px + a.pz * a.pz)
            if r > maxOrbitR then maxOrbitR = r end
        end
        renderDistSq = math.min(MAX_RENDER_DIST_SQ, (maxOrbitR * 3) ^ 2)
    end

    local PhysicsComponents = require("Modules.Physics.Components")

    return function(entity, blendMode)
        if blendMode ~= BlendMode.Disabled then return end
        if not lodMesh then return end

        local eye = CameraManager:getEye()
        if not eye then return end

        -- Get entity's world position (for rings: planet position)
        local entPosX, entPosY, entPosZ = 0, 0, 0
        local transform = entity:get(PhysicsComponents.Transform)
        if transform then
            local p = transform:getPos()
            entPosX, entPosY, entPosZ = p.x, p.y, p.z
        end

        local eyeX, eyeY, eyeZ = eye.x, eye.y, eye.z

        asteroid_shader:start()
        asteroid_shader:setTex2D('texDiffuse', asteroid_tex)

        local drawn = 0
        for i, a in ipairs(asteroidData) do
            if drawn >= MAX_DRAWN_PER_FRAME then break end
            if a.spawned then goto next_asteroid end

            -- Asteroid world pos = entity pos + local asteroid offset
            local rx = entPosX + a.px - eyeX
            local ry = entPosY + a.py - eyeY
            local rz = entPosZ + a.pz - eyeZ
            local distSq = rx * rx + ry * ry + rz * rz

            if distSq < renderDistSq then
                -- Lazy rotation: compute once, cache forever
                local rot = rotCache[i]
                if not rot then
                    local rng = RNG.Create(a.rotSeed)
                    local ax = rng:getUniform() - 0.5
                    local ay = rng:getUniform() - 0.5
                    local az = rng:getUniform() - 0.5
                    local len = math.sqrt(ax * ax + ay * ay + az * az)
                    if len > 0.001 then ax, ay, az = ax / len, ay / len, az / len end
                    rot = Quat.FromAxisAngle(Vec3f(ax, ay, az), rng:getUniform() * math.pi * 2)
                    rotCache[i] = rot
                end

                relPos.x = rx; relPos.y = ry; relPos.z = rz
                local mat = Matrix.FromPosRotScale(relPos, rot, a.scale)

                asteroid_shader:setMatrix('mWorld', mat)
                asteroid_shader:setFloat('scale', a.scale)
                lodMesh:draw(distSq)

                drawn = drawn + 1
            end

            ::next_asteroid::
        end

        asteroid_shader:stop()
    end
end

return AsteroidBeltRenderer
