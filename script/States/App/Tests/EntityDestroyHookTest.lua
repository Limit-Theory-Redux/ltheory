local Application      = require('States.Application')
local Registry         = require("Core.ECS.Registry")
local Component        = require("Core.ECS.Component")

---@class HookProbeComponent: Component
---Stands in for a component owning unique native cdata (Mesh/Tex2D): the
---Lua GC can never finalize those, so release must be deterministic via
---Registry:destroyEntity firing onDestroy exactly once.
local HookProbeComponent = Subclass("HookProbeComponent", Component, function(self, tag)
    self:setComponentName("HookProbe")
    self.tag = tag or "?"
    self.freeCount = 0
    self.releaseSeq = 0
end)

---Monotonic release sequence shared across all probes (order assertions).
---@diagnostic disable-next-line: inject-field
HookProbeComponent.nextReleaseSeq = 0

function HookProbeComponent:onDestroy(entityId)
    HookProbeComponent.nextReleaseSeq = HookProbeComponent.nextReleaseSeq + 1
    self.releaseSeq  = HookProbeComponent.nextReleaseSeq
    self.freeCount   = self.freeCount + 1
    self.releasedBy  = entityId
end

---@class EntityDestroyHookTest: Application
local EntityDestroyHookTest = Subclass("EntityDestroyHookTest", Application)

local baselineCount = 0

function EntityDestroyHookTest:onInit()
    self.elapsed  = 0.0
    self.done     = false
    baselineCount = Registry:getEntityCount()
end

---@param data EventData
function EntityDestroyHookTest:onSim(data)
    if self.done then return end
    self.done = true
    self.elapsed = self.elapsed + data:deltaTime()

    local NameComponent = require("Modules.Core.Components.NameComponent")

    -- Parent carries: probe (hook), plain component (no hook).
    -- Child carries: its own probe - proves cascade fires hooks too.
    local parent = Registry:createEntity()
    Registry:add(parent, NameComponent("hookParent"))
    local parentProbe = Registry:add(parent, HookProbeComponent("parent"))

    local child = Registry:createEntity()
    Registry:add(child, NameComponent("hookChild"))
    local childProbe = Registry:add(child, HookProbeComponent("child"))

    Registry:attachEntity(parent, child)

    -- Sanity: both alive, neither released yet.
    assert(parent:isValid() and child:isValid(), "FAIL: entities invalid before destroy")
    assert(parentProbe.freeCount == 0 and childProbe.freeCount == 0,
        "FAIL: premature onDestroy")

    -- Act: cascading destroy.
    Registry:destroyEntity(parent, Registry.DESTROY_MODE.DESTROY_CHILDREN)

    -- Contract 1: teardown happened (hooks run AFTER the registry row died).
    assert(not parent:isValid(), "FAIL: parent still valid after destroy")
    assert(not child:isValid(), "FAIL: child still valid after cascade destroy")

    -- Contract 2: every hook fired exactly once, children-first.
    assert(parentProbe.freeCount == 1,
        "FAIL: parent onDestroy fired " .. tostring(parentProbe.freeCount) .. "x")
    assert(childProbe.freeCount == 1,
        "FAIL: child onDestroy fired " .. tostring(childProbe.freeCount) .. "x")

    -- Contract 2b: children-first ordering (child released before parent).
    assert(childProbe.releaseSeq < parentProbe.releaseSeq,
        "FAIL: expected children-first release order")

    -- Contract 3: destroying an already-dead entity is a safe no-op and
    -- must NOT re-fire hooks.
    assert(Registry:destroyEntity(parent) == false, "FAIL: double destroy returned true")
    assert(parentProbe.freeCount == 1, "FAIL: double destroy re-fired onDestroy")
    assert(childProbe.freeCount == 1, "FAIL: double destroy re-fired child onDestroy")

    -- Contract 4: registry bookkeeping fully reclaimed.
    assert(Registry:getEntityCount() == baselineCount,
        "FAIL: entity count not restored after destroy")

    Log.Info("[EntityDestroyHookTest] cascade+once-only+no-op contracts PASS")
    Log.Info("[EntityDestroyHookTest] ALL PASS")
    self:quit()
end

return EntityDestroyHookTest
