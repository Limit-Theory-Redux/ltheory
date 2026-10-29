-- Checks that an entity's kind is its exact name: matches, other kinds, names that only contain
-- the kind, entities without a name, and destroyed entities. Logs each failed check and ends
-- with a failure exit code.
local Application = require("States.Application")
local Registry    = require("Core.ECS.Registry")
local Entity      = require("Core.ECS.Entity")
local EntityKind  = require("Shared.Helpers.EntityKind")

---@class CoreEntityKindTest: Application
local EntityKindTest = Subclass("CoreEntityKindTest", Application)

local MOON          = "MoonEntity"
local ASTEROID      = "AsteroidEntity"
local ASTEROID_BELT = "AsteroidBeltEntity"

function EntityKindTest:check(ok, what)
    if ok then
        self.passed = self.passed + 1
    else
        self.failed = self.failed + 1
        Log.Error("CoreEntityKindTest: %s failed", what)
    end
end

function EntityKindTest:onInit()
    self.passed, self.failed = 0, 0

    local moon = Entity.Create(MOON)
    self:check(EntityKind.Is(moon, MOON), "an entity is its own kind")
    self:check(not EntityKind.Is(moon, "PlanetEntity"), "an entity is not another kind")
    self:check(not EntityKind.Is(moon, ""), "an entity is not the empty kind")

    -- The kinds the old substring checks confused.
    local belt = Entity.Create(ASTEROID_BELT)
    self:check(not EntityKind.Is(belt, ASTEROID), "a kind that contains another is not that kind")
    self:check(EntityKind.Is(belt, ASTEROID_BELT), "the containing kind matches itself")
    self:check(not EntityKind.Is(Entity.Create(ASTEROID), ASTEROID_BELT),
        "a kind that is contained in another is not that kind")
    self:check(not EntityKind.Is(Entity.Create("My" .. MOON .. "2"), MOON),
        "a name with a prefix and a suffix is not the kind")
    self:check(not EntityKind.Is(Entity.Create("moonentity"), MOON), "the match is case-sensitive")

    self:check(not EntityKind.Is(Registry:createEntity(), MOON), "an entity without a name has no kind")
    self:check(not EntityKind.Is(nil, MOON), "no entity has no kind")

    Registry:destroyEntity(moon, Enums.Registry.EntityDestroyMode.DestroyChildren)
    self:check(not EntityKind.Is(moon, MOON), "a destroyed entity has no kind")

    Log.Info("CoreEntityKindTest: %d checks passed, %d failed", self.passed, self.failed)
    self:quit(self.failed == 0 and Core.ExitCode.Success or Core.ExitCode.Failure)
end

return EntityKindTest
