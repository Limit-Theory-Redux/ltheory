local Registry = require("Core.ECS.Registry")
local PhysicsComponents = require("Modules.Physics.Components")
local ConstructComponents = require("Modules.Constructs.Components")
local ConstructionScope = require("Modules.Constructs.Managers.ConstructionScope")
local ShipGenerator = require("Modules.Constructs.Managers.Generators.ShipGenerator")
local TargetGenerator = require("Modules.Constructs.Managers.Generators.TargetGenerator")

---@class ConstructManager
---@overload fun(registry?: Registry, physicsWorld?: table): ConstructManager
local ConstructManager = Class("ConstructManager", function(self, registry, physicsWorld)
    self.registry = registry or Registry
    self.physicsWorld = physicsWorld
    self.handles = {}
end)

local function isKnownShipType(shipType)
    return shipType == Enums.ShipType.Fighter
        or shipType == Enums.ShipType.Capital
        or shipType == Enums.ShipType.Basic
end

---@param args table {seed, shipType, config, stats}
---@return table ConstructHandle
function ConstructManager:createShip(args)
    assert(type(args) == "table", "ship construction requires an argument table")
    assert(args.seed ~= nil, "ship construction requires a seed")
    assert(isKnownShipType(args.shipType),
        "ship construction requires a known Enums.ShipType value")
    local scope = ConstructionScope(self.registry, self.physicsWorld)
    local ok, result = xpcall(function()
        local root = ShipGenerator:create(args.seed, args.shipType, args.config, args.stats)
        local bodyComponent = root:get(PhysicsComponents.RigidBody)
        assert(bodyComponent and bodyComponent:getRigidBody(),
            "ship generator produced no rigid body")
        local shipData = root:get(ConstructComponents.ShipData)
        assert(shipData and shipData:getGeneratedMesh(),
            "ship generator produced no generated mesh")
        scope:trackEntity(root, true)
        scope:addRigidBody(bodyComponent:getRigidBody())
        return scope:commit({
            root = root,
            bodyComponent = bodyComponent,
            rigidBody = bodyComponent:getRigidBody(),
            shipData = shipData,
            generatedMesh = shipData:getGeneratedMesh(),
            structuralSockets = shipData:getGeneratedMountSockets(),
            shipType = args.shipType,
            seed = args.seed,
        })
    end, debug.traceback)
    if not ok then
        scope:rollback()
        error(result, 0)
    end
    table.insert(self.handles, result)
    return result
end

---@param args table {seed, shipType, position, scale, ...}
---@return table ConstructHandle
function ConstructManager:createTarget(args)
    assert(type(args) == "table", "target construction requires an argument table")
    assert(args.seed ~= nil, "target construction requires a seed")
    assert(isKnownShipType(args.shipType),
        "target construction requires a known Enums.ShipType value")
    local scope = ConstructionScope(self.registry, self.physicsWorld)
    local ok, result = xpcall(function()
        local target = TargetGenerator:create(args.seed, args)
        scope:trackEntity(target.entity, true)
        scope:addRigidBody(target.body)
        return scope:commit({
            root = target.entity,
            bodyComponent = target.entity:get(PhysicsComponents.RigidBody),
            rigidBody = target.body,
            shipData = target.shipData,
            generatedMesh = target.shipData:getGeneratedMesh(),
            structuralSockets = target.shipData:getGeneratedMountSockets(),
            health = target.health,
            radius = target.radius,
            targetPointSeed = target.targetPointSeed,
            targetable = target.entity:get(ConstructComponents.Targetable),
            shipType = args.shipType,
            seed = args.seed,
        })
    end, debug.traceback)
    if not ok then
        scope:rollback()
        error(result, 0)
    end
    table.insert(self.handles, result)
    return result
end

function ConstructManager:destroy(handle)
    if not handle then
        return
    end
    if handle.scope then
        handle.scope:destroy()
    end
    for index = #self.handles, 1, -1 do
        if self.handles[index] == handle then
            table.remove(self.handles, index)
            break
        end
    end
end

function ConstructManager:destroyAll()
    for index = #self.handles, 1, -1 do
        local handle = self.handles[index]
        if handle.scope then
            handle.scope:destroy()
        end
        self.handles[index] = nil
    end
end

return ConstructManager
