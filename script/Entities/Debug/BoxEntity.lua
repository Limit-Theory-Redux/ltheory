local Entity = require("Entities.Entity")

-- Components
local TransformComponent = require("Components.Physics.TransformComponent")
local RenderComponent = require("Components.Rendering.RenderComponent")
local RigidBodyComponent = require("Components.Physics.RigidBodyComponent")
local HierarchyComponent = require("Components.Core.EntityHierarchy")

-- Types
local EntityInfo = require("Shared.Types.EntityInfo")

---@class BoxEntity: Entity
---@overload fun(self: BoxEntity, material: Material): BoxEntity subclass internal
---@overload fun(material: Material): BoxEntity subclass external
local BoxEntity = Subclass("BoxEntity", Entity, function(self, material)
    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.BoxEntity)

    -- Transform Component
    self:addComponent(TransformComponent())

    -- Render Component
    self:addComponent(RenderComponent({ material }, Enums.MeshType.Box))

    -- RigidBody Component
    self:addComponent(RigidBodyComponent())

    -- Hierarchy/Children Component
    self:addComponent(HierarchyComponent(EntityInfo {
        id = self:getGuid(),
        archetype = self:getArchetype()
    }))
end)

return BoxEntity
