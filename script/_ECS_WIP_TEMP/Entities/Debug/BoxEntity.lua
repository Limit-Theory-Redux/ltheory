local Entity = require("_ECS_WIP_TEMP.Entities.Entity") --!temp path

-- Components
local TransformComponent = require("_ECS_WIP_TEMP.Components.Physics.TransformComponent") --!temp path
local RenderComponent = require("_ECS_WIP_TEMP.Components.Rendering.RenderComponent")     --!temp path
local RigidBodyComponent = require("_ECS_WIP_TEMP.Components.Physics.RigidBodyComponent") --!temp path
local HierarchyComponent = require("_ECS_WIP_TEMP.Components.Core.EntityHierarchy")       --!temp path

-- Types
local EntityInfo = require("_ECS_WIP_TEMP.Shared.Types.EntityInfo")
local Materials = require("_ECS_WIP_TEMP.Shared.Registries.Materials")

---@class BoxEntity: Entity
---@overload fun(self: BoxEntity): BoxEntity subclass internal
---@overload fun(): BoxEntity subclass external
local BoxEntity = Subclass(Entity, function(self)
    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.BoxEntity)

    -- Transform Component
    self:addComponent(TransformComponent())

    -- Render Component
    local boxMaterial = Materials.DebugColor() ---@type Material
    boxMaterial:addStaticShaderVar("color", Enums.UniformType.Float3, function() return 1.0, 0.0, 1.0 end)
    self:addComponent(RenderComponent({ boxMaterial }, Enums.MeshType.Box))

    -- RigidBody Component
    self:addComponent(RigidBodyComponent())

    -- Hierarchy/Children Component
    self:addComponent(HierarchyComponent(EntityInfo {
        id = self:getGuid(),
        archetype = self:getArchetype()
    }))
end)

return BoxEntity
