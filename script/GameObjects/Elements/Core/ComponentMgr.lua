local Entity = require('GameObjects.Entity')

-- This module integrates functionality for multiple components of each type installed
--    in a vessel (which currently includes Ship and Station but not Asteroid or Planet)

-- WARNING : Note the subtlety that isAlive and isDestroyed are NOT
--           complementary! An asteroid is not alive, but neither has it been
--           destroyed. Both 'alive' and 'destroyed' require an entity to have a Hull.

-- *** GENERAL FUNCTIONS ***
function Entity:addComponents()
    assert(not self.components)
    self.components = {
        armor       = {},
        bay         = {},
        capacitor   = {},
        cloak       = {},
        commo       = {},
        computer    = {},
        drone       = {},
        hull        = {},
        inventory   = {},
        sensor      = {},
        shield      = {},
        thruster    = {},
        turret      = {},
    }
end

-- TODO: Function interfaces for drone racks

function Entity:isAlive()
    --printf("isAlive() self = %s", self:getName())
    if self.components and self.components.hull and #self.components.hull > 0 then
        local hull = self.components.hull[1]
        if not hull then
            return false
        else
            return hull:getHealth() > 0
        end
    else
        return false -- such as for asteroids
    end
end

function Entity:isDestroyed()
    --printf("isDestroyed() self = %s", self:getName())
    if self.components and self.components.hull and #self.components.hull > 0 then
        local hull = self.components.hull[1]
        if not hull then
            return false
        else
            return hull:getHealth() <= 0
        end
    else
        return false
    end
end

-- *** ARMOR FUNCTIONS ***
function Entity:mgrArmorDamageHealth(value)
    local armors = self.components.armor
    if not armors or #armors == 0 then return end

    -- Spread armor damage amount evenly across all installed armors that have health remaining
    local armorCount = 0
    for _, armor in ipairs(armors) do
        if armor:getHealth() > 0 then armorCount = armorCount + 1 end
    end

    local spreadValue = value / armorCount
    for _, armor in ipairs(armors) do
        if armor:getHealth() > 0 then
            armor:damageHealth(spreadValue)
            --printf("damaging %s by %s to %s (max %s)", armor:getName(), spreadValue, armor:getHealth(), armor:getHealthMax())
        end
    end
end

function Entity:mgrArmorGetHealth()
    local armors = self.components.armor
    if not armors or #armors == 0 then return 0 end

    local health = 0
    for _, armor in ipairs(armors) do
        health = health + armor:getHealth()
    end

    return health
end

function Entity:mgrArmorGetHealthMax()
    local armors = self.components.armor
    if not armors or #armors == 0 then return 0 end

    local healthMax = 0
    for _, armor in ipairs(armors) do
        healthMax = healthMax + armor:getHealthMax()
    end

    return healthMax
end

function Entity:mgrArmorGetHealthPercent()
    local health    = self:mgrArmorGetHealth()
    local healthMax = self:mgrArmorGetHealthMax()

    if health == 0 or healthMax == 0 then return 0 end

    return 100.0 * health / healthMax
end

-- *** CAPACITOR FUNCTIONS ***
function Entity:mgrCapacitorDamageHealth(value)
    local capacitors = self.components.capacitor
    if not capacitors or #capacitors == 0 then return end

    -- Spread capacitor damage amount evenly across all installed capacitors that have health remaining
    local capCount = 0
    for _, cap in ipairs(capacitors) do
        if cap:getHealth() > 0 then capCount = capCount + 1 end
    end

    local spreadValue = value / capCount
    for _, cap in ipairs(capacitors) do
        if cap:getHealth() > 0 then
            cap:damageHealth(spreadValue)
            --printf("damaging %s by %s to %s (max %s)", cap:getName(), spreadValue, cap:getHealth(), cap:getHealthMax())
        end
    end
end

function Entity:mgrCapacitorDischarge(amount)
    local capacitors = self.components.capacitor
    if not capacitors then return end

    -- Remove charge from capacitors in sequence until none of the requested amount remains undischarged
    -- If any does remain, return the total amount that was discharged
    local undischarged = amount
    for _, cap in ipairs(capacitors) do
        undischarged = cap:discharge(undischarged)
    end

    return undischarged
end

function Entity:mgrCapacitorGetCharge()
    local capacitors = self.components.capacitor
    if not capacitors then return 0 end

    local charge = 0
    for _, cap in ipairs(capacitors) do
        charge = charge + cap:getCharge()
    end

    return charge
end

function Entity:mgrCapacitorGetChargeMax()
    local capacitors = self.components.capacitor
    if not capacitors then return 0 end

    local chargeMax = 0
    for _, cap in ipairs(capacitors) do
        chargeMax = chargeMax + cap:getChargeMax()
    end

    return chargeMax
end

function Entity:mgrCapacitorGetChargePercent()
    local charge    = self:mgrCapacitorGetCharge()
    local chargeMax = self:mgrCapacitorGetChargeMax()

    if charge == 0 or chargeMax == 0 then return 0 end

    return 100.0 * charge / chargeMax
end

-- *** CLOAK FUNCTIONS ***
function Entity:mgrCloakActivate()
    local cloaks = self.components.cloak
    if not cloaks or #cloaks == 0 then return end

    local count = 0
    for _, cloak in ipairs(cloaks) do
        cloak:activate()
    end
end

function Entity:mgrCloakDamageHealth(value)
    local cloaks = self.components.cloak
    if not cloaks or #cloaks == 0 then return end

    local cloakCount = 0
    for _, cloak in ipairs(cloaks) do
        if cloak:getHealth() > 0 then cloakCount = cloakCount + 1 end
    end

    -- Spread cloak damage amount evenly across all installed cloaking devices that have health remaining
    local spreadValue = value / cloakCount
    for _, cloak in ipairs(cloaks) do
        if cloak:getHealth() > 0 then
            cloak:damageHealth(spreadValue)
            --printf("damaging %s by %s to %s (max %s)", cloak:getName(), spreadValue, cloak:getHealth(), cloak:getHealthMax())
        end
    end
end

function Entity:mgrCloakDeactivate()
    local cloaks = self.components.cloak
    if not cloaks or #cloaks == 0 then return end

    local strength = 0
    for _, cloak in ipairs(cloaks) do
        cloak:deactivate()
    end
end

function Entity:mgrCloakGetHealth()
    local cloaks = self.components.cloak
    if not cloaks or #cloaks == 0 then return end

    local health = 0
    for _, cloak in ipairs(cloaks) do
        health = health + cloak:getHealth()
    end

    return health
end

function Entity:mgrCloakGetStrength()
    -- Returns the total current strength of all _active_ cloaking devices installed
    -- TODO: Continuously compare the current cloak strength against the hull size + active emissions of the ship
    --       If the cloak strength >= size + emissions, then the ship remains cloaked
    local cloaks = self.components.cloak
    if not cloaks or #cloaks == 0 then return end

    local strength = 0
    for _, cloak in ipairs(cloaks) do
        strength = strength + cloak:getStrength()
    end

    return strength
end

-- *** COMPUTER FUNCTIONS ***
function Entity:mgrComputerDamageHealth(value)
    local computers = self.components.computer
    if not computers or #computers == 0 then return end

    -- Spread computer damage amount evenly across all installed computers that have health remaining
    local computerCount = 0
    for _, computer in ipairs(computers) do
        if computer:getHealth() > 0 then computerCount = computerCount + 1 end
    end

    local spreadValue = value / computerCount
    for _, computer in ipairs(computers) do
        if computer:getHealth() > 0 then
            computer:damageHealth(spreadValue)
            --printf("damaging %s by %s to %s (max %s)", computer:getName(), spreadValue, computer:getHealth(), computer:getHealthMax())
        end
    end
end

function Entity:mgrComputerGetHealth()
    local computers = self.components.computer
    if not computers or #computers == 0 then return 0 end

    local health = 0
    for _, computer in ipairs(computers) do
        health = health + computer:getHealth()
    end

    return health
end

function Entity:mgrComputerGetLockCount()
    local computers = self.components.computer
    if not computers or #computers == 0 then return 0 end

    local count = 0
    for _, computer in ipairs(computers) do
        count = count + computer:getLockCount()
    end

    return count
end

function Entity:mgrComputerGetLockStrength()
    local computers = self.components.computer
    if not computers or #computers == 0 then return 0 end

    local strength = 0
    for _, computer in ipairs(computers) do
        strength = strength + computer:getLockStrength()
    end

    return strength
end

function Entity:mgrComputerGetMappingSpeed()
    local computers = self.components.computer
    if not computers or #computers == 0 then return 0 end

    local speed = 0
    for _, computer in ipairs(computers) do
        speed = speed + computer:getMappingSpeed()
    end

    return speed
end

function Entity:mgrComputerGetRating()
    local computers = self.components.computer
    if not computers or #computers == 0 then return 0 end

    local rating = 0
    for _, computer in ipairs(computers) do
        rating = rating + computer:getRating()
    end

    return rating
end

-- *** COMMUNICATOR FUNCTIONS ***
function Entity:mgrCommunicatorDamageHealth(value)
    local commos = self.components.commo
    if not commos or #commos == 0 then return end

    -- Spread communicator damage amount evenly across all installed communicators that have health remaining
    local commoCount = 0
    for _, commo in ipairs(commos) do
        if commo:getHealth() > 0 then commoCount = commoCount + 1 end
    end

    local spreadValue = value / commoCount
    for _, commo in ipairs(commos) do
        if commo:getHealth() > 0 then
            commo:damageHealth(spreadValue)
            --printf("damaging %s by %s to %s (max %s)", commo:getName(), spreadValue, commo:getHealth(), commo:getHealthMax())
        end
    end
end

function Entity:mgrCommunicatorGetHealth()
    local commos = self.components.commo
    if not commos or #commos == 0 then return 0 end

    local health = 0
    for _, commo in ipairs(commos) do
        health = health + commo:getHealth()
    end

    return health
end

function Entity:mgrCommunicatorGetRating()
    local commos = self.components.commo
    if not commos or #commos == 0 then return 0 end

    local rating = 0
    for _, commo in ipairs(commos) do
        rating = rating + commo:getRating()
    end

    return rating
end

-- *** HULL FUNCTIONS ***
function Entity:mgrHullDamageHealth(amount)
    local hull = self.components.hull[1]
    assert(hull)
    --printf("reducing %s by %s to %s (max %s)", hull:getName(), amount, hull:getHealth(), hull:getHealthMax())
    return hull:damageHealth(amount)
end

function Entity:mgrHullGetHealth()
    local hull = self.components.hull[1]
    assert(hull)
    return hull:getHealth()
end

function Entity:mgrHullGetHealthMax()
    local hull = self.components.hull[1]
    assert(hull)
    return hull:getHealthMax()
end

function Entity:mgrHullGetHealthPercent()
    local hull = self.components.hull[1]
    assert(hull)
    return hull:getHealthPercent()
end

function Entity:mgrHullGetName()
    local hull = self.components.hull[1]
    assert(hull)
    return hull:getName()
end

function Entity:mgrHullSetHealth(healthCurr, healthMax)
    local hull = self.components.hull[1]
    assert(hull)
    return hull:setHealth(healthCurr, healthMax)
end

function Entity:mgrHullSetName(newName)
    local hull = self.components.hull[1]
    assert(hull)
    return hull:setName(newName)
end

-- *** INVENTORY FUNCTIONS ***
function Entity:mgrInventoryGetCapacity()
    -- Return maximum # of units (NOT count of any item) of inventory space
    local inventory = self.components.inventory
    if not inventory then return 0 end

    local invCap = 0
    for _, inv in ipairs(inventory) do
        invCap = invCap + inv:getInventoryCapacity()
    end

    return invCap
end

function Entity:mgrInventoryGetFreeMax(itemSize)
    -- Return # of storage units (NOT count per itemsize) of free space available for maximum number of items of itemSize
    local inventory = self.components.inventory
    if not inventory then return 0 end

    local iFreeMax = 0
    for _, inv in ipairs(inventory) do
        iFreeMax = iFreeMax + floor(inv:getInventoryFree() / itemSize)
    end
    iFreeMax = iFreeMax * itemSize

    return iFreeMax
end

function Entity:mgrInventoryGetFreeTotal()
    -- Return total size of all free inventory available
    local inventory = self.components.inventory
    if not inventory then return 0 end

    local iFreeTotal = 0
    for _, inv in ipairs(inventory) do
        iFreeTotal = iFreeTotal + inv:getInventoryFree()
    end

    return iFreeTotal
end

function Entity:mgrInventoryAddItem(item, count)
    local itemsAdded = false
    local inventory = self.components.inventory
    if inventory then
        if count > 0 then
            local mass = item:getMass()
            local itemsAddedCount = 0
            for _, inv in ipairs(inventory) do
                local compCount = floor(inv:getInventoryFree() / mass)
                for j = 1, compCount do
                    if inv:addItem(item, 1) then
                        itemsAddedCount = itemsAddedCount + 1
                    end
                    if itemsAddedCount == count then break end
                end
                if itemsAddedCount == count then break end
            end
            --printf("COMPONENTMGR:mgrInventoryAddItem - added %d units of %s to %s", itemsAddedCount, item:getName(), self:getName())

            if itemsAddedCount == count then itemsAdded = true end
        end
    end

    return itemsAdded
end

function Entity:mgrInventoryGetItemCount(item)
    local itemsCount = 0
    local inventory = self.components.inventory
    if inventory then
        for _, inv in ipairs(inventory) do
            itemsCount = itemsCount + inv:getItemCount(item)
        end
    end

    return itemsCount
end

function Entity:mgrInventoryGetItems(item)
    local itemsList = {}
    local inventory = self.components.inventory
    if inventory then
        for _, inv in ipairs(inventory) do
            local items = inv:getInventory()
            for item, count in pairs(items) do
                itemsList[item] = (itemsList[item] or 0) + count
            end
        end
    end

    return itemsList
end

function Entity:mgrInventoryHasItem(item, count)
    return self:mgrInventoryGetItemCount(item) >= count
end

function Entity:mgrInventoryRemoveItem(item, count)
    local itemsRemoved = false
    local inventory = self.components.inventory
    if inventory then
        if count > 0 then
            if self:mgrInventoryGetItemCount(item) >= count then
                local itemsRemovedCount = 0
                for _, inv in ipairs(inventory) do
                    while inv:removeItem(item, 1) do
                        itemsRemovedCount = itemsRemovedCount + 1
                        if itemsRemovedCount == count then break end
                    end
                    if itemsRemovedCount == count then break end
                end
                --printf("COMPONENTMGR:mgrInventoryRemoveItem - removed %d units of %s from %s", itemsRemovedCount, item:getName(), self:getName())

                if itemsRemovedCount == count then itemsRemoved = true end
            end
        end
    end

    return itemsRemoved
end

function Entity:mgrInventoryDebug(state)
    if not self:isDestroyed() then
        local inventoryList = self:mgrInventoryGetItems()
        local ctx = state.context
        ctx:text("Inventory for %s (capacity: %d/%d)", self:getName(), self:mgrInventoryGetFreeTotal(),
            self:mgrInventoryGetCapacity())
        ctx:indent()
        for k, v in pairs(inventoryList) do
            ctx:text("%d x %s (mass = %s)", v, k:getName(), k:getMass())
        end
        ctx:undent()
    end
end

-- *** SENSOR FUNCTIONS ***
function Entity:mgrSensorDamageHealth(value)
    local sensors = self.components.sensor
    if not sensors or #sensors == 0 then return end

    -- Spread sensor damage amount evenly across all installed sensors that have health remaining
    local sensorCount = 0
    for _, sensor in ipairs(sensors) do
        if sensor:getHealth() > 0 then sensorCount = sensorCount + 1 end
    end

    local spreadValue = value / sensorCount
    for _, sensor in ipairs(sensors) do
        if sensor:getHealth() > 0 then
            sensor:damageHealth(spreadValue)
            --printf("damaging %s by %s to %s (max %s)", sensor:getName(), spreadValue, sensor:getHealth(), sensor:getHealthMax())
        end
    end
end

function Entity:mgrSensorGetHealth()
    local sensors = self.components.sensor
    if not sensors or #sensors == 0 then return 0 end

    local health = 0
    for _, sensor in ipairs(sensors) do
        health = health + sensor:getHealth()
    end

    return health
end

function Entity:mgrSensorGetMappingRange()
    local sensors = self.components.sensor
    if not sensors or #sensors == 0 then return 0 end

    local range = 0
    for _, sensor in ipairs(sensors) do
        range = range + sensor:getMappingRange()
    end

    return range
end

function Entity:mgrSensorGetRating()
    local sensors = self.components.sensor
    if not sensors or #sensors == 0 then return 0 end

    local rating = 0
    for _, sensor in ipairs(sensors) do
        rating = rating + sensor:getRating()
    end

    return rating
end

function Entity:mgrSensorGetScanSpeed()
    local sensors = self.components.sensor
    if not sensors or #sensors == 0 then return 0 end

    local speed = 0
    for _, sensor in ipairs(sensors) do
        speed = speed + sensor:getScanSpeed()
    end

    return speed
end

function Entity:mgrSensorGetScanDetail()
    local sensors = self.components.sensor
    if not sensors or #sensors == 0 then return 0 end

    local detail = 0
    for _, sensor in ipairs(sensors) do
        detail = detail + sensor:getScanDetail()
    end

    return detail
end

function Entity:mgrSensorGetLockBreaking()
    local sensors = self.components.sensor
    if not sensors or #sensors == 0 then return 0 end

    local breaking = 0
    for _, sensor in ipairs(sensors) do
        breaking = breaking + sensor:getLockBreaking()
    end

    return breaking
end

-- *** SHIELD FUNCTIONS ***
function Entity:mgrShieldDamageHealth(value)
    local shields = self.components.shield
    if not shields or #shields == 0 then return end

    -- Spread shield damage amount evenly across all installed armors that have health remaining
    local shieldCount = 0
    for _, shield in ipairs(shields) do
        if shield:getHealth() > 0 then shieldCount = shieldCount + 1 end
    end

    local spreadValue = value / shieldCount
    for _, shield in ipairs(shields) do
        if shield:getHealth() > 0 then
            shield:damageHealth(spreadValue)
            --printf("damaging %s by %s to %s (max %s)", shield:getName(), spreadValue, shield:getHealth(), shield:getHealthMax())
        end
    end
end

function Entity:mgrShieldGetHealth()
    local shields = self.components.shield
    if not shields or #shields == 0 then return 0 end

    local health = 0
    for _, shield in ipairs(shields) do
        health = health + shield:getHealth()
    end

    return health
end

function Entity:mgrShieldGetStrength()
    local shields = self.components.shield
    if not shields or #shields == 0 then return 0 end

    local strength = 0
    for _, shield in ipairs(shields) do
        strength = strength + shield:getStrength()
    end

    return strength
end

function Entity:mgrShieldGetStrengthMax()
    local shields = self.components.shield
    if not shields or #shields == 0 then return 0 end

    local strengthMax = 0
    for _, cap in ipairs(shields) do
        strengthMax = strengthMax + cap:getStrengthMax()
    end

    return strengthMax
end

function Entity:mgrShieldGetStrengthPercent()
    local strength    = self:mgrShieldGetStrength()
    local strengthMax = self:mgrShieldGetStrengthMax()

    if strength == 0 or strengthMax == 0 then return 0 end

    return 100.0 * strength / strengthMax
end

function Entity:mgrShieldReduceStrength(value)
    local shields = self.components.shield
    if not shields or #shields == 0 then return end

    -- Spread shield reduction amount evenly across all installed shields that have strength remaining
    local shieldCount = 0
    for _, shield in ipairs(shields) do
        if shield:getStrength() > 0 then shieldCount = shieldCount + 1 end
    end

    local spreadValue = value / shieldCount
    for _, shield in ipairs(shields) do
        if shield:getStrength() > 0 then
            shield:reduceStrength(spreadValue)
            --printf("reducing %s by %s to %s (max %s)", shield:getName(), spreadValue, shield:getStrength(), shield:getStrengthMax())
        end
    end
end
