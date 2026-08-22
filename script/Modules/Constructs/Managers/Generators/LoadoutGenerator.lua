local WeaponMountSizing = require("Shared.Helpers.WeaponMountSizing")

---@class LoadoutGenerator
---@overload fun(): LoadoutGenerator
local LoadoutGenerator = Class("LoadoutGenerator", function() end)



local function matchesAssignmentRule(mount, selector)
    for key, value in pairs(selector or {}) do
        local actual = mount[key]
        if key == "mountSizeClass" then
            actual = actual and WeaponMountSizing:normalize(actual)
            value = value and WeaponMountSizing:normalize(value)
        end
        if actual ~= value then
            return false
        end
    end
    return true
end

local function selectPolicyEntry(policy, mount, cursors)
    for _, rule in ipairs(policy.rules or {}) do
        local selector = rule.when or rule.selector or {}
        if matchesAssignmentRule(mount, selector) then
            local entry = rule.entry or rule.assignment
            assert(type(entry) == "table",
                "loadout policy rules require an entry table")
            return table.copy(entry)
        end
    end

    local sizeClass = mount.mountSizeClass
        and WeaponMountSizing:normalize(mount.mountSizeClass)
    local assignments = policy.sizeClassAssignments
        or policy.byMountSizeClass
        or {}
    local choices = assignments[sizeClass]
    if choices and #choices > 0 then
        cursors[sizeClass] = (cursors[sizeClass] or 0) + 1
        local choice = choices[((cursors[sizeClass] - 1) % #choices) + 1]
        return table.copy(choice)
    end

    if policy.fallback then
        return table.copy(policy.fallback)
    end
    return nil
end

---Expand explicit entries and deterministic policy assignments over discovered mounts.
---@param mounts table[] Ordered mount records from HullMountDiscovery.
---@param explicitLoadout table[]|nil Explicit mount-to-identity entries.
---@param policy table|nil Mirroring, selector, and size policy.
---@return table<string, table>
function LoadoutGenerator:expand(mounts, explicitLoadout, policy)
    assert(type(mounts) == "table", "loadout expansion requires discovered mounts")

    local loadoutByMount = {}
    local discoveredByMountId = {}
    for _, mount in ipairs(mounts) do
        assert(type(mount.mountId) == "string" and #mount.mountId > 0,
            "discovered hull mount requires a non-empty mountId")
        assert(not discoveredByMountId[mount.mountId],
            "hull discovery produced a duplicate mountId: " .. mount.mountId)
        discoveredByMountId[mount.mountId] = mount
    end

    for index, entry in ipairs(explicitLoadout or {}) do
        assert(type(entry) == "table" and entry.mountId,
            "loadout entry " .. tostring(index) .. " requires a mountId")
        assert(discoveredByMountId[entry.mountId],
            "loadout references unknown mount: " .. entry.mountId)
        assert(not loadoutByMount[entry.mountId],
            "loadout contains duplicate mount: " .. entry.mountId)
        loadoutByMount[entry.mountId] = table.copy(entry)
    end

    if not policy then
        return loadoutByMount
    end

    local pairsById = {}
    local pairOrder = {}
    for _, mount in ipairs(mounts) do
        if mount.pairId then
            if not pairsById[mount.pairId] then
                pairsById[mount.pairId] = {}
                table.insert(pairOrder, mount.pairId)
            end
            table.insert(pairsById[mount.pairId], mount)
        end
    end

    local cursors = {}
    local processedPairs = {}
    for _, mount in ipairs(mounts) do
        local pairId = mount.pairId
        if pairId and policy.mirrorPairs ~= false and not processedPairs[pairId] then
            local members = pairsById[pairId]
            assert(#members == 2,
                "mirrored loadout policy requires exactly two mounts for pair " .. pairId)
            local first = members[1]
            local second = members[2]
            local firstEntry = loadoutByMount[first.mountId]
            local secondEntry = loadoutByMount[second.mountId]
            if firstEntry and not secondEntry then
                secondEntry = table.copy(firstEntry)
                secondEntry.mountId = second.mountId
                loadoutByMount[second.mountId] = secondEntry
            elseif secondEntry and not firstEntry then
                firstEntry = table.copy(secondEntry)
                firstEntry.mountId = first.mountId
                loadoutByMount[first.mountId] = firstEntry
            elseif firstEntry and secondEntry then
                if policy.allowAsymmetricOverrides ~= true then
                    local firstIdentity = firstEntry.generatedKey
                        or firstEntry.weaponId
                        or (firstEntry.weaponRef and firstEntry.weaponRef.canonicalKey)
                    local secondIdentity = secondEntry.generatedKey
                        or secondEntry.weaponId
                        or (secondEntry.weaponRef and secondEntry.weaponRef.canonicalKey)
                    assert(firstIdentity == secondIdentity,
                        "explicit mirrored loadout overrides must be symmetric for pair " .. pairId)
                end
            elseif not firstEntry and not secondEntry then
                assert(first.mountSizeClass == nil
                    or second.mountSizeClass == nil
                    or WeaponMountSizing:normalize(first.mountSizeClass)
                    == WeaponMountSizing:normalize(second.mountSizeClass),
                    "mirrored loadout pair has incompatible mount sizes: " .. pairId)
                if policy.allowAsymmetricOverrides == true then
                    -- Consult the policy per mount so per-mount rules can
                    -- differentiate pair members (e.g. color variants).
                    local firstSelected = selectPolicyEntry(policy, first, cursors)
                    if firstSelected then
                        firstSelected.mountId = first.mountId
                        loadoutByMount[first.mountId] = firstSelected
                    end
                    local secondSelected = selectPolicyEntry(policy, second, cursors)
                    if secondSelected then
                        secondSelected.mountId = second.mountId
                        loadoutByMount[second.mountId] = secondSelected
                    end
                else
                    local selected = selectPolicyEntry(policy, first, cursors)
                    if selected then
                        selected.mountId = first.mountId
                        loadoutByMount[first.mountId] = selected
                        local mirrored = table.copy(selected)
                        mirrored.mountId = second.mountId
                        loadoutByMount[second.mountId] = mirrored
                    end
                end
            end
            processedPairs[pairId] = true
        elseif not pairId and not loadoutByMount[mount.mountId] then
            local selected = selectPolicyEntry(policy, mount, cursors)
            if selected then
                selected.mountId = mount.mountId
                loadoutByMount[mount.mountId] = selected
            end
        end
    end

    for _, pairId in ipairs(pairOrder) do
        if not processedPairs[pairId] then
            for _, mount in ipairs(pairsById[pairId]) do
                if not loadoutByMount[mount.mountId] then
                    local selected = selectPolicyEntry(policy, mount, cursors)
                    if selected then
                        selected.mountId = mount.mountId
                        loadoutByMount[mount.mountId] = selected
                    end
                end
            end
        end
    end

    return loadoutByMount
end

---Validate role/identity quotas against the resolved loadout.
---@param policy table|nil
---@param mounts table[]
---@param loadoutByMount table<string, table>
function LoadoutGenerator:validateQuotas(policy, mounts, loadoutByMount)
    for role, quota in pairs((policy and policy.quotas) or {}) do
        local unitsById = {}
        for _, mount in ipairs(mounts) do
            if mount.mountRole == role then
                local unitId = mount.pairId or mount.mountId
                unitsById[unitId] = unitsById[unitId] or {}
                table.insert(unitsById[unitId], mount)
            end
        end

        local satisfiedUnits = 0
        for _, unit in pairs(unitsById) do
            local completePair = #unit == 2
                or (#unit == 1 and quota.allowUnpaired == true)
            if completePair then
                local satisfied = true
                for _, mount in ipairs(unit) do
                    local entry = loadoutByMount[mount.mountId]
                    if not entry then
                        satisfied = false
                        break
                    end
                    if quota.generatedKey ~= nil
                        and entry.generatedKey ~= quota.generatedKey
                    then
                        satisfied = false
                        break
                    end
                end
                if satisfied then
                    satisfiedUnits = satisfiedUnits + 1
                end
            end
        end

        local minimumPairs = quota.minimumPairs or quota.minimumUnits or 0
        assert(satisfiedUnits >= minimumPairs,
            "loadout policy quota for role " .. tostring(role)
            .. " requires " .. tostring(minimumPairs)
            .. " satisfied mirrored units, got " .. tostring(satisfiedUnits))
    end
end

---Resolve identities and enforce physical mount capacity.
---@param mounts table[]
---@param explicitLoadout table[]|nil
---@param policy table|nil
---@param resolver fun(entry: table): table, table
---@param options table|nil {requireAll?: boolean}
---@return table result {loadoutByMount, loadout, rawLoadoutByMount}
function LoadoutGenerator:build(mounts, explicitLoadout, policy, resolver, options)
    assert(type(resolver) == "function", "loadout generation requires an identity resolver")
    options = options or {}
    local rawLoadoutByMount = self:expand(mounts, explicitLoadout, policy)
    local loadoutByMount = {}
    local resolvedLoadout = {}

    for _, mount in ipairs(mounts) do
        local entry = rawLoadoutByMount[mount.mountId]
        if entry then
            local weapon, identity = resolver(entry)
            assert(weapon and type(identity) == "table",
                "loadout resolver returned no weapon identity for " .. mount.mountId)
            local fits = WeaponMountSizing:fits(
                mount.mountSizeClass,
                weapon.mountSizeClass)
            if mount.allowedSizeClasses then
                local allowed = false
                for _, allowedSizeClass in ipairs(mount.allowedSizeClasses) do
                    if weapon.mountSizeClass
                        and WeaponMountSizing:normalize(allowedSizeClass)
                        == WeaponMountSizing:normalize(weapon.mountSizeClass)
                    then
                        allowed = true
                        break
                    end
                end
                fits = fits and allowed
            end
            assert(fits,
                "weapon mount size is incompatible with mount " .. mount.mountId
                .. " (mount=" .. tostring(mount.mountSizeClass)
                .. ", weapon=" .. tostring(weapon.mountSizeClass) .. ")")
            local resolved = table.copy(entry)
            resolved.weaponId = identity.weaponId
            resolved.weaponRef = identity.weaponRef
            resolved.weapon = weapon
            loadoutByMount[mount.mountId] = resolved
            table.insert(resolvedLoadout, resolved)
        end
    end

    self:validateQuotas(policy, mounts, loadoutByMount)
    if options.requireAll ~= false then
        for _, mount in ipairs(mounts) do
            assert(loadoutByMount[mount.mountId],
                "loadout is missing mount: " .. mount.mountId)
        end
    end

    return {
        rawLoadoutByMount = rawLoadoutByMount,
        loadoutByMount = loadoutByMount,
        loadout = resolvedLoadout,
    }
end

return LoadoutGenerator()
