local Entity = require("Entities.Entity")

-- Components
local NameComponent = require("Components.Core.EntityName")
local SeedComponent = require("Components.Generation.SeedComponent")
local HierarchyComponent = require("Components.Core.EntityHierarchy")

-- Types
local EntityInfo = require("Shared.Types.EntityInfo")

---@class UniverseEntity: Entity
---@overload fun(self: UniverseEntity, seed: integer): UniverseEntity subclass internal
---@overload fun(seed: integer): UniverseEntity subclass external
local UniverseEntity = Subclass("UniverseEntity", Entity, function(self, seed)
    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.UniverseEntity)

    -- Name Component
    self:addComponent(NameComponent())

    -- Seed Component
    self:addComponent(SeedComponent(seed))

    -- Hierarchy/Children Component
    self:addComponent(HierarchyComponent(EntityInfo {
        id = self:getGuid(),
        archetype = self:getArchetype()
    }))
end)

return UniverseEntity
