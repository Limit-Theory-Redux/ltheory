local Registry            = require("Core.ECS.Registry")
local PhysicsComponents   = require("Modules.Physics.Components")
local SensorComponents    = require("Modules.Sensors.Components")
local SensorModel         = require("Modules.Sensors.SensorModel")
local UniverseScaleConfig = require("Config.Gen.UniverseScaleConfig")

--- SensorSystem: runs every `Sensor` against every `SensorSignature`.
---
--- Each update, for each sensor: places every signature relative to the
--- sensor's entity (in real metres), fires a requested pulse, evaluates the
--- sensor model and moves its state on (echoes fade, scans read, heat). Ships'
--- heat follows their activity once per update, for all sensors alike.
---
--- A sensor sees each signature through a view of its own (its `pos` and the
--- signature's `entity`), kept for as long as the signature exists, so pulse
--- echoes, scan reads and the memory of it stay with it from one update to the
--- next. A signature that is gone is forgotten.
---
--- A view also keeps where the object is in the world (`world`, game units),
--- where the sensor last had it as a contact (`seenAt`), and that place
--- relative to the sensor now (`lastPos`, metres): a contact it loses stays
--- where it was last seen.
---@class SensorSystem
local SensorSystem = {}

--- The position of an entity: its rigid body's, or its transform's.
---@param entity Entity
---@return Position|nil
function SensorSystem:positionOf(entity)
    local rbCmp = entity:get(PhysicsComponents.RigidBody)
    local rb = rbCmp and rbCmp:getRigidBody()
    if rb then return rb:getPos() end
    local transform = entity:get(PhysicsComponents.Transform)
    return transform and transform:getPos() or nil
end

--- The radius of an entity in real metres: its rigid body's bounding radius
--- (with its parts), or its transform's scale.
---@param entity Entity
---@return number
function SensorSystem:radiusOf(entity)
    local rbCmp = entity:get(PhysicsComponents.RigidBody)
    local rb = rbCmp and rbCmp:getRigidBody()
    local r = rb and rb:getBoundingRadiusCompound()
    if not (r and r > 0) then
        local transform = entity:get(PhysicsComponents.Transform)
        r = transform and transform:getScale() or 0
    end
    return r / UniverseScaleConfig.globalScale
end

---@param dt number seconds
function SensorSystem:update(dt)
    local signatures = {}
    for entity, signature in Registry:view(SensorComponents.SensorSignature) do
        if signature:getRadius() == nil then
            signature:setRadius(self:radiusOf(entity))
        end
        SensorModel.stepObjectHeat(signature:getObject(), dt)
        local pos = self:positionOf(entity)
        if pos then
            signatures[#signatures + 1] = { entity = entity, signature = signature, pos = pos }
        end
    end

    for entity, sensor in Registry:view(SensorComponents.Sensor) do
        self:updateSensor(entity, sensor, signatures, dt)
    end
end

---@param entity Entity the sensor's
---@param sensor SensorComponent
---@param signatures { entity: Entity, signature: SensorSignatureComponent, pos: Position }[]
---@param dt number
function SensorSystem:updateSensor(entity, sensor, signatures, dt)
    local origin = self:positionOf(entity)
    if not origin then return end
    local state = sensor:getState()
    local toMetres = 1 / UniverseScaleConfig.globalScale

    local objects, seen = {}, {}
    for _, s in ipairs(signatures) do
        if s.entity.id ~= entity.id then
            local view = sensor:getView(s.signature)
            if not view then
                view = setmetatable({ pos = { x = 0, y = 0, z = 0 }, entity = s.entity },
                    { __index = s.signature:getObject() })
                sensor:setView(s.signature, view)
            end
            view.pos.x = (s.pos.x - origin.x) * toMetres
            view.pos.y = (s.pos.y - origin.y) * toMetres
            view.pos.z = (s.pos.z - origin.z) * toMetres
            view.world = s.pos
            objects[#objects + 1] = view
            seen[view] = true
        end
    end
    for signature, view in pairs(sensor:getViews()) do
        if not seen[view] then
            SensorModel.forget(state, view)
            sensor:setView(signature, nil)
        end
    end
    state.objects = objects

    if sensor:isPulseRequested() then
        SensorModel.firePulse(state)
        sensor:clearPulseRequest()
    end
    local readings = SensorModel.evaluate(state)
    SensorModel.step(state, readings, dt)
    sensor:setReadings(readings)

    for _, view in ipairs(objects) do
        if readings[view].tier >= 1 then
            view.seenAt = view.seenAt or {}
            view.seenAt.x, view.seenAt.y, view.seenAt.z = view.world.x, view.world.y, view.world.z
        end
        if view.seenAt then
            view.lastPos = view.lastPos or {}
            view.lastPos.x = (view.seenAt.x - origin.x) * toMetres
            view.lastPos.y = (view.seenAt.y - origin.y) * toMetres
            view.lastPos.z = (view.seenAt.z - origin.z) * toMetres
        end
    end
end

return SensorSystem
