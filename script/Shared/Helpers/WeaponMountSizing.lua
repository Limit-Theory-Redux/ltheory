---@class WeaponMountSizing
local WeaponMountSizing = {}

local ORDER = {
    sm = 1,
    m = 2,
    l = 3,
    xl = 4,
}

local ALIASES = {
    small = "sm",
    medium = "m",
    large = "l",
    capital = "xl",
    ["extra-large"] = "xl",
    ["extra_large"] = "xl",
}

local function normalize(value)
    if value == nil then
        return nil
    end
    local normalized = string.lower(tostring(value))
    normalized = ALIASES[normalized] or normalized
    assert(ORDER[normalized], "unknown weapon mount size class: " .. tostring(value))
    return normalized
end

function WeaponMountSizing:normalize(value)
    return normalize(value)
end

function WeaponMountSizing:isValid(value)
    if value == nil then
        return false
    end
    local normalized = string.lower(tostring(value))
    normalized = ALIASES[normalized] or normalized
    return ORDER[normalized] ~= nil
end

---A mount capacity can install weapons up to its declared size.
---@param mountSizeClass string|nil
---@param weaponSizeClass string|nil
---@return boolean
function WeaponMountSizing:fits(mountSizeClass, weaponSizeClass)
    if mountSizeClass == nil or weaponSizeClass == nil then
        return true
    end
    local mountSize = normalize(mountSizeClass)
    local weaponSize = normalize(weaponSizeClass)
    return ORDER[weaponSize] <= ORDER[mountSize]
end

function WeaponMountSizing:order(value)
    local normalized = normalize(value)
    return normalized and ORDER[normalized] or nil
end

return WeaponMountSizing
