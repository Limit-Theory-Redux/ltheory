-- Checks that each entity constructor adds its kind marker, and that a registry view of a marker
-- lists exactly the entities of that kind in creation order, without confusing a belt with an
-- asteroid. Logs each failed check and ends with a failure exit code.
local Application       = require("States.Application")
local Registry          = require("Core.ECS.Registry")
local Celestial         = require("Modules.CelestialObjects.Components")
local Constructs        = require("Modules.Constructs.Components")
local Economy           = require("Modules.Economy.Components")
local Spatial           = require("Modules.Spatial.Components")
local CelestialEntities = require("Modules.CelestialObjects.Entities")
local ConstructEntities = require("Modules.Constructs.Entities")
local SpatialEntities   = require("Modules.Spatial.Entities")

---@class CoreKindMarkersTest: Application
local KindMarkersTest = Subclass("CoreKindMarkersTest", Application)

function KindMarkersTest:check(ok, what)
    if ok then
        self.passed = self.passed + 1
    else
        self.failed = self.failed + 1
        Log.Error("CoreKindMarkersTest: %s failed", what)
    end
end

--- The entities of a view, in the order the view lists them.
---@param marker table
---@return Entity[]
local function viewed(marker)
    local list = {}
    for entity in Registry:view(marker) do table.insert(list, entity) end
    return list
end

--- True when the view lists exactly these entities, in this order.
---@param marker table
---@param expected Entity[]
---@return boolean
local function viewIs(marker, expected)
    local list = viewed(marker)
    if #list ~= #expected then return false end
    for i = 1, #list do
        if list[i].id ~= expected[i].id then return false end
    end
    return true
end

function KindMarkersTest:onInit()
    self.passed, self.failed = 0, 0

    local seed = 1
    local constructors = {
        { "Star",         Celestial.Star,         CelestialEntities.Star(seed) },
        { "StarSystem",   Celestial.StarSystem,   CelestialEntities.StarSystem(seed) },
        { "Planet",       Celestial.Planet,       CelestialEntities.Planet(seed, {}) },
        { "Moon",         Celestial.Moon,         CelestialEntities.Moon(seed, {}) },
        { "AsteroidBelt", Celestial.AsteroidBelt, CelestialEntities.AsteroidBelt(seed) },
        { "AsteroidRing", Celestial.AsteroidRing, CelestialEntities.AsteroidRing(seed, {}) },
        { "Asteroid",     Celestial.Asteroid,     CelestialEntities.Asteroid(seed, { id = 1 }, 1) },
        { "SpaceStation", Constructs.SpaceStation, ConstructEntities.SpaceStation(seed, {}) },
        { "Zone",         Spatial.Zone,           SpatialEntities.Zone() },
    }
    for _, c in ipairs(constructors) do
        local name, marker, entity = c[1], c[2], c[3]
        self:check(entity:get(marker) ~= nil, name .. " constructor adds its marker")
        self:check(viewIs(marker, { entity }), name .. " view lists only that entity")
    end

    -- A belt is not an asteroid, and the reverse: the case the name-substring checks got wrong.
    self:check(constructors[5][3]:get(Celestial.Asteroid) == nil, "a belt has no asteroid marker")
    self:check(constructors[7][3]:get(Celestial.AsteroidBelt) == nil, "an asteroid has no belt marker")

    -- An asteroid yields an item only when it is given one.
    local bare = CelestialEntities.Asteroid(seed)
    self:check(bare:get(Celestial.Asteroid) ~= nil, "an asteroid without an item has its marker")
    self:check(bare:get(Economy.Item) == nil and bare:get(Economy.Quantity) == nil,
        "an asteroid without an item has no item or quantity")
    local yielding = constructors[7][3]
    self:check(yielding:get(Economy.Item):getItem() == 1 and yielding:get(Economy.Quantity) ~= nil,
        "an asteroid with an item has its item and quantity")

    -- A view lists entities in creation order.
    local first = CelestialEntities.Moon(seed, {})
    local second = CelestialEntities.Moon(seed, {})
    local moons = viewed(Celestial.Moon)
    self:check(#moons == 3 and moons[2].id == first.id and moons[3].id == second.id,
        "a view lists entities in creation order")

    -- A destroyed entity leaves the view.
    Registry:destroyEntity(first, Enums.Registry.EntityDestroyMode.DestroyChildren)
    local remaining = viewed(Celestial.Moon)
    self:check(#remaining == 2 and remaining[2].id == second.id, "a destroyed entity leaves the view")

    Log.Info("CoreKindMarkersTest: %d checks passed, %d failed", self.passed, self.failed)
    self:quit(self.failed == 0 and Core.ExitCode.Success or Core.ExitCode.Failure)
end

return KindMarkersTest
