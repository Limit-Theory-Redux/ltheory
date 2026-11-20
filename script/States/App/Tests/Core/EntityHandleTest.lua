-- Checks that entity handles compare by the entity they refer to: two handles of one entity
-- are equal wherever they come from, and handles of different entities are not. Logs each
-- failed check and ends with a failure exit code.
local Application     = require("States.Application")
local Registry        = require("Core.ECS.Registry")
local Entity          = require("Core.ECS.Entity")
local Spatial         = require("Modules.Spatial.Components")
local SpatialEntities = require("Modules.Spatial.Entities")

---@class CoreEntityHandleTest: Application
local EntityHandleTest = Subclass("CoreEntityHandleTest", Application)

function EntityHandleTest:check(ok, what)
    if ok then
        self.passed = self.passed + 1
    else
        self.failed = self.failed + 1
        Log.Error("CoreEntityHandleTest: %s failed", what)
    end
end

function EntityHandleTest:onInit()
    self.passed, self.failed = 0, 0

    local zone = SpatialEntities.Zone()
    local other = SpatialEntities.Zone()

    self:check(Entity(zone.id) == zone, "a new handle of an entity equals the entity")
    self:check(Registry:getEntityById(zone.id) == zone, "a handle from getEntityById equals the entity")
    local viewed
    for entity in Registry:view(Spatial.Zone) do
        if entity.id == zone.id then viewed = entity end
    end
    self:check(viewed ~= nil and rawequal(viewed, zone) == false and viewed == zone,
        "a handle from a view is a new table and equals the entity")
    self:check(zone ~= other, "handles of different entities differ")
    self:check(zone ~= nil, "a handle differs from nil")
    self:check(zone ~= { id = zone.id }, "a handle differs from a table that is not a handle")

    Log.Info("CoreEntityHandleTest: %d checks passed, %d failed", self.passed, self.failed)
    self:quit(self.failed == 0 and Core.ExitCode.Success or Core.ExitCode.Failure)
end

return EntityHandleTest
