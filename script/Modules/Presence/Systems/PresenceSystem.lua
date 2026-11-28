local Registry = require("Core.ECS.Registry")
local QuickProfiler = require("Shared.Tools.QuickProfiler")
local ScaleConfig = require("Config.Gen.UniverseScaleConfig")
local CameraManager = require("Modules.Cameras.Managers.CameraManager")
local PresenceComponents = require("Modules.Presence.Components")

-- Design: doc/script/presence.md

---@class Materializer
---@field materialize fun(entity: Entity): RigidBody|nil Builds the entity's construct where its virtual body is; returns its body
---@field dematerialize fun(entity: Entity) Takes the construct down (the entity stays, as data)

---Keeps entities as data while nothing needs them in the physics scene, and as constructs where they
---are seen. An entity with a `PresenceComponent` and a `VirtualBodyComponent` is materialized (its
---kind's `Materializer` builds its construct) when the active camera comes within `materializeRange`
---of it, and dematerialized beyond `dematerializeRange` (the gap keeps one at the edge from going in
---and out). While virtual, its virtual body drifts at its velocity, slowed by its drag. While
---materialized, a "virtual" drive keeps moving the virtual body and places the construct's body on
---it; a "physical" drive leaves the construct to fly and copies its body into the virtual one. The
---ranges are checked every `checkS`; the bodies move every frame. Runs at PostSim.
---@class PresenceSystem
local PresenceSystem = Class("PresenceSystem", function(self)
    self:registerVars()
    self:registerEvents()
end)

function PresenceSystem:registerVars()
    self.profiler = QuickProfiler("PresenceSystem", false, false)
    self.materializeRange = ScaleConfig:toGameUnits(30e3, "starSystem")
    self.dematerializeRange = ScaleConfig:toGameUnits(40e3, "starSystem")
    self.checkS = 0.5
    self.untilCheck = 0
    ---@type table<string, Materializer>
    self.materializers = {}
    ---@type table<integer, RigidBody> Entity id -> the body of its construct, while materialized
    self.bodies = {}
end

function PresenceSystem:registerEvents()
    EventBus:subscribe(Event.PostSim, self, self.onPostSim)
end

---@param kind string A `PresenceComponent` kind
---@param materializer Materializer
function PresenceSystem:register(kind, materializer)
    self.materializers[kind] = materializer
end

---@param e EventData
function PresenceSystem:onPostSim(e)
    self:update(e:deltaTime())
end

---Where the scene is seen from, or nil when nothing looks at it (a run without a camera keeps
---everything virtual).
---@return Position|nil
local function observer()
    if not CameraManager:getActiveCameraEntity() then return nil end
    return CameraManager:getEye()
end

---@param dt number
function PresenceSystem:update(dt)
    if dt <= 0 then return end
    self.profiler:start()
    self.untilCheck = self.untilCheck - dt
    local check = self.untilCheck <= 0
    if check then self.untilCheck = self.checkS end
    local eye = check and observer() or nil
    local inRange, outRange = self.materializeRange, self.dematerializeRange
    -- Built and taken down after the walk: a construct adds entities to the registry.
    local entering, leaving = nil, nil

    for entity, presence in Registry:view(PresenceComponents.Presence) do
        local virtual = entity:get(PresenceComponents.VirtualBody)
        if virtual then
            local body = self.bodies[entity.id]
            if body and presence.drive == "physical" then
                local p = body:getPos()
                virtual.position = Position(p.x, p.y, p.z)
                virtual.rotation = body:getRot()
            else
                local p, v = virtual.position, virtual.velocity
                p.x, p.y, p.z = p.x + v.x * dt, p.y + v.y * dt, p.z + v.z * dt
                if virtual.drag > 0 then
                    local keep = math.max(0, 1 - virtual.drag * dt)
                    v.x, v.y, v.z = v.x * keep, v.y * keep, v.z * keep
                end
                if body then
                    body:setPos(p)
                    body:setRot(virtual.rotation)
                end
            end
            if check then
                local p = virtual.position
                local d = math.huge
                if eye then
                    local dx, dy, dz = p.x - eye.x, p.y - eye.y, p.z - eye.z
                    d = math.sqrt(dx * dx + dy * dy + dz * dz)
                end
                if not presence.materialized and d <= inRange then
                    entering = entering or {}
                    entering[#entering + 1] = { entity, presence }
                elseif presence.materialized and d > outRange then
                    leaving = leaving or {}
                    leaving[#leaving + 1] = { entity, presence }
                end
            end
        end
    end
    for _, entry in ipairs(leaving or {}) do self:dematerialize(entry[1], entry[2]) end
    for _, entry in ipairs(entering or {}) do self:materialize(entry[1], entry[2]) end
    self.profiler:stop()
end

---@param entity Entity
---@param presence PresenceComponent
function PresenceSystem:materialize(entity, presence)
    local materializer = self.materializers[presence.kind]
    if not materializer then return end
    local body = materializer.materialize(entity)
    if not body then return end
    self.bodies[entity.id] = body
    presence:setMaterialized(true)
end

---@param entity Entity
---@param presence PresenceComponent
function PresenceSystem:dematerialize(entity, presence)
    local materializer = self.materializers[presence.kind]
    self.bodies[entity.id] = nil
    presence:setMaterialized(false)
    if materializer then materializer.dematerialize(entity) end
end

---Takes an entity out of presence: its construct comes down if it is up. The entity itself is the
---caller's to destroy.
---@param entity Entity
function PresenceSystem:forget(entity)
    local presence = entity:get(PresenceComponents.Presence)
    if presence and presence.materialized then self:dematerialize(entity, presence) end
    self.bodies[entity.id] = nil
end

---The body of an entity's construct, while it is materialized.
---@param entity Entity
---@return RigidBody|nil
function PresenceSystem:bodyOf(entity)
    return self.bodies[entity.id]
end

return PresenceSystem()
