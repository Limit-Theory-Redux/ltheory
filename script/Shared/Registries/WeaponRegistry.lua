---@class WeaponDefinition
---@field id integer
---@field name string
---@field category integer
---@field range number
---@field damage number
---@field damagePerSecond number|nil
---@field laserProfileId integer|nil
---@field laserProfile LaserProfileDefinition|nil
---@field cooldown number
---@field interShotGap number
---@field capacitorCost number
---@field turretScale number
---@field accuracy table
---@field tracking table
---@field firePolicy table
---@field capacityPolicy table
---@field ai table
---@field effect ProjectileDefinition|BeamDefinition

---@class WeaponRegistry
local WeaponRegistry = {
    _definitions = {},
    _order = {},
}

---Register a ship weapon definition by its enum ID.
---@param id integer
---@param definition WeaponDefinition
---@return WeaponDefinition
function WeaponRegistry:new(id, definition)
    assert(type(id) == "number" and id > 0, "weapon ID must be a positive number")
    assert(rawtype(definition) == "table", "weapon definition must be a table")

    local existing = self._definitions[id]
    if existing then
        Log.Warn("Ship weapon already registered: %s", tostring(id))
        return existing
    end

    definition.id = id
    self._definitions[id] = definition
    table.insert(self._order, id)
    return definition
end

---@param id integer
---@return WeaponDefinition|nil
function WeaponRegistry:get(id)
    return self._definitions[id]
end

---@param id integer
---@return boolean
function WeaponRegistry:has(id)
    return self._definitions[id] ~= nil
end

---@return integer[]
function WeaponRegistry:getIds()
    return self._order
end

---@param weapon WeaponDefinition
---@return LaserProfileDefinition|nil
function WeaponRegistry:getLaserProfile(weapon)
    return weapon and weapon.laserProfile or nil
end

---@param weapon WeaponDefinition
---@return table|nil
function WeaponRegistry:getPresentation(weapon)
    local profile = self:getLaserProfile(weapon)
    if profile then
        return profile.presentation
    end
    return weapon and weapon.effect and weapon.effect.visual or nil
end

---@param weapon WeaponDefinition
---@return number
function WeaponRegistry:getDamagePerSecond(weapon)
    local profile = self:getLaserProfile(weapon)
    if profile and profile.damagePerSecond then
        return profile.damagePerSecond
    end
    return weapon and weapon.damagePerSecond or 0
end

setmetatable(WeaponRegistry, {
    __index = function(self, id)
        local definition = rawget(self, "_definitions")[id]
        if definition then
            return definition
        end
        return rawget(WeaponRegistry, id)
    end,
})

return WeaponRegistry
