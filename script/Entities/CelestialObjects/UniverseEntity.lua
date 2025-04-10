local Entity = require("Entities.Entity")

-- Components
local NameComponent = require("Components.Core.EntityName")
local SeedComponent = require("Components.Generation.SeedComponent")
local HierarchyComponent = require("Components.Core.EntityHierarchy")

---@class UniverseEntity: Entity
---@overload fun(self: UniverseEntity, seed: integer): UniverseEntity subclass internal
---@overload fun(seed: integer): UniverseEntity subclass external
local UniverseEntity = Subclass("UniverseEntity", Entity, function(self, seed)
    -- Name Component
    self:addComponent(NameComponent())

    -- Seed Component
    self:addComponent(SeedComponent(seed))

    -- Hierarchy/Children Component
    self:addComponent(HierarchyComponent(self:getEntityId()))
end)

return UniverseEntity
