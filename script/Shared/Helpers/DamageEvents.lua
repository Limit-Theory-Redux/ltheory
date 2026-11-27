-- Who hit whom: the damage code reports each hit with the entity hit and the entity whose weapon
-- dealt it (a turret is resolved to the ship that carries it), and listeners are told at once.
-- Factions turn their relations on it; anything that reacts to being attacked can listen.

local Registry          = require("Core.ECS.Registry")
local Entity            = require("Core.ECS.Entity")
local ParentComponent   = require("Modules.Core.Components.ParentComponent")

local DamageEvents = { listeners = {} }

---The entity at the top of `entity`'s parent chain (a turret's ship).
---@param entity Entity|nil
---@return Entity|nil
function DamageEvents.rootOf(entity)
    local seen = 0
    while entity and Registry:hasEntity(entity) and seen < 16 do
        local parent = entity:get(ParentComponent)
        local up = parent and parent:getParent()
        if not up or not Registry:hasEntity(up) then return entity end
        entity, seen = up, seen + 1
    end
    return entity
end

---@param listener fun(victim: Entity, attacker: Entity|nil, amount: number)
---@return fun() unsubscribe
function DamageEvents.subscribe(listener)
    DamageEvents.listeners[listener] = true
    return function() DamageEvents.listeners[listener] = nil end
end

---Tells the listeners of a hit.
---@param victimId integer The entity id of what was hit (its health's entity)
---@param source Entity|nil What dealt it (a turret or a ship)
---@param amount number
function DamageEvents.report(victimId, source, amount)
    if not next(DamageEvents.listeners) or amount <= 0 then return end
    local victim = Entity(victimId)
    if not Registry:hasEntity(victim) then return end
    local attacker = DamageEvents.rootOf(source)
    for listener in pairs(DamageEvents.listeners) do listener(victim, attacker, amount) end
end

return DamageEvents
