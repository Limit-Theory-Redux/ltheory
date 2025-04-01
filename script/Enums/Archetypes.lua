-- Archetypes

---@enum EntityArchetype
Enums.EntityArchetype = { --* reorder these later
    StarSystemEntity = 1,
    StarEntity = 2,
    PlanetEntity = 3,
    MoonEntity = 4,
    AsteroidEntity = 5,
    SpaceStationEntity = 6,
    SpaceshipEntity = 7,
    EffectEntity = 8,
    PlayerEntity = 9,
    CameraEntity = 10,
    UniverseEntity = 11,
    AsteroidBeltEntity = 12,
    AsteroidRingEntity = 13,
    ZoneEntity = 14,
    TriggerEntity = 15,
    ItemEntity = 16,
    OrderEntity = 17,
    BoxEntity = 18,
    -- ...
}

local entityArchetypeNames = {}
for k, v in pairs(Enums.EntityArchetype) do
    entityArchetypeNames[v] = k
end

---@param archetype EntityArchetype
---@return string
function Enums.EntityArchetype:getName(archetype)
    return entityArchetypeNames[archetype]
end

---@enum ComponentArchetype
Enums.ComponentArchetype = { --* reorder these later
    NameComponent = 1,
    TypeComponent = 2,
    HierarchyComponent = 3,
    RNGComponent = 4,
    TransformComponent = 5,
    PlayerListComponent = 6,
    RenderComponent = 7,
    MeshComponent = 8,
    MaterialComponent = 9,
    RigidBodyComponent = 10,
    CameraDataComponent = 11,
    SeedComponent = 12,
    ShapeComponent = 13,
    MarketplaceComponent = 14,
    MassComponent = 15,
    InventoryComponent = 16,
    QuantityComponent = 17,
    OwnershipComponent = 18,
    OrderStatusComponent = 19,
    ExpiryComponent = 20,
    PlayerBankAccount = 21,
    PriceComponent = 22,
    OrderItemTypeComponent = 23
    -- ...
}

local componentArchetypeNames = {}
for k, v in pairs(Enums.ComponentArchetype) do
    componentArchetypeNames[v] = k
end

---@param archetype EntityArchetype
---@return string
function Enums.ComponentArchetype:getName(archetype)
    return componentArchetypeNames[archetype]
end
