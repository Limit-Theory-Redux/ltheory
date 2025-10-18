local Entity = require('Legacy.GameObjects.Entity')
local SocketType = require('Legacy.GameObjects.Entities.Ship.SocketType')
local RenderComponent = require('Modules.Rendering.Components.RenderComponent')

local genColor = function(rng)
    local h = rng:getUniformRange(0, 0.5)
    local l = Math.Saturate(rng:getUniformRange(0.2, 0.3) + 0.05 * rng:getExp())
    local s = rng:getUniformRange(0.1, 0.3)
    local c = Color.FromHSL(h, s, l)
    return Vec3f(c.r, c.g, c.b)
end

local Planet = Subclass("Planet", Entity, function(self, seed)
    local rng = RNG.Create(seed)

    -- TODO: Improve planet size generation
    local planetSizeType = Config.gen.sizePlanet

    -- TODO : Had to lower quality to 2 because RigidBody is automatically
    -- building BSP, and sphere is pathological case for BSPs. Need
    -- generalized CollisionShape.
    local mesh = Gen.Primitive.IcoSphere(5)
    self:addRigidBody(true, mesh)

    -- TODO: Generate planetary mass based on type, size, and composition
    self:setMass(Config.gen.massPlanetTrue) -- TODO: change from Earth's actual mass value

    -- TODO: Replace with 0 - N colonies, each of which has its own distinct
    -- market/production/research capabilities
    self:addActions()
    self:addChildren()
    self:addDockable()     -- TODO: rethink how "docking with planets" should work
    self:addFlows()
    self:addMinable(false) -- TODO: should be 'true' temporarily (planets have Yield), but will change with Colonies
    self:addTrackable(true)

    self.mesh = mesh

    self.texSurface     = Gen.GenUtil.ShaderToTexCube(2048, TexFormat.RGBA16F, 'gen/planet', {
        seed = rng:getUniform(),
        freq = 4 + rng:getExp(),
        power = 1 + 0.5 * rng:getExp(),
        coef = (rng:getVec4(0.05, 1.00) ^ Vec4f(2, 2, 2, 2)):normalize()
    })

    self.cloudLevel     = rng:getUniformRange(-0.2, 0.15)
    self.oceanLevel     = rng:getUniform() ^ 1.5
    self.atmoScale      = 1.1

    self.color1         = genColor(rng)
    self.color2         = genColor(rng)
    self.color3         = genColor(rng)
    self.color4         = genColor(rng)
    
    local meshAtmo = Gen.Primitive.IcoSphere(5)
    meshAtmo:computeNormals()
    meshAtmo:invert()

    -- Add renderables
    local matSurface = Material.Create("material/planet")
    matSurface.state:setFloat('heightMult', 1.0)
    matSurface.state:setFloat('oceanLevel', self.oceanLevel)
    matSurface.state:setFloat3('color1', self.color1.x, self.color1.y, self.color1.z)
    matSurface.state:setFloat3('color2', self.color2.x, self.color2.y, self.color2.z)
    matSurface.state:setFloat3('color3', self.color3.x, self.color3.y, self.color3.z)
    matSurface.state:setFloat3('color4', self.color4.x, self.color4.y, self.color4.z)
    matSurface.state:setTexCube("surface", self.texSurface)
    matSurface.state:setFloat3('starColor', 1.0, 0.5, 0.1)
    local entity = self
    matSurface.onSetState = function(self, shader, body, eye)
        local origin = entity:getPos():relativeTo(eye)
        shader:setFloat3('origin', origin.x, origin.y, origin.z)
        shader:setFloat('rPlanet', body:getScale())
        shader:setFloat('rAtmo', body:getScale() * entity.atmoScale)
    end

    local matAtmo = Material.Create("material/atmosphere")
    matAtmo.blendMode = BlendMode.Alpha
    matAtmo.state:setFloat3('starColor', 1.0, 0.5, 0.1)
    matAtmo.onSetState = function(self, shader, body, eye)
        do -- TODO : Scale the atmosphere mesh in shader...
            local mScale = Matrix.Scaling(1.5, 1.5, 1.5)
            local mWorld = body:getToWorldMatrix(eye):product(mScale)
            local mWorldIT = mWorld:inverse()
            shader:setMatrix('mWorld', mWorld)
            shader:setMatrix('mWorldIT', mWorldIT)
        end

        local scale = body:getScale()
        shader:setFloat('rAtmo', scale * entity.atmoScale)
        shader:setFloat('rPlanet', scale)
        local pos = entity:getPos():relativeTo(eye)
        shader:setFloat3('origin', pos.x, pos.y, pos.z)
        shader:setFloat3('scale', scale, scale, scale)
    end
    matAtmo.onStart = function()
        RenderState.PushCullFace(CullFace.Back)
        RenderState.PushBlendMode(BlendMode.PreMultAlpha)
    end
    matAtmo.onStop = function()
        RenderState.PopCullFace()
        RenderState.PopBlendMode()
    end

    self.entity:add(RenderComponent({
        {mesh = mesh, material = matSurface},
        {mesh = meshAtmo, material = matAtmo},
    }))

    -- TEMP: give each planet the maximum number of every applicable component
    self.countCommo     = Config.gen.planetComponents[Enums.PlanetComponents.Commo][planetSizeType]
    self.countComputer  = Config.gen.planetComponents[Enums.PlanetComponents.Computer][planetSizeType]
    self.countSensor    = Config.gen.planetComponents[Enums.PlanetComponents.Sensor][planetSizeType]
    self.countInventory = Config.gen.planetComponents[Enums.PlanetComponents.Inventory][planetSizeType]
    self.countShield    = Config.gen.planetComponents[Enums.PlanetComponents.Shield][planetSizeType]

    self:addComponents()

    -- Add all the _positions_ for socketable components (the components are added later)
    self.positions = {
        [SocketType.Commo]     = {},
        [SocketType.Computer]  = {},
        [SocketType.Sensor]    = {},
        [SocketType.Inventory] = {},
        [SocketType.Shield]    = {},
    }

    -- Communicator sockets
    for i = 1, self.countCommo do
        insert(self.positions[SocketType.Commo], Vec3f(1, 1, 1))
    end

    -- Computer sockets
    for i = 1, self.countComputer do
        insert(self.positions[SocketType.Computer], Vec3f(1, 1, 1))
    end

    -- Sensor sockets
    for i = 1, self.countSensor do
        insert(self.positions[SocketType.Sensor], Vec3f(1, 1, 1))
    end

    -- Inventory sockets
    for i = 1, self.countInventory do
        insert(self.positions[SocketType.Inventory], Vec3f(1, 1, 1))
    end

    -- Shield sockets
    for i = 1, self.countShield do
        insert(self.positions[SocketType.Shield], Vec3f(1, 1, 1))
    end

    -- Add all sockets to parent
    -- TODO : Suggestive that JS-style prototype objects + 'clone' would work
    -- better for ShipType etc.
    self:addSockets()

    for type, elems in pairs(self.positions) do
        for i, pos in ipairs(elems) do
            self:addSocket(type, pos, true)
        end
    end

    self:setDrag(10, 10) -- fix planet in place
end)

return Planet
