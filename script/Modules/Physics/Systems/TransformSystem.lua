local QuickProfiler = require("Shared.Tools.QuickProfiler")
local Registry = require("Core.ECS.Registry")
local PhysicsComponents = require("Modules.Physics.Components")

---@class TransformSystem
---@overload fun(self: TransformSystem): TransformSystem class internal
---@overload fun(): TransformSystem class external
local TransformSystem = Class("TransformSystem", function(self)
    ---@diagnostic disable-next-line: invisible
    self:registerVars()
    self:registerEvents()
end)

---@private
function TransformSystem:registerVars()
    ---@private
    self.profiler = QuickProfiler("TransformSystem", false, false)
end

function TransformSystem:registerEvents()
    EventBus:subscribe(Event.PreSim, self, self.onPreSim)
    EventBus:subscribe(Event.PostSim, self, self.onPostSim)
end

-- Helper: Compare two vectors with epsilon tolerance
---@param a table Vector with x, y, z
---@param b table Vector with x, y, z
---@param eps number Epsilon tolerance (default 0.0001)
---@return boolean True if vectors are equal within epsilon
local function vecEquals(a, b, eps)
    if not a or not b then return a == b end
    eps = eps or 0.0001
    return math.abs(a.x - b.x) < eps
        and math.abs(a.y - b.y) < eps
        and math.abs(a.z - b.z) < eps
end

-- Helper: Compare two quaternions with epsilon tolerance
---@param a table Quaternion with x, y, z, w
---@param b table Quaternion with x, y, z, w
---@param eps number Epsilon tolerance (default 0.0001)
---@return boolean True if quaternions are equal within epsilon (handles negation)
local function quatEquals(a, b, eps)
    if not a or not b then return a == b end
    eps = eps or 0.0001

    -- Quaternions q and -q represent the same rotation
    -- Check both orientations
    local dotProduct = a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w

    if dotProduct < 0 then
        -- Check negated quaternion
        return math.abs(a.x + b.x) < eps
            and math.abs(a.y + b.y) < eps
            and math.abs(a.z + b.z) < eps
            and math.abs(a.w + b.w) < eps
    else
        -- Check direct equality
        return math.abs(a.x - b.x) < eps
            and math.abs(a.y - b.y) < eps
            and math.abs(a.z - b.z) < eps
            and math.abs(a.w - b.w) < eps
    end
end

---@param data EventData
function TransformSystem:onPreSim(data)
    self.profiler:start("PreSim")

    for entity, transform in Registry:iterEntities(PhysicsComponents.Transform) do
        -- Check validity
        if not entity:isValid() or not transform or not transform:isDirty() then
            goto continue
        end

        -- Get RigidBody component
        local rbCmp = entity:get(PhysicsComponents.RigidBody)
        if not rbCmp then
            goto continue
        end

        local rb = rbCmp:getRigidBody()
        if not rb then
            goto continue
        end

        -- Get transform values
        local pos = transform:getPos()
        local rot = transform:getRot()
        local posLocal = transform:getPosLocal()
        local rotLocal = transform:getRotLocal()

        -- Update rigid body transform (only if values differ)
        -- CRITICAL: Use vecEquals/quatEquals instead of ~= for cdata comparison

        if pos and not vecEquals(rb:getPos(), pos) then
            rb:setPos(pos)
        end

        if rot and not quatEquals(rb:getRot(), rot) then
            rb:setRot(rot)
        end

        if posLocal and not vecEquals(rb:getPosLocal(), posLocal) then
            rb:setPosLocal(posLocal)
        end

        if rotLocal and not quatEquals(rb:getRotLocal(), rotLocal) then
            rb:setRotLocal(rotLocal)
        end

        transform:setDirty(false)

        ::continue::
    end

    self.profiler:stop("PreSim")
end

---@param data EventData
function TransformSystem:onPostSim(data)
    self.profiler:start("PostSim")

    for entity, rbCmp in Registry:iterEntities(PhysicsComponents.RigidBody) do
        if not entity:isValid() or not rbCmp then
            goto continue
        end

        local rb = rbCmp:getRigidBody()
        if not rb then
            goto continue
        end

        local transform = entity:get(PhysicsComponents.Transform)
        if not transform then
            goto continue
        end

        -- Get RigidBody values
        local pos = rb:getPos()
        local rot = rb:getRot()
        local posLocal = rb:getPosLocal()
        local rotLocal = rb:getRotLocal()

        -- Update transform component (only if values differ)
        local isDirty = false

        if pos and not vecEquals(transform:getPos(), pos) then
            transform:setPos(pos)
            isDirty = true
        end

        if rot and not quatEquals(transform:getRot(), rot) then
            transform:setRot(rot)
            isDirty = true
        end

        if posLocal and not vecEquals(transform:getPosLocal(), posLocal) then
            transform:setPosLocal(posLocal)
            isDirty = true
        end

        if rotLocal and not quatEquals(transform:getRotLocal(), rotLocal) then
            transform:setRotLocal(rotLocal)
            isDirty = true
        end

        if isDirty then
            transform:setDirty(true)
        end

        ::continue::
    end

    self.profiler:stop("PostSim")
end

return TransformSystem()
