local Registry = require("Core.ECS.Registry")
local ShipEntity = require('Modules.Constructs.Entities.SpaceshipEntity')
local ShipComponents = require("Modules.Constructs.Components")
local Physics = require("Modules.Physics.Components")

-- Legacy Generator Imports
local ShipBasic = require('Legacy.Systems.Gen.ShipBasic')
local ShipCapital = require('Legacy.Systems.Gen.ShipCapital')
local ShipFighter = require('Legacy.Systems.Gen.ShipFighter')

local Materials = require("Shared.Registries.Materials")

---@class ShipGenerator
---@overload fun(): ShipGenerator
local ShipGenerator = Class("ShipGenerator", function()
    -- hello world
end)

---Generate mesh based on ship type
---@param seed integer
---@param shipType ShipType
---@param hull any
---@param res any
---@return Mesh
local function generateShipMesh(seed, shipType, hull, res)
    local rng = RNG.Create(seed)

    if shipType == Enums.ShipType.Fighter then
        Log.Debug("@@@ ShipGenerator.Fighter:(create) - hull = %s, res = %s", hull, res)

        local type = rng:choose({ 1, 2 })
        if type == 1 then
            Profiler.Begin('Gen.ShipFighter.Standard')
            local result = ShipFighter.Standard(rng, hull)
            Profiler.End()
            return result
        elseif type == 3 then
            Profiler.Begin('Gen.ShipFighter.Surreal')
            local result = ShipFighter.Surreal(rng, hull)
            Profiler.End()
            return result
        else
            Log.Debug("Ship type non-existent. Defaulting to Standard.")
            Profiler.Begin('Gen.ShipFighter.StandardDefault')
            local result = ShipFighter.Standard(rng, hull)
            Profiler.End()
            return result
        end
    elseif shipType == Enums.ShipType.Capital then
        Log.Debug("@@@ ShipGenerator.Capital:(create) - hull = %s, res = %s", hull, res)
        Profiler.Begin('Gen.ShipCapital')
        local result = ShipCapital.Sausage(rng, hull)
        Profiler.End()
        return result
    elseif shipType == Enums.ShipType.Basic then
        Log.Debug("@@@ ShipGenerator.Basic:(create) - hull = %s, res = %s", hull, res)
        Profiler.Begin('Gen.ShipBasic')
        local result = ShipBasic.Tube(rng, hull)
        Profiler.End()
        return result
    end

    Log.Warn("Unknown ship type: %s, defaulting to Fighter", shipType)
    return generateShipMesh(seed, Enums.ShipType.Fighter, hull, res)
end

---@class FighterGenConfig
---@field material Material|nil
---@field hull any
---@field res any
---@field position Vec3f|nil
---@field scale Vec3f|number|nil
---@field rotation Quat|nil
---@field isKinematic boolean|nil

---Create a Fighter ship with automatic mesh generation
---@param seed integer
---@param config FighterGenConfig|nil Configuration {material, hull, res, position, scale, rotation, isKinematic}
---@param stats ShipStats|nil
---@return Entity
function ShipGenerator:createFighter(seed, config, stats)
    config = config or {}

    local shipType = Enums.ShipType.Fighter

    -- Generate mesh
    local mesh = generateShipMesh(seed, shipType, config.hull, config.res)
    mesh:computeNormals()
    mesh:computeAO(0.3 * mesh:getRadius())

    -- Get or create material
    local material = config.material or (Materials.Metal and Materials.Metal())

    local meshes = { { mesh = mesh, material = material } }

    -- Create entity
    local entity = ShipEntity(seed, meshes, shipType, stats)

    -- Setup physics if configured
    if config.position or config.scale or config.rotation or config.isKinematic ~= nil then
        local rbCmp = entity:get(Physics.RigidBody)
        if rbCmp then
            local rb = RigidBody.CreateConvexDecompositionFromMesh(mesh)

            if config.isKinematic then
                rb:setKinematic(true)
            end

            if config.position then
                rb:setPos(config.position)
            end

            if config.scale then
                rb:setScale(config.scale)
            end

            if config.rotation then
                rb:setRot(config.rotation)
            end

            rbCmp:setRigidBody(rb)
        end
    end

    -- Set additional ship data
    local shipData = entity:get(ShipComponents.ShipData)
    if shipData then
        if shipType then
            shipData:setShipType(shipType)
        end
    end

    return entity
end

---Create a Capital ship with automatic mesh generation
---@param seed integer
---@param config table|nil Configuration {material, hull, res, position, scale, rotation, isKinematic}
---@param stats ShipStats|nil
---@return Entity
function ShipGenerator:createCapital(seed, config, stats)
    config = config or {}

    local shipType = Enums.ShipType.Capital

    -- Generate mesh
    local mesh = generateShipMesh(seed, shipType, config.hull, config.res)
    mesh:computeNormals()

    -- Get or create material
    local material = config.material or (Materials.Metal and Materials.Metal())

    local meshes = { { mesh = mesh, material = material } }

    -- Create entity
    local entity = ShipEntity(seed, meshes, shipType, stats)

    -- Setup physics if configured
    if config.position or config.scale or config.rotation or config.isKinematic ~= nil then
        local rbCmp = entity:get(Physics.RigidBody)
        if rbCmp then
            local rb = RigidBody.CreateConvexDecompositionFromMesh(mesh)

            if config.isKinematic then
                rb:setKinematic(true)
            end

            if config.position then
                rb:setPos(config.position)
            end

            if config.scale then
                rb:setScale(config.scale)
            end

            if config.rotation then
                rb:setRot(config.rotation)
            end

            rbCmp:setRigidBody(rb)
        end
    end

    -- Set additional ship data
    local shipData = entity:get(ShipComponents.ShipData)
    if shipData then
        if shipType then
            shipData:setShipType(shipType)
        end
    end

    return entity
end

---Create a Basic ship with automatic mesh generation
---@param seed integer
---@param config table|nil Configuration {material, hull, res, position, scale, rotation, isKinematic}
---@param stats ShipStats|nil
---@return Entity
function ShipGenerator:createBasic(seed, config, stats)
    config = config or {}

    local shipType = Enums.ShipType.Basic

    -- Generate mesh
    local mesh = generateShipMesh(seed, shipType, config.hull, config.res)
    mesh:computeNormals()

    -- Get or create material
    local material = config.material or (Materials.Metal and Materials.Metal())

    local meshes = { { mesh = mesh, material = material } }

    -- Create entity
    local entity = ShipEntity(seed, meshes, shipType, stats)

    -- Setup physics if configured
    if config.position or config.scale or config.rotation or config.isKinematic ~= nil then
        local rbCmp = entity:get(Physics.RigidBody)
        if rbCmp then
            local rb = RigidBody.CreateConvexDecompositionFromMesh(mesh)

            if config.isKinematic then
                rb:setKinematic(true)
            end

            if config.position then
                rb:setPos(config.position)
            end

            if config.scale then
                rb:setScale(config.scale)
            end

            if config.rotation then
                rb:setRot(config.rotation)
            end

            rbCmp:setRigidBody(rb)
        end
    end

    -- Set additional ship data
    local shipData = entity:get(ShipComponents.ShipData)
    if shipData then
        if shipType then
            shipData:setShipType(shipType)
        end
    end

    return entity
end

---Create a ship of random type
---@param seed integer
---@param config table|nil Configuration
---@return Entity
function ShipGenerator:createRandom(seed, config)
    local rng = RNG.Create(seed)
    local shipType = rng:choose({
        Enums.ShipType.Fighter,
        Enums.ShipType.Capital,
        Enums.ShipType.Basic
    })

    if shipType == Enums.ShipType.Fighter then
        return ShipGenerator:createFighter(seed, config)
    elseif shipType == Enums.ShipType.Capital then
        return ShipGenerator:createCapital(seed, config)
    else
        return ShipGenerator:createBasic(seed, config)
    end
end

return ShipGenerator()
