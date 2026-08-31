local Registry = require("Core.ECS.Registry")
local CoreComponents = require("Modules.Core.Components")

---@class ConstructionScope
---@overload fun(registry?: Registry, physicsWorld?: table): ConstructionScope
local ConstructionScope = Class("ConstructionScope", function(self, registry, physicsWorld)
    self.registry = registry or Registry
    self.physicsWorld = physicsWorld
    self.entities = {}
    self.entityIds = {}
    self.rigidBodies = {}
    self.rigidBodyIds = {}
    self.root = nil
    self.closed = false
end)

function ConstructionScope:trackEntity(entity, isRoot)
    assert(entity and entity.id, "construction scope requires an entity")
    if not self.entityIds[entity.id] then
        self.entityIds[entity.id] = true
        table.insert(self.entities, entity)
    end
    if isRoot or not self.root then
        self.root = entity
    end
    return entity
end

function ConstructionScope:trackRigidBody(body, world)
    assert(body, "construction scope requires a rigid body")
    if not self.rigidBodyIds[body] then
        self.rigidBodyIds[body] = true
        table.insert(self.rigidBodies, {
            body = body,
            world = world or self.physicsWorld,
        })
    end
    return body
end

function ConstructionScope:addRigidBody(body, world)
    local targetWorld = world or self.physicsWorld
    if targetWorld then
        targetWorld:addRigidBody(body)
    end
    return self:trackRigidBody(body, targetWorld)
end

function ConstructionScope:commit(metadata)
    assert(not self.closed, "cannot commit a closed construction scope")
    self.committed = true
    local scope = self
    local handle = metadata or {}
    handle.root = handle.root or self.root
    handle.entities = self.entities
    handle.rigidBodies = self.rigidBodies
    handle.scope = self
    function handle:destroy()
        scope:destroy()
    end
    return handle
end

function ConstructionScope:_detachTrackedRelationships()
    local parentComponent = CoreComponents.Parent
    local childrenComponent = CoreComponents.Children
    for _, entity in ipairs(self.entities) do
        if self.registry:hasEntity(entity) then
            local parent = self.registry:get(entity, parentComponent)
            if parent then
                local parentEntity = parent:getParent()
                if parentEntity and self.registry:hasEntity(parentEntity) then
                    self.registry:detachEntity(parentEntity, entity)
                else
                    self.registry:remove(entity, parentComponent)
                end
            end
        end
    end
    for _, entity in ipairs(self.entities) do
        if self.registry:hasEntity(entity) then
            local children = self.registry:get(entity, childrenComponent)
            if children then
                local snapshot = { table.unpack(children.children) }
                for _, child in ipairs(snapshot) do
                    if self.entityIds[child.id] then
                        self.registry:detachEntity(entity, child)
                    end
                end
            end
        end
    end
end

function ConstructionScope:destroy()
    if self.closed then
        return
    end
    self.closed = true
    for index = #self.rigidBodies, 1, -1 do
        local tracked = self.rigidBodies[index]
        if tracked.world and tracked.body then
            pcall(function()
                tracked.world:removeRigidBody(tracked.body)
            end)
        end
    end
    self:_detachTrackedRelationships()
    for index = #self.entities, 1, -1 do
        local entity = self.entities[index]
        if self.registry:hasEntity(entity) then
            self.registry:destroyEntity(entity, self.registry.DESTROY_MODE.KEEP_CHILDREN)
        end
    end
end

function ConstructionScope:rollback()
    self:destroy()
end

return ConstructionScope
