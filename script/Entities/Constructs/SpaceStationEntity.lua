local Entity = require("Entities.Entity")

-- Components
local NameComponent = require("Components.Core.EntityName")
local SeedComponent = require("Components.Generation.SeedComponent")
local TransformComponent = require("Components.Physics.TransformComponent")
local MassComponent = require("Components.Physics.MassComponent")
local HierarchyComponent = require("Components.Core.EntityHierarchy")
local InventoryComponent = require("Components.Economy.InventoryComponent")
local MarketplaceComponent = require("Components.Economy.MarketplaceComponent")

-- Types
local EntityInfo = require("Shared.Types.EntityInfo")

---@class SpaceStationEntity: Entity
---@overload fun(self: SpaceStationEntity, seed: integer): SpaceStationEntity subclass internal
---@overload fun(seed: integer): SpaceStationEntity subclass external
local SpaceStationEntity = Subclass("SpaceStationEntity", Entity, function(self, seed)
    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.SpaceStationEntity)

    -- Name Component
    self:addComponent(NameComponent())

    -- Seed Component
    self:addComponent(SeedComponent(seed))

    -- Transform Component
    self:addComponent(TransformComponent())

    -- Mass Component
    self:addComponent(MassComponent())

    -- Hierarchy/Children Component
    self:addComponent(HierarchyComponent(EntityInfo {
        id = self:getGuid(),
        archetype = self:getArchetype()
    }))

    -- Inventory Component
    self:addComponent(InventoryComponent())

    -- Marketplace Component
    self:addComponent(MarketplaceComponent())
end)

return SpaceStationEntity
