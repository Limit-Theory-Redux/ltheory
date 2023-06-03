local Entity = require('GameObjects.Entity')

-- This module integrates functionality for multiple components of each type installed
--    in a vessel (which currently includes Ship and Station but not Asteroid or Planet)

-- WARNING : Note the subtlety that isAlive and isDestroyed are NOT
--           complementary! An asteroid is not alive, but neither has it been
--           destroyed. Both 'alive' and 'destroyed' require an entity to have a Hull.

-- *** GENERAL FUNCTIONS ***
function Entity:addComponents ()
  assert(not self.components)
  self.components = {
                      hull        = {},
                      computer    = {},
                      sensor      = {},
                      lifeSupport = {},
                      capacitor   = {},
                      thruster    = {},
                      turret      = {},
                      bay         = {},
                      inventory   = {},
                      drone       = {},
                      shield      = {},
                      armor       = {},
                    }
end

-- TODO: Function interfaces for:
--         Computers
--         Sensors
--         Life Support
--         Drones

function Entity:isAlive ()
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

function Entity:isDestroyed ()
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

-- *** HULL FUNCTIONS ***
function Entity:mgrHullGetHull ()
  local hull = self.components.hull[1]
  assert(hull)
  return hull:getHealth()
end

function Entity:mgrHullGetHullMax ()
  local hull = self.components.hull[1]
  assert(hull)
  return hull:getHealthMax()
end

function Entity:mgrHullGetHullPercent ()
  local hull = self.components.hull[1]
  assert(hull)
  return hull:getHealthPercent()
end

function Entity:mgrHullGetName ()
  local hull = self.components.hull[1]
  assert(hull)
  return hull:getName()
end

function Entity:mgrHullReduceHull (amount)
  local hull = self.components.hull[1]
  assert(hull)
--printf("reducing %s by %s to %s (max %s)", hull:getName(), amount, hull:getHealth(), hull:getHealthMax())
  return hull:damageHealth(amount)
end

function Entity:mgrHullSetHull (healthCurr, healthMax)
  local hull = self.components.hull[1]
  assert(hull)
  return hull:setHealth(healthCurr, healthMax)
end

function Entity:mgrHullSetName (newName)
  local hull = self.components.hull[1]
  assert(hull)
  return hull:setName(newName)
end

-- *** INVENTORY FUNCTIONS ***
function Entity:mgrInventoryGetCapacity ()
  -- Return maximum # of units (NOT count) of space
  local inventory = self.components.inventory
  if not inventory then return 0 end

  local invCap = 0
  for _, inv in ipairs(inventory) do
    invCap = invCap + inv:getInventoryCapacity()
  end

  return invCap
end

function Entity:mgrInventoryGetFreeMax (itemSize)
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

function Entity:mgrInventoryGetFreeTotal ()
  -- Return total size of all free inventory available
  local inventory = self.components.inventory
  if not inventory then return 0 end

  local iFreeTotal = 0
  for _, inv in ipairs(inventory) do
    iFreeTotal = iFreeTotal + inv:getInventoryFree()
  end

  return iFreeTotal
end

function Entity:mgrInventoryAddItem (item, count)
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

function Entity:mgrInventoryGetItemCount (item)
  local itemsCount = 0
  local inventory = self.components.inventory
  if inventory then
    for _, inv in ipairs(inventory) do
      itemsCount = itemsCount + inv:getItemCount(item)
    end
  end

  return itemsCount
end

function Entity:mgrInventoryGetItems (item)
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

function Entity:mgrInventoryHasItem (item, count)
  return self:mgrInventoryGetItemCount(item) >= count
end

function Entity:mgrInventoryRemoveItem (item, count)
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

function Entity:mgrInventoryDebug (state)
  if not self:isDestroyed() then
    local inventoryList = self:mgrInventoryGetItems()
    local ctx = state.context
    ctx:text("Inventory for %s (capacity: %d/%d)", self:getName(), self:mgrInventoryGetFreeTotal(), self:mgrInventoryGetCapacity())
    ctx:indent()
    for k, v in pairs(inventoryList) do
      ctx:text("%d x %s (mass = %s)", v, k:getName(), k:getMass())
    end
    ctx:undent()
  end
end

-- *** CAPACITOR FUNCTIONS ***
function Entity:mgrCapacitorDischarge (amount)
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

function Entity:mgrCapacitorGetCharge ()
  local capacitors = self.components.capacitor
  if not capacitors then return 0 end

  local charge = 0
  for _, cap in ipairs(capacitors) do
    charge = charge + cap:getCharge()
  end

  return charge
end

function Entity:mgrCapacitorGetChargeMax ()
  local capacitors = self.components.capacitor
  if not capacitors then return 0 end

  local chargeMax = 0
  for _, cap in ipairs(capacitors) do
    chargeMax = chargeMax + cap:getChargeMax()
  end

  return chargeMax
end

function Entity:mgrCapacitorGetChargePercent ()
  local charge    = self:mgrCapacitorGetCharge()
  local chargeMax = self:mgrCapacitorGetChargeMax()

  if charge == 0 or chargeMax == 0 then return 0 end

  return 100.0 * charge / chargeMax
end

-- *** SHIELD FUNCTIONS ***
function Entity:mgrShieldGetShield ()
  local shields = self.components.shield
  if not shields or #shields == 0 then return 0 end

  local strength = 0
  for _, shield in ipairs(shields) do
    strength = strength + shield:getShield()
  end

  return strength
end

function Entity:mgrShieldGetShieldMax ()
  local shields = self.components.shield
  if not shields or #shields == 0 then return 0 end

  local strengthMax = 0
  for _, cap in ipairs(shields) do
    strengthMax = strengthMax + cap:getShieldMax()
  end

  return strengthMax
end

function Entity:mgrShieldGetShieldPercent ()
  local strength    = self:mgrShieldGetShield()
  local strengthMax = self:mgrShieldGetShieldMax()

  if strength == 0 or strengthMax == 0 then return 0 end

  return 100.0 * strength / strengthMax
end

function Entity:mgrShieldReduceShield (value)
  local shields = self.components.shield
  if not shields or #shields == 0 then return end

  -- Spread shield reduction amount evenly across all installed shields that have strength remaining
  local shieldCount = 0
  for _, shield in ipairs(shields) do
    if shield:getShield() > 0 then shieldCount = shieldCount + 1 end
  end

  local spreadValue = value / shieldCount
  for _, shield in ipairs(shields) do
    if shield:getShield() > 0 then
      shield:reduceShield(spreadValue)
--printf("reducing %s by %s to %s (max %s)", shield:getName(), spreadValue, shield:getShield(), shield:getShieldMax())
    end
  end
end

-- *** ARMOR FUNCTIONS ***
function Entity:mgrArmorGetArmor ()
  local armors = self.components.armor
  if not armors or #armors == 0 then return 0 end

  local health = 0
  for _, armor in ipairs(armors) do
    health = health + armor:getHealth()
  end

  return health
end

function Entity:mgrArmorGetArmorMax ()
  local armors = self.components.armor
  if not armors or #armors == 0 then return 0 end

  local healthMax = 0
  for _, armor in ipairs(armors) do
    healthMax = healthMax + armor:getHealthMax()
  end

  return healthMax
end

function Entity:mgrArmorGetArmorPercent ()
  local health    = self:mgrArmorGetArmor()
  local healthMax = self:mgrArmorGetArmorMax()

  if health == 0 or healthMax == 0 then return 0 end

  return 100.0 * health / healthMax
end

function Entity:mgrArmorReduceArmor (value)
  local armors = self.components.armor
  if not armors or #armors == 0 then return end

  -- Spread armor reduction amount evenly across all installed armors that have strength remaining
  local armorCount = 0
  for _, armor in ipairs(armors) do
    if armor:getHealth() > 0 then armorCount = armorCount + 1 end
  end

  local spreadValue = value / armorCount
  for _, armor in ipairs(armors) do
    if armor:getHealth() > 0 then
      armor:damageHealth(spreadValue)
--printf("reducing %s by %s to %s (max %s)", armor:getName(), spreadValue, armor:getHealth(), armor:getHealthMax())
    end
  end
end
