-- Archetypes

---@enum EntityArchetype
Enums.EntityArchetype = {
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
Enums.ComponentArchetype = {
    NameComponent = 1,
    TypeComponent = 2,
    HierarchyComponent = 3,
    RandomNumberGeneratorComponent = 4,
    TransformComponent = 5,
    PlayerListComponent = 6,
    MeshComponent = 7,
    MaterialComponent = 8
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
