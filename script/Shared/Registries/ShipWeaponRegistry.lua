---@class ShipWeaponDefinition
---@field key string
---@field projectileSpeed number
---@field range number
---@field damage number
---@field projectileLifetime number
---@field cooldown number
---@field interShotGap number
---@field traverseRate number
---@field aimTolerance number
---@field turretScale number
---@field projectileScale number
---@field pulseHeadSize number
---@field pulseTailWidth number
---@field pulseTailLength number
---@field ai table

---@class ShipWeaponRegistry
local ShipWeaponRegistry = {
    _definitions = {},
    _order = {},
}

---Register a ship weapon definition.
---@param key string
---@param definition ShipWeaponDefinition
---@return ShipWeaponDefinition
function ShipWeaponRegistry:new(key, definition)
    assert(type(key) == "string" and #key > 0, "ship weapon key must be a non-empty string")
    assert(type(definition) == "table", "ship weapon definition must be a table")

    local existing = self._definitions[key]
    if existing then
        Log.Warn("Ship weapon already registered: %s", key)
        return existing
    end

    definition.key = key
    self._definitions[key] = definition
    table.insert(self._order, key)
    return definition
end

---Get a registered ship weapon definition.
---@param key string
---@return ShipWeaponDefinition|nil
function ShipWeaponRegistry:get(key)
    return self._definitions[key]
end

---Return whether a ship weapon is registered.
---@param key string
---@return boolean
function ShipWeaponRegistry:has(key)
    return self._definitions[key] ~= nil
end

---Return registered keys in deterministic registration order.
---@return string[]
function ShipWeaponRegistry:getKeys()
    return self._order
end

setmetatable(ShipWeaponRegistry, {
    __index = function(self, key)
        local definition = rawget(self, "_definitions")[key]
        if definition then
            return definition
        end
        return rawget(ShipWeaponRegistry, key)
    end,
})

return ShipWeaponRegistry
