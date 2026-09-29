local Component = require("Core.ECS.Component")
local WeaponMountSizing = require("Shared.Helpers.WeaponMountSizing")

local function normalizeSupportedSizes(value)
    if value == nil then
        return nil
    end
    if type(value) == "string" then
        value = { value }
    end
    assert(type(value) == "table", "capacitor supportedSizeClasses must be a table or size string")
    local result = {}
    local seen = {}
    for _, sizeClass in ipairs(value) do
        local normalized = WeaponMountSizing:normalize(sizeClass)
        assert(not seen[normalized], "duplicate capacitor supported size class: " .. normalized)
        seen[normalized] = true
        table.insert(result, normalized)
    end
    assert(#result > 0, "capacitor supportedSizeClasses cannot be empty")
    return result
end

---@class WeaponCapacitorComponent: Component
---@overload fun(config?: table): WeaponCapacitorComponent
local WeaponCapacitorComponent = Subclass("WeaponCapacitorComponent", Component, function(self, config)
    self:setComponentName("WeaponCapacitor")
    config = config or {}
    local bankSpecs = config.banks or config
    self.banks = {}
    self.banksById = {}
    self.banksByGroup = {}
    self.reservations = {}
    self.reservationSerial = 0

    for index, spec in ipairs(bankSpecs or {}) do
        assert(type(spec) == "table", "capacitor bank definition must be a table")
        if spec.id ~= nil then
            assert(type(spec.id) == "string" or type(spec.id) == "number",
                "capacitor bank id must be a string or number when provided")
        end
        local id = tostring(spec.id or ("bank_" .. tostring(index)))
        assert(id ~= "", "capacitor bank id cannot be empty")
        assert(not self.banksById[id], "duplicate capacitor bank id: " .. id)

        local maxCharge = math.max(0, spec.maxCharge or 0)
        local charge = spec.charge
        if charge == nil then
            charge = maxCharge
        end
        local bank = {
            id = id,
            groupId = spec.groupId,
            supportedSizeClasses = normalizeSupportedSizes(spec.supportedSizeClasses),
            charge = math.max(0, math.min(maxCharge, charge)),
            maxCharge = maxCharge,
            chargeRate = math.max(0, spec.chargeRate or 0),
            declarationIndex = index,
        }
        if bank.groupId ~= nil then
            assert(type(bank.groupId) == "number" and bank.groupId > 0,
                "capacitor bank groupId must be a positive integer enum")
            assert(math.floor(bank.groupId) == bank.groupId,
                "capacitor bank groupId must be a positive integer enum")
            self.banksByGroup[bank.groupId] = self.banksByGroup[bank.groupId] or {}
            table.insert(self.banksByGroup[bank.groupId], bank)
        end
        self.banksById[id] = bank
        table.insert(self.banks, bank)
    end
    self.policies = config.policies or {}
end)

function WeaponCapacitorComponent:getBanks(groupId)
    if groupId ~= nil then
        return self.banksByGroup[groupId] or {}
    end
    return self.banks
end

function WeaponCapacitorComponent:getBank(id)
    return self.banksById[tostring(id)]
end

function WeaponCapacitorComponent:getReservations()
    return self.reservations
end

function WeaponCapacitorComponent:getReservation(id)
    return self.reservations[id]
end

function WeaponCapacitorComponent:setReservation(id, reservation)
    assert(id ~= nil and reservation ~= nil)
    self.reservations[id] = reservation
end

function WeaponCapacitorComponent:removeReservation(id)
    local reservation = self.reservations[id]
    self.reservations[id] = nil
    return reservation
end

function WeaponCapacitorComponent:nextReservationId(prefix)
    self.reservationSerial = self.reservationSerial + 1
    return (prefix or "capacitor") .. ":" .. tostring(self.reservationSerial)
end

function WeaponCapacitorComponent:getPolicies()
    return self.policies
end

function WeaponCapacitorComponent:getPolicy(mode)
    return self.policies[mode]
end

return WeaponCapacitorComponent
