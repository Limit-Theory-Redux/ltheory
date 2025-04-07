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
