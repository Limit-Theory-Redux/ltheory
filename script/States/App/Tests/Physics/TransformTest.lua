local Application       = require("States.Application")
local Registry          = require("Core.ECS.Registry")
local PhysicsComponents = require("Modules.Physics.Components")
local TransformSystem   = require("Modules.Physics.Systems.TransformSystem")

---@class TransformTest: Application
local TransformTest     = Subclass("TransformTest", Application)

function TransformTest:onInit()
    EventBus:subscribe(Event.PreSim, self, self.onStatePreSim)
    EventBus:subscribe(Event.PostSim, self, self.onStatePostSim)

    self.entity = Registry:createEntity()
    self.t = self.entity:add(PhysicsComponents.Transform())
    self.rbCmp = self.entity:add(PhysicsComponents.RigidBody())
    local rb = RigidBody.CreateBox()
    self.rbCmp:setRigidBody(rb)

    print(self.entity, self.rbCmp, self.t, self.rbCmp)

    -- test bookkeeping
    self.step     = 0
    self.pending  = false
    self.expected = nil
    self.finished = false

    print("[TransformTest] ready – will wait a full frame for every check")
end

local function copyPos(p) return p and Position(p.x, p.y, p.z) or nil end
local function copyQuat(q) return q and Quat(q.x, q.y, q.z, q.w) or nil end

local function posEq(a, b, eps)
    eps = eps or 0.001
    if not a or not b then
        return a == b
    end
    return math.abs(a.x - b.x) < eps
        and math.abs(a.y - b.y) < eps
        and math.abs(a.z - b.z) < eps
end

local function quatEq(a, b, eps)
    eps = eps or 0.001
    if not a or not b then
        return a == b
    end
    local dot = a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w
    if dot < 0 then
        return math.abs(a.x + b.x) < eps
            and math.abs(a.y + b.y) < eps
            and math.abs(a.z + b.z) < eps
            and math.abs(a.w + b.w) < eps
    else
        return math.abs(a.x - b.x) < eps
            and math.abs(a.y - b.y) < eps
            and math.abs(a.z - b.z) < eps
            and math.abs(a.w - b.w) < eps
    end
end

local function posStr(p) return p and string.format("(%.3f, %.3f, %.3f)", p.x, p.y, p.z) or "nil" end
local function quatStr(q) return q and string.format("(%.3f, %.3f, %.3f, %.3f)", q.x, q.y, q.z, q.w) or "nil" end

function TransformTest:onStatePreSim()
    if self.finished then return end

    self.step = self.step + 1
    local t   = self.t
    local rb  = self.rbCmp:getRigidBody()

    if self.step == 1 then
        print("\n[STEP 1] Transform to RB : world position")
        t:setPos(Position(10, 0, 0))
        t:setDirty(true)

        self.expected = {
            worldPos = Position(10, 0, 0),
            worldRot = copyQuat(t:getRot()),
            localPos = copyPos(t:getPosLocal()),
            localRot = copyQuat(t:getRotLocal())
        }
    elseif self.step == 2 then
        print("\n[STEP 2] Transform to RB : world rotation")
        t:setRot(Quat.FromEuler(0, 45, 0))
        t:setDirty(true)

        self.expected = {
            worldPos = copyPos(t:getPos()),
            worldRot = Quat.FromEuler(0, 45, 0),
            localPos = copyPos(t:getPosLocal()),
            localRot = copyQuat(t:getRotLocal())
        }
    elseif self.step == 3 then
        print("\n[STEP 3] Transform to RB : local position")
        t:setPosLocal(Position(0, 2, 0))
        t:setDirty(true)

        self.expected = {
            worldPos = copyPos(t:getPos()),
            worldRot = copyQuat(t:getRot()),
            localPos = Position(0, 2, 0),
            localRot = copyQuat(t:getRotLocal())
        }
    elseif self.step == 4 then
        print("\n[STEP 4] Transform to RB : local rotation")
        t:setRotLocal(Quat.FromEuler(30, 0, 0))
        t:setDirty(true)

        self.expected = {
            worldPos = copyPos(t:getPos()),
            worldRot = copyQuat(t:getRot()),
            localPos = copyPos(t:getPosLocal()),
            localRot = Quat.FromEuler(30, 0, 0)
        }
    elseif self.step == 5 then
        print("\n[STEP 5] Clear dirty – expect no change")
        t:setDirty(false)
        self.expected = nil
        self.pending  = false
        return
    elseif self.step == 6 then
        print("\n[STEP 6] RB to Transform : modify RB directly")
        rb:setPos(Position(20, 0, 0))
        rb:setRot(Quat.FromEuler(0, 90, 0))
        rb:setPosLocal(Position(0, 4, 0))
        rb:setRotLocal(Quat.FromEuler(45, 0, 0))
        t:setDirty(false)

        self.expected = {
            worldPos = Position(20, 0, 0),
            worldRot = Quat.FromEuler(0, 90, 0),
            localPos = Position(0, 4, 0),
            localRot = Quat.FromEuler(45, 0, 0)
        }
    elseif self.step >= 7 then
        self.step = 7
        return
    end

    self.pending = true
end

function TransformTest:onStatePostSim()
    if self.finished or not self.pending then
        if self.step >= 7 then
            print("\n[TransformTest] ALL TESTS FINISHED")
            self.finished = true
            self:quit()
        end
        return
    end

    local t        = self.t
    local rb       = self.rbCmp:getRigidBody()
    local exp      = self.expected

    local stepName = ({
        [1] = "Transform to RB (world pos)",
        [2] = "Transform to RB (world rot)",
        [3] = "Transform to RB (local pos)",
        [4] = "Transform to RB (local rot)",
        [6] = "RB to Transform (direct modify)"
    })[self.step] or ("step " .. self.step)

    print(string.format("\n[VALIDATE] %s  (one full frame later)", stepName))

    local ok = true

    local function check(name, tVal, rbVal, expVal, isQuat)
        local tOk    = expVal == nil or (isQuat and quatEq(tVal, expVal) or posEq(tVal, expVal))
        local rbOk   = expVal == nil or (isQuat and quatEq(rbVal, expVal) or posEq(rbVal, expVal))
        local sync   = (isQuat and quatEq(tVal, rbVal) or posEq(tVal, rbVal))

        local status = (tOk and rbOk and sync) and "PASS" or "FAIL"
        ok           = ok and (status == "PASS")

        local tStr   = isQuat and quatStr(tVal) or posStr(tVal)
        local rbStr  = isQuat and quatStr(rbVal) or posStr(rbVal)
        print(string.format("  %s to %s  T:%s  RB:%s  sync:%s",
            name, status, tStr, rbStr, sync and "YES" or "NO"))
    end

    check("World Pos", t:getPos(), rb:getPos(), exp.worldPos, false)
    check("World Rot", t:getRot(), rb:getRot(), exp.worldRot, true)
    check("Local Pos", t:getPosLocal(), rb:getPosLocal(), exp.localPos, false)
    check("Local Rot", t:getRotLocal(), rb:getRotLocal(), exp.localRot, true)

    print(ok and "ALL CHECKS PASSED" or "SOME CHECKS FAILED")

    self.pending  = false
    self.expected = nil
end

return TransformTest
