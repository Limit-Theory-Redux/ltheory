local Registry            = require("Core.ECS.Registry")
local Entity              = require("Core.ECS.Entity")
local CoreComponents      = require("Modules.Core.Components")
local PhysicsComponents   = require("Modules.Physics.Components")
local CelestialComponents = require("Modules.CelestialObjects.Components")
local RenderComp          = require("Modules.Rendering.Components").Render
local Materials           = require("Shared.Registries.Materials")

local Primitive           = require("Legacy.Systems.Gen.Primitive")
local GenUtil             = require("Legacy.Systems.Gen.GenUtil")
local AsteroidBeltRenderer = require("Modules.CelestialObjects.Systems.AsteroidBeltRenderer")
local AsteroidMeshPool    = require("Modules.CelestialObjects.Systems.AsteroidMeshPool")
local SpatialComponents   = require("Modules.Spatial.Components")

---@class SolarSystemVisualizer
local SolarSystemVisualizer = {}

--- Walk the entity hierarchy and add visual representations (meshes, materials, rigid bodies)
--- to generated celestial entities that currently have no visual data.
---@param universe Entity The root universe entity from UniverseManager:createUniverse()
---@param physicsWorld Physics The physics world to add rigid bodies to
function SolarSystemVisualizer:materialize(universe, physicsWorld)
    -- Pre-load asteroid mesh pool (loads from cache or generates once)
    AsteroidMeshPool:init(8, 42)
    self:_walkEntity(universe, physicsWorld)
end

---@param entity Entity
---@param physicsWorld Physics
function SolarSystemVisualizer:_walkEntity(entity, physicsWorld)
    local name = tostring(entity)

    -- Belt/ring entities that already carry a render fn (e.g. the rocks
    -- emitted by _materializeAsteroidRing) must not be re-materialized by
    -- the walk when it recurses into the ring's children - that would
    -- generate a SECOND, default-radius belt over the real one.
    local existingRender = entity:get(RenderComp)
    if existingRender and existingRender:getRenderFn() then
        existingRender:setVisible(true)
        goto recurse_children
    end

    if name:find("StarEntity") then
        self:_materializeStar(entity, physicsWorld)
    elseif name:find("PlanetEntity") then
        self:_materializePlanet(entity, physicsWorld)
    elseif name:find("MoonEntity") then
        self:_materializeMoon(entity, physicsWorld)
    elseif name:find("AsteroidBeltEntity") then
        self:_materializeAsteroidBelt(entity, physicsWorld)
    elseif name:find("AsteroidRingEntity") then
        self:_materializeAsteroidRing(entity, physicsWorld)
    else
        -- Hide entities with empty RenderComponents (e.g. AsteroidRingEntity)
        -- to prevent the renderer from iterating nil meshes
        local renderCmp = entity:get(RenderComp)
        if renderCmp and not renderCmp:getMeshes() and not renderCmp:getRenderFn() then
            renderCmp:setVisible(false)
        end
    end

    ::recurse_children::
    -- Recurse into children
    local childrenCmp = entity:get(CoreComponents.Children)
    if childrenCmp then
        for child in childrenCmp:iterChildren() do
            self:_walkEntity(child, physicsWorld)
        end
    end
end

---@param entity Entity
---@param physicsWorld Physics
function SolarSystemVisualizer:_materializeStar(entity, physicsWorld)
    local mesh = Primitive.IcoSphere(5)

    -- Animated star material with sun texture
    local material = Materials.Star()

    local renderCmp = entity:get(RenderComp)
    if renderCmp then
        renderCmp:setMeshes({ { mesh = mesh, material = material } })
    else
        entity:add(RenderComp({ { mesh = mesh, material = material } }))
    end

    -- Create kinematic rigid body for raycasting/interaction
    local rbCmp = entity:get(PhysicsComponents.RigidBody)
    if not rbCmp then
        rbCmp = PhysicsComponents.RigidBody()
        entity:add(rbCmp)
    end

    local transform = entity:get(PhysicsComponents.Transform)
    local rb = RigidBody.CreateSphereFromMesh(mesh)
    rb:setKinematic(true)
    rb:setPos(transform:getPos())
    rb:setScale(transform:getScale())
    rbCmp:setRigidBody(rb)
    physicsWorld:addRigidBody(rb)
end

---@param entity Entity
---@param physicsWorld Physics
function SolarSystemVisualizer:_materializePlanet(entity, physicsWorld)
    local seedCmp = entity:get(CoreComponents.Seed)
    local seed = seedCmp and seedCmp:getSeed() or 0
    local planetRNG = RNG.Create(seed)

    local mesh = Primitive.IcoSphere(5)
    local meshAtmo = Primitive.IcoSphere(5, 1.5)
    meshAtmo:computeNormals()
    meshAtmo:invert()

    -- Generate procedural planet colors
    local genColor = function(rng)
        local h = rng:getUniformRange(0, 0.5)
        local l = Math.Saturate(rng:getUniformRange(0.2, 0.3) + 0.05 * rng:getExp())
        local s = rng:getUniformRange(0.1, 0.3)
        local c = Color.FromHSL(h, s, l)
        return Vec3f(c.r, c.g, c.b)
    end

    local genOptions = {
        surfaceFreq  = 4 + planetRNG:getExp(),
        surfacePower = 1 + 0.5 * planetRNG:getExp(),
        surfaceCoef  = (planetRNG:getVec4(0.05, 1.00) ^ Vec4f(2, 2, 2, 2)):normalize(),
        color1       = genColor(planetRNG),
        color2       = genColor(planetRNG),
        color3       = genColor(planetRNG),
        color4       = genColor(planetRNG),
        oceanLevel   = planetRNG:getUniform() ^ 1.5,
        cloudLevel   = planetRNG:getUniformRange(-0.2, 0.15),
        atmoScale    = 1.1,
    }

    local texRes = Config.game.planetTexRes or 1024
    local texSurface = GenUtil.ShaderToTexCube(texRes, TexFormat.RGBA16F, 'gen/planet', {
        seed  = planetRNG:getUniform(),
        freq  = genOptions.surfaceFreq,
        power = genOptions.surfacePower,
        coef  = genOptions.surfaceCoef
    })

    local matPlanet = Materials.PlanetSurface()
    local matAtmo = Materials.PlanetAtmosphere()
    matPlanet:setTexture("surface", texSurface)

    -- Add gen component for shader var lookups
    local planetGenCmp = CelestialComponents.Gen.Planet(genOptions)
    entity:add(planetGenCmp)

    -- Add cloud motion simulation component if not present
    if not entity:get(CelestialComponents.Simulation.CloudMotion) then
        entity:add(CelestialComponents.Simulation.CloudMotion(1, 1))
    end

    -- Set meshes on existing RenderComponent or add one
    local renderCmp = entity:get(RenderComp)
    if renderCmp then
        renderCmp:setMeshes({
            { mesh = mesh,     material = matPlanet },
            { mesh = meshAtmo, material = matAtmo },
        })
    else
        entity:add(RenderComp({
            { mesh = mesh,     material = matPlanet },
            { mesh = meshAtmo, material = matAtmo },
        }))
    end

    -- Setup physics rigid body
    local rbCmp = entity:get(PhysicsComponents.RigidBody)
    if not rbCmp then
        rbCmp = PhysicsComponents.RigidBody()
        entity:add(rbCmp)
    end

    local transform = entity:get(PhysicsComponents.Transform)
    local rb = RigidBody.CreateSphereFromMesh(mesh)
    rb:setKinematic(true)
    rb:setPos(transform:getPos())
    rb:setScale(transform:getScale())
    rbCmp:setRigidBody(rb)
    physicsWorld:addRigidBody(rb)
end

---@param entity Entity
---@param physicsWorld Physics
function SolarSystemVisualizer:_materializeMoon(entity, physicsWorld)
    local seedCmp = entity:get(CoreComponents.Seed)
    local seed = seedCmp and seedCmp:getSeed() or 0
    local moonRNG = RNG.Create(seed)

    local mesh = Primitive.IcoSphere(4)

    -- Moon color palette
    local bodies = {
        { highland = Vec3f(0.72, 0.72, 0.72), maria = Vec3f(0.25, 0.25, 0.25) },
        { highland = Vec3f(0.74, 0.72, 0.68), maria = Vec3f(0.28, 0.27, 0.24) },
        { highland = Vec3f(0.76, 0.74, 0.70), maria = Vec3f(0.30, 0.28, 0.25) },
    }
    local body = moonRNG:choose(bodies)

    local function perturbColor(color, rng, amount)
        return Vec3f(
            Math.Clamp(color.x + rng:getUniformRange(-amount, amount), 0, 1),
            Math.Clamp(color.y + rng:getUniformRange(-amount, amount), 0, 1),
            Math.Clamp(color.z + rng:getUniformRange(-amount, amount), 0, 1)
        )
    end

    local moonOptions = {
        craterDensity     = 0.1,
        craterSharpness   = 0.47,
        mariaAmount       = 0.45,
        mountainHeight    = 1.0,
        mountainScale     = 1.0,
        proceduralBlend   = 0.85,
        brightRayStrength = 0.40,
        highlandColor     = perturbColor(body.highland, moonRNG, moonRNG:getUniformRange(0.002, 0.04)),
        mariaColor        = perturbColor(body.maria, moonRNG, moonRNG:getUniformRange(0.002, 0.06)),
        heightMult        = 0.045,
        enableAtmosphere  = false,
    }

    local texRes = Config.game.moonTexRes or 1024
    local texSurface = GenUtil.ShaderToTexCube(texRes, TexFormat.RGBA16F, 'gen/moon', {
        seed              = moonRNG:getUniform(),
        craterDensity     = moonOptions.craterDensity,
        craterSharpness   = moonOptions.craterSharpness,
        mariaAmount       = moonOptions.mariaAmount,
        mountainHeight    = moonOptions.mountainHeight,
        mountainScale     = moonOptions.mountainScale,
        proceduralBlend   = moonOptions.proceduralBlend,
        brightRayStrength = moonOptions.brightRayStrength,
    })
    texSurface:genMipmap()
    texSurface:setMagFilter(TexFilter.Linear)
    texSurface:setMinFilter(TexFilter.LinearMipLinear)

    -- Add gen component for shader var lookups
    entity:add(CelestialComponents.Gen.Moon(moonOptions))

    local matMoon = Materials.MoonSurface()
    matMoon:setTexture("surface", texSurface)

    local renderCmp = entity:get(RenderComp)
    if renderCmp then
        renderCmp:setMeshes({ { mesh = mesh, material = matMoon } })
    else
        entity:add(RenderComp({ { mesh = mesh, material = matMoon } }))
    end

    -- Setup physics rigid body
    local rbCmp = entity:get(PhysicsComponents.RigidBody)
    if not rbCmp then
        rbCmp = PhysicsComponents.RigidBody()
        entity:add(rbCmp)
    end

    local transform = entity:get(PhysicsComponents.Transform)
    local rb = RigidBody.CreateSphereFromMesh(mesh)
    rb:setKinematic(true)
    rb:setPos(transform:getPos())
    rb:setScale(transform:getScale())
    rbCmp:setRigidBody(rb)
    physicsWorld:addRigidBody(rb)
end

--- Materialize an asteroid belt with batch rendering
---@param entity Entity
---@param physicsWorld Physics
function SolarSystemVisualizer:_materializeAsteroidBelt(entity, physicsWorld)
    local seedCmp = entity:get(CoreComponents.Seed)
    local seed = seedCmp and seedCmp:getSeed() or 12345
    local transform = entity:get(PhysicsComponents.Transform)

    -- Compute orbit radius from belt position (rebased = relative to star at origin)
    local pos = transform and transform:getPos() or Position(0, 0, 0)
    local orbitRadius = math.sqrt(pos.x * pos.x + pos.y * pos.y + pos.z * pos.z)
    if orbitRadius < 100 then
        local orbitCmp = entity:get(SpatialComponents.Orbit)
        orbitRadius = orbitCmp and orbitCmp:getOrbitRadius() or 10000
    end

    -- Belt asteroids are generated in absolute world space (centered at star origin)
    -- Reset entity position to (0,0,0) so the renderer's offset doesn't double them
    if transform then
        transform:setPos(Position(0, 0, 0))
    end

    -- Width from spatial component
    local widthCmp = entity:get(SpatialComponents.Width)
    local width = widthCmp and widthCmp:getWidth() or (orbitRadius * 0.1)

    -- Inclination
    local inclCmp = entity:get(SpatialComponents.Inclination)
    local inclination = inclCmp and inclCmp:getInclination() or 0

    -- Density from component
    local densityCmp = entity:get(CelestialComponents.Density)
    local density = densityCmp and densityCmp:getDensity() or 0.5

    -- Asteroid count: proportional to belt circumference and density
    local circumference = 2 * math.pi * orbitRadius
    local count = math.min(20000, math.max(2000, math.floor(circumference * density * 0.01)))

    -- Get shared asteroid mesh from pool (no per-belt generation)
    local lodMesh = AsteroidMeshPool:getFromSeed(seed)

    -- Generate asteroid positions
    local asteroids = AsteroidBeltRenderer.generateBeltAsteroids({
        orbitRadius = orbitRadius,
        width = width,
        count = count,
        inclination = math.rad(inclination),
        seed = seed,
        minScale = 0.1,   -- ~1km asteroid
        maxScale = 5.0,   -- ~50km asteroid
    })

    Log.Info("Materialized asteroid belt: %d asteroids, orbit=%.0f, width=%.0f",
        count, orbitRadius, width)

    -- Store asteroid data + mesh as a component for other systems (spawning, map, labels)
    entity:add(CelestialComponents.AsteroidBelt(asteroids, orbitRadius, width, lodMesh))

    -- Create render component with custom batch render function
    local renderCmp = entity:get(RenderComp)
    if not renderCmp then
        renderCmp = RenderComp()
        entity:add(renderCmp)
    end
    renderCmp:setRenderFn(AsteroidBeltRenderer.createRenderFn(asteroids, lodMesh))
    renderCmp:setVisible(true)
end

--- Materialize a planetary asteroid ring (same logic as belt but smaller scale)
---@param entity Entity
---@param physicsWorld Physics
function SolarSystemVisualizer:_materializeAsteroidRing(entity, physicsWorld)
    local seedCmp = entity:get(CoreComponents.Seed)
    local seed = seedCmp and seedCmp:getSeed() or 54321
    local transform = entity:get(PhysicsComponents.Transform)

    -- Ring orbit radius: use parent planet scale * multiplier
    -- (ring has no Orbit component to avoid OrbitalSystem moving it)
    local parentCmp = entity:get(CoreComponents.Parent)
    local parentEntity = parentCmp and parentCmp:getParent() or nil
    local planetScale = 1000
    if parentEntity then
        local pTransform = parentEntity:get(PhysicsComponents.Transform)
        if pTransform then planetScale = pTransform:getScale() end
    end
    -- Deterministic ring radius from seed
    local ringRng = RNG.Create(seed + 777)
    local orbitRadius = planetScale * (2.0 + ringRng:getUniform() * 3.0)

    local widthCmp = entity:get(SpatialComponents.Width)
    local width = widthCmp and widthCmp:getWidth() or (orbitRadius * 0.2)
    -- Cap width
    width = math.min(width, orbitRadius * 0.3)

    local inclCmp = entity:get(SpatialComponents.Inclination)
    local inclination = inclCmp and inclCmp:getInclination() or 0
    local tiltRad = math.rad(inclination)

    local densityCmp = entity:get(CelestialComponents.Density)
    local density = densityCmp and densityCmp:getDensity() or 0.5

    -- The ring entity is the SHADER BAND: a flat annulus rendered with
    -- the procedural PlanetRing material (same as PlanetTest's ring).
    -- The individual ROCKS are a separate concern - they belong to the
    -- asteroid BELT system (AsteroidBeltEntity + AsteroidBeltRenderer),
    -- which the generator emits as its own entity.
    local innerRadius = orbitRadius - width * 0.5
    local outerRadius = orbitRadius + width * 0.5
    local mesh = Primitive.Ring(innerRadius, outerRadius, 128)

    local matRing = Materials.PlanetRing()
    matRing:setTexture("ringTex", Tex2D.Create(512, 512, TexFormat.RGBA8), Enums.UniformType.Tex2D)
    matRing:addStaticShaderVar("rMin", Enums.UniformType.Float, function() return innerRadius end)
    matRing:addStaticShaderVar("rMax", Enums.UniformType.Float, function() return outerRadius end)
    matRing:addStaticShaderVar("ringHeight", Enums.UniformType.Float, function() return 50 end)
    matRing:addStaticShaderVar("rotationSpeed", Enums.UniformType.Float, function() return 2.0 end)
    matRing:addStaticShaderVar("twistFactor", Enums.UniformType.Float, function() return 0.25 end)
    matRing:addStaticShaderVar("enableDebug", Enums.UniformType.Int, function() return 0 end)
    matRing:addStaticShaderVar("debugMode", Enums.UniformType.Int, function() return 0 end)

    local renderCmp = entity:get(RenderComp)
    if not renderCmp then
        renderCmp = RenderComp()
        entity:add(renderCmp)
    end
    renderCmp:setMeshes({ { mesh = mesh, material = matRing } })
    renderCmp:setVisible(true)

    -- Tilt the band against the planet axis (matching the generated
    -- inclination). The band is attached to the planet; when the planet
    -- orbits, OrbitalSystem's follower pass keeps the ring transform at
    -- the planet's current position.
    local rbCmp = entity:get(PhysicsComponents.RigidBody)
    if not rbCmp then
        rbCmp = PhysicsComponents.RigidBody()
        entity:add(rbCmp)
    end
    local rb = RigidBody.CreateSphere()
    rb:setKinematic(true)
    rb:setPos(transform:getPos())
    rb:setRot(Quat.FromAxisAngle(Vec3f(1, 0, 0), tiltRad))
    rbCmp:setRigidBody(rb)
    if physicsWorld then
        physicsWorld:addRigidBody(rb)
    end

    -- ROCKS: the ring band is populated by the asteroid BELT system
    -- (AsteroidBeltEntity + AsteroidBeltRenderer), the same system that
    -- renders star-system belts. The belt entity is a child of the ring
    -- so it shares the ring's parent-tracking (follows the planet), and
    -- its render origin resolves through that parent to the planet's
    -- current position.
    local beltEntity = Entity.Create("AsteroidBeltEntity", CoreComponents.Seed(seed + 99))
    local beltTransform = beltEntity:get(PhysicsComponents.Transform)
    if beltTransform then beltTransform:setPos(transform:getPos()) end
    local lodMesh = AsteroidMeshPool:getFromSeed(seed)
    local beltCount = math.min(2000, math.max(200, math.floor(density * 2000)))
    local asteroids = AsteroidBeltRenderer.generateBeltAsteroids({
        orbitRadius = orbitRadius,
        width = width,
        count = beltCount,
        inclination = tiltRad,
        seed = seed,
        minScale = 0.1,  -- ~1km rock
        maxScale = 5.0,  -- ~50km rock
    })
    beltEntity:add(CelestialComponents.AsteroidBelt(asteroids, orbitRadius, width, lodMesh))
    local beltRenderCmp = beltEntity:get(RenderComp)
    if not beltRenderCmp then
        beltRenderCmp = RenderComp()
        beltEntity:add(beltRenderCmp)
    end
    beltRenderCmp:setRenderFn(AsteroidBeltRenderer.createRenderFn(asteroids, lodMesh))
    beltRenderCmp:setVisible(true)
    Registry:attachEntity(entity, beltEntity)

    Log.Info("Materialized planet ring: orbit=%.0f, width=%.0f, tilt=%.1f deg, %d rocks",
        orbitRadius, width, inclination, beltCount)
end

return SolarSystemVisualizer
