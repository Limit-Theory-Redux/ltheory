local WeaponRegistry = require("Shared.Registries.WeaponRegistry")

---@class WeaponResolver
local WeaponResolver = {}

local function hasValue(value)
    return value ~= nil
end

---@param identity table
---@return table {weaponId?: integer, weaponRef?: table}
function WeaponResolver:normalize(identity)
    assert(type(identity) == "table", "weapon identity must be a table")
    local hasWeaponId = hasValue(identity.weaponId)
    local hasWeaponRef = hasValue(identity.weaponRef)
    assert(hasWeaponId ~= hasWeaponRef,
        "weapon identity must contain exactly one weaponId or weaponRef")
    if hasWeaponId then
        assert(type(identity.weaponId) == "number",
            "weaponId identity must use a numeric Enums.Weapon.Type value")
        return { weaponId = identity.weaponId }
    end
    assert(type(identity.weaponRef) == "table"
        and type(identity.weaponRef.canonicalKey) == "string"
        and #identity.weaponRef.canonicalKey > 0,
        "weaponRef identity requires a canonical procedural key")
    return { weaponRef = identity.weaponRef }
end

---@param identity table
---@return table|nil
function WeaponResolver:resolve(identity)
    local normalized = self:normalize(identity)
    return WeaponRegistry:resolveIdentity(
        normalized.weaponId,
        normalized.weaponRef)
end

return WeaponResolver
