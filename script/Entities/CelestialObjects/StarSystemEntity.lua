local Registry = require("Systems.Storage.Registry")

-- Entities
local Entity = require("Entities.Entity")

-- Components
local NameComponent = require("Components.Core.EntityName")
local SeedComponent = require("Components.Generation.SeedComponent")
local TransformComponent = require("Components.Physics.TransformComponent")
local MassComponent = require("Components.Physics.MassComponent")
local HierarchyComponent = require("Components.Core.EntityHierarchy")

-- Utils
local Words = require('Legacy.Systems.Gen.Words')

---@class StarSystemEntity: Entity
---@overload fun(self: StarSystemEntity, seed: integer): StarSystemEntity subclass internal
---@overload fun(seed: integer): StarSystemEntity subclass external
local StarSystemEntity = Subclass("StarSystemEntity", Entity, function(self, seed)
    -- Name Component
    self:addComponent(NameComponent())

    -- Seed Component
    self:addComponent(SeedComponent(seed))

    -- Transform Component
    self:addComponent(TransformComponent())

    -- Mass Component
    self:addComponent(MassComponent())

    -- Hierarchy/Children Component
    self:addComponent(HierarchyComponent(self:getEntityId()))
end)

return StarSystemEntity
