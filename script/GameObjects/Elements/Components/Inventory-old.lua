local Entity = require('GameObjects.Entity')
local SocketType = require('GameObjects.Entities.Ship.SocketType')

function Entity:getSocketType ()
  return SocketType.Inventory
end

function Entity:addInventory (capacity)
  assert(not self.inventory)
  assert(capacity)
  self.inventory = {}
  self.inventoryCapacity = capacity
  self.inventoryFree = capacity
--printf("Register: Inventory type = %s, handler = %s", Event.Debug, Entity.debugInventory)
  self:register(Event.Debug, Entity.debugInventory)
end

function Entity:addItem (item, count)
  assert(self.inventory)
  assert(count >= 0)

  local mass = count * item:getMass()
  if mass > self.inventoryFree then return false end -- no more room!

  self.inventoryFree = self.inventoryFree - mass
  self.inventory[item] = self:getItemCount(item) + count

--printf("Added %d units of item %s to inventory of object %s, count now = %d",
--      count, item:getName(), self:getName(), self.inventory[item])

  return true
end

function Entity:debugInventory (state)
  if not self:isDestroyed() then
    local ctx = state.context
    ctx:text("Credits on board: %d", self:getCredits())
    ctx:text("Inventory (capacity: %d/%d)", self:getInventoryFree(), self:getInventoryCapacity())
    ctx:indent()
    for k, v in pairs(self.inventory) do
      ctx:text("%d x %s", v, k:getName())
    end
    ctx:undent()
  end
end

function Entity:getInventory ()
  assert(self.inventory)
  return self.inventory
end

function Entity:getInventoryCapacity ()
  assert(self.inventory)
  return self.inventoryCapacity
end

function Entity:getInventoryFree ()
  assert(self.inventory)
  return self.inventoryFree
end

function Entity:getItemCount (item)
  assert(self.inventory)
  return self.inventory[item] or 0
end

function Entity:hasInventory ()
  return self.inventory ~= nil
end

function Entity:hasItem (item, count)
  assert(self.inventory)
  return self:getItemCount(item) >= count
end

function Entity:refreshInventory ()
  assert(self.inventory)
  self.inventoryFree = self.inventoryCapacity
  for k, v in pairs(self.inventory) do
    self.inventoryFree = self.inventoryFree - v * k:getMass()
  end
end

function Entity:removeItem (item, count)
  assert(self.inventory)
  assert(count >= 0)
  if self:getItemCount(item) < count then
    return false
  end

  local mass = count * item:getMass()
  self.inventoryFree = self.inventoryFree + mass
  self.inventory[item] = self:getItemCount(item) - count

--printf("Removed %d units of item %s to inventory of object %s, count now = %d",
--      count, item:getName(), self:getName(), self.inventory[item])

  if self.inventory[item] == 0 then
    self.inventory[item] = nil
  end

  return true
end

function Entity:setInventoryCapacity (capacity)
  assert(self.inventory)
  self.inventoryCapacity = capacity
  self:refreshInventory()
end
