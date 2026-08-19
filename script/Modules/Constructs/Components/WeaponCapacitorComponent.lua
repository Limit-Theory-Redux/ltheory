local Component = require("Core.ECS.Component")

---@class WeaponCapacitorComponent: Component
---@overload fun(config?: table): WeaponCapacitorComponent
local WeaponCapacitorComponent = Subclass("WeaponCapacitorComponent", Component, function(self, config)
    self:setComponentName("WeaponCapacitor")
    config = config or {}
    local bankSpecs = config.banks or config
    self.banks = {}
    self.banksByGroup = {}
    self.groupedBanks = false
    local hasUngroupedBank = false
    for _, spec in ipairs(bankSpecs or {}) do
        assert(type(spec) == "table", "capacitor bank definition must be a table")
        local maxCharge = math.max(0, spec.maxCharge or 0)
        local charge = spec.charge
        if charge == nil then
            charge = maxCharge
        end
        local bank = {
            groupId = spec.groupId,
            charge = math.max(0, math.min(maxCharge, charge)),
            maxCharge = maxCharge,
            chargeRate = math.max(0, spec.chargeRate or 0),
        }
        if bank.groupId ~= nil then
            assert(type(bank.groupId) == "number" and bank.groupId > 0,
                "capacitor bank groupId must be a positive integer enum")
            assert(math.floor(bank.groupId) == bank.groupId,
                "capacitor bank groupId must be a positive integer enum")
            assert(not self.banksByGroup[bank.groupId],
                "duplicate capacitor bank groupId: " .. tostring(bank.groupId))
            self.groupedBanks = true
            self.banksByGroup[bank.groupId] = bank
        else
            hasUngroupedBank = true
        end
        table.insert(self.banks, bank)
    end
    assert(not (self.groupedBanks and hasUngroupedBank),
        "capacitor banks must be either all grouped or all ungrouped")
    self.policies = config.policies or {}
end)

function WeaponCapacitorComponent:getBanks(groupId)
    if groupId ~= nil and self.groupedBanks then
        local bank = self.banksByGroup[groupId]
        return bank and { bank } or {}
    end
    return self.banks
end

function WeaponCapacitorComponent:hasGroupedBanks()
    return self.groupedBanks
end

function WeaponCapacitorComponent:getPolicies()
    return self.policies
end

function WeaponCapacitorComponent:getPolicy(mode)
    return self.policies[mode]
end

return WeaponCapacitorComponent
