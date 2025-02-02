local Entity = require("_ECS_WIP_TEMP.Entities.Entity") --!temp path

-- Components
local TransformComponent = require("_ECS_WIP_TEMP.Components.Physics.TransformComponent") --!temp path
local RenderComponent = require("_ECS_WIP_TEMP.Components.Rendering.RenderComponent")     --!temp path
local RigidBodyComponent = require("_ECS_WIP_TEMP.Components.Physics.RigidBodyComponent") --!temp path
local HierarchyComponent = require("_ECS_WIP_TEMP.Components.Core.EntityHierarchy")       --!temp path

-- Types
local EntityInfo = require("_ECS_WIP_TEMP.Shared.Types.EntityInfo")

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
