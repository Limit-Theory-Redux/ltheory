local Registry = require("Core.ECS.Registry")
local Entity = require("Core.ECS.Entity")
local QuickProfiler = require("Shared.Tools.QuickProfiler")
local NameComponent = require("Modules.Core.Components.NameComponent")
local CoreComponents = require("Modules.Core.Components")
local ConstructComponents = require("Modules.Constructs.Components")
local Presence = require("Modules.Presence")
local Salvage = require("Modules.Salvage.Components")
local ShipKinds = require("Shared.Definitions.ShipKindDefs")

-- Design: doc/script/presence.md

---Keeps the wrecks of lost ships. A wreck is data (`WreckComponent`, a `VirtualBodyComponent` that
---drifts on from the ship's last motion, slowed by `drag`) until it is seen: then `PresenceSystem`
---has it materialized as a hulk, the ship's hull without weapons or crew, out of the weapons'
---targeting, placed on the wreck's virtual body. Damage to a hulk wears the wreck's integrity; a wreck
---that breaks up (no integrity left) or is salvaged empty is gone. Runs at PostSim, every `checkS`.
---@class WreckSystem
local WreckSystem = Class("WreckSystem", function(self)
    self:registerVars()
    self:registerEvents()
end)

function WreckSystem:registerVars()
    self.profiler = QuickProfiler("WreckSystem", false, false)
    self.checkS = 0.5
    self.untilCheck = 0
    -- The share of a wreck's drift it loses a second.
    self.drag = 0.05
    ---@type ConstructManager|nil The scene's, which builds and takes down the hulks
    self.constructs = nil
    ---@type table<integer, table> Wreck entity id -> its hulk's ConstructManager handle
    self.hulks = {}
    ---@type Entity[]
    self.wrecks = {}
    Presence.System:register("wreck", {
        materialize = function(wreck) return self:buildHulk(wreck) end,
        dematerialize = function(wreck) self:takeDownHulk(wreck) end,
    })
end

function WreckSystem:registerEvents()
    EventBus:subscribe(Event.PostSim, self, self.onPostSim)
end

---@param constructs ConstructManager The scene's
function WreckSystem:attach(constructs)
    self.constructs = constructs
end

---Destroys every wreck (and its hulk) and forgets the scene.
function WreckSystem:clear()
    for _, wreck in ipairs(self.wrecks) do self:remove(wreck) end
    self.wrecks, self.hulks, self.constructs = {}, {}, nil
end

---@return Entity[]
function WreckSystem:getWrecks()
    return self.wrecks
end

---Leaves the wreck of a ship of `kind` where it was lost, drifting on as it moved.
---@param spec { kind: string, seed: integer, name: string, position: Position, velocity: Vec3d|nil, rotation: Quat|nil, cargo: table<integer, integer>|nil }
---@return Entity wreck
function WreckSystem:wreck(spec)
    local kind = ShipKinds[spec.kind] or {}
    local name = "Wreck of " .. spec.name
    local wreck = Entity.Create("Wreck",
        NameComponent(name),
        Salvage.Wreck({ kind = spec.kind, seed = spec.seed, name = name, scrap = kind.scrap,
            cargo = spec.cargo, integrity = kind.wreckIntegrity }),
        Presence.Components.VirtualBody(spec.position, spec.velocity, spec.rotation, self.drag),
        Presence.Components.Presence("wreck", "virtual"))
    self.wrecks[#self.wrecks + 1] = wreck
    Log.Debug("[Wreck] %s left at (%.2f, %.2f, %.2f)", name, spec.position.x, spec.position.y, spec.position.z)
    return wreck
end

---Builds a wreck's hulk: its ship's hull where the wreck is, moved by it (kinematic), and not a
---target.
---@param wreck Entity
---@return RigidBody|nil
function WreckSystem:buildHulk(wreck)
    if not self.constructs then return nil end
    local data = wreck:get(Salvage.Wreck)
    local virtual = wreck:get(Presence.Components.VirtualBody)
    local ok, handle = pcall(function()
        return self.constructs:createShip({ seed = data:getSeed(), shipType = Enums.ShipType.Fighter, config = {
            hull = Enums.ShipHulls.Solo,
            position = virtual:getPosition(),
            scale = Config.game.shipHulls.scale[Enums.ShipHulls.Solo],
            isKinematic = true,
            flightProfile = Enums.FlightProfile.Solo,
        } })
    end)
    if not ok or not handle then
        Log.Warn("[Wreck] %s: no hulk: %s", data:getName(), tostring(handle))
        return nil
    end
    local hulk = handle.root
    hulk:get(NameComponent):setName(data:getName())
    local targetable = hulk:get(ConstructComponents.Targetable)
    if targetable then targetable:setEnabled(false) end
    local health = hulk:get(CoreComponents.Health)
    if health then
        health:setMaxHealth(data:getMaxIntegrity())
        health:setCurrentHealth(data:getIntegrity())
    end
    handle.rigidBody:setRot(virtual:getRotation())
    self.hulks[wreck.id] = handle
    return handle.rigidBody
end

---Takes a wreck's hulk down; the damage it took stays with the wreck.
---@param wreck Entity
function WreckSystem:takeDownHulk(wreck)
    local handle = self.hulks[wreck.id]
    if not handle then return end
    self.hulks[wreck.id] = nil
    local data = Registry:hasEntity(wreck) and wreck:get(Salvage.Wreck)
    local health = handle.root and Registry:hasEntity(handle.root) and handle.root:get(CoreComponents.Health)
    if data and health then data:setIntegrity(health:getCurrentHealth()) end
    if self.constructs then self.constructs:destroy(handle) end
end

---Destroys a wreck and its hulk.
---@param wreck Entity
function WreckSystem:remove(wreck)
    if not Registry:hasEntity(wreck) then return end
    Presence.System:forget(wreck)
    self:takeDownHulk(wreck)
    Registry:destroyEntity(wreck)
end

---@param e EventData
function WreckSystem:onPostSim(e)
    self.untilCheck = self.untilCheck - e:deltaTime()
    if self.untilCheck > 0 then return end
    self.untilCheck = self.checkS
    self:update()
end

---A hulk's damage wears its wreck; a wreck broken up or salvaged empty is gone.
function WreckSystem:update()
    self.profiler:start()
    local kept = {}
    for _, wreck in ipairs(self.wrecks) do
        local data = Registry:hasEntity(wreck) and wreck:get(Salvage.Wreck)
        if data then
            local handle = self.hulks[wreck.id]
            local health = handle and Registry:hasEntity(handle.root) and handle.root:get(CoreComponents.Health)
            if health then data:setIntegrity(health:getCurrentHealth()) end
            if data:getIntegrity() <= 0 or data:isEmpty() then
                Log.Debug("[Wreck] %s is gone (%s)", data:getName(), data:getIntegrity() <= 0 and "broken up" or "salvaged")
                self:remove(wreck)
            else
                kept[#kept + 1] = wreck
            end
        end
    end
    self.wrecks = kept
    self.profiler:stop()
end

return WreckSystem()
