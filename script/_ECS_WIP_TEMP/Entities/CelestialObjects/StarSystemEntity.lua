local GlobalStorage = require("_ECS_WIP_TEMP.Systems.GlobalStorage") --!temp path

-- Entities
local Entity = require("_ECS_WIP_TEMP.Entities.Entity") --!temp path

-- Components
local NameComponent = require("_ECS_WIP_TEMP.Components.Core.EntityName")           --!temp path
local HierarchyComponent = require("_ECS_WIP_TEMP.Components.Core.EntityHierarchy") --!temp path
local PlayerListComponent = require("_ECS_WIP_TEMP.Components.Economy.PlayerList")  --!temp path

-- Types
local EntityInfo = require("_ECS_WIP_TEMP.Shared.Types.EntityInfo")

-- Utils
local Words = require('Systems.Gen.Words')

---@class StarSystem: Entity
---@overload fun(self: StarSystem, seed: integer): StarSystem subclass internal
---@overload fun(seed: integer): StarSystem subclass external
local StarSystem = Subclass(Entity, function(self, seed)
    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.StarSystemEntity)

    -- Name Component
    self:addComponent(NameComponent())
    -- PlayerList Component
    self:addComponent(PlayerListComponent())
    --todo: include projectiles

    -- Hierarchy/Children Component
    self:addComponent(HierarchyComponent(EntityInfo {
        id = self:getGuid(),
        archetype = self:getArchetype()
    }))
end)

return StarSystem
