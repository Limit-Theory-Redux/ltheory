local Registry            = require("Core.ECS.Registry")
local CoreComponents      = require("Modules.Core.Components")
local PhysicsComponents   = require("Modules.Physics.Components")
local CelestialComponents = require("Modules.CelestialObjects.Components")
local RenderComp          = require("Modules.Rendering.Components").Render
local Materials           = require("Shared.Registries.Materials")

local Primitive           = require("Legacy.Systems.Gen.Primitive")
local GenUtil             = require("Legacy.Systems.Gen.GenUtil")
local AsteroidBeltRenderer = require("Modules.CelestialObjects.Systems.AsteroidBeltRenderer")
local SpatialComponents   = require("Modules.Spatial.Components")

---@class SolarSystemVisualizer
local SolarSystemVisualizer = {}

--- Walk the entity hierarchy and add visual representations (meshes, materials, rigid bodies)
--- to generated celestial entities that currently have no visual data.
---@param universe Entity The root universe entity from UniverseManager:createUniverse()
---@param physicsWorld Physics The physics world to add rigid bodies to
function SolarSystemVisualizer:materialize(universe, physicsWorld)
    self:_walkEntity(universe, physicsWorld)
end

---@param entity Entity
---@param physicsWorld Physics
function SolarSystemVisualizer:_walkEntity(entity, physicsWorld)
    local name = tostring(entity)

    if name:find("StarEntity") then
        self:_materializeStar(entity, physicsWorld)
    elseif name:find("PlanetEntity") then
        self:_materializePlanet(entity, physicsWorld)
    elseif name:find("MoonEntity") then
        self:_materializeMoon(entity, physicsWorld)
    elseif name:find("AsteroidBeltEntity") then
        self:_materializeAsteroidBelt(entity, physicsWorld)
    else
        -- Hide entities with empty RenderComponents (e.g. AsteroidRingEntity)
        -- to prevent the renderer from iterating nil meshes
        local renderCmp = entity:get(RenderComp)
        if renderCmp and not renderCmp:getMeshes() and not renderCmp:getRenderFn() then
            renderCmp:setVisible(false)
        end
    end

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

    local texRes = Config.game.solarSystemPlayable and Config.game.solarSystemPlayable.planetTexRes or 1024
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

    local texRes = Config.game.solarSystemPlayable and Config.game.solarSystemPlayable.moonTexRes or 1024
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
        -- Fallback: try Orbit component
        local orbitCmp = entity:get(SpatialComponents.Orbit)
        orbitRadius = orbitCmp and orbitCmp:getOrbitRadius() or 10000
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
    local count = math.min(100000, math.max(5000, math.floor(circumference * density * 0.05)))

    -- Generate proper SDF-based asteroid mesh with built-in LOD chain
    local GenerateAsteroidMesh = require("Core.ECS.Mesh.CelestialObjects.AsteroidMesh")
    local lodMesh = GenerateAsteroidMesh(seed)

    -- Generate asteroid positions
    local asteroids = AsteroidBeltRenderer.generateBeltAsteroids({
        orbitRadius = orbitRadius,
        width = width,
        count = count,
        inclination = math.rad(inclination),
        seed = seed,
        minScale = 50,
        maxScale = 500,
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

return SolarSystemVisualizer
