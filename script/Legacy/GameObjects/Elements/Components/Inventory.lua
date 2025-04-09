local Entity      = require('Legacy.GameObjects.Entity')
local BasicShapes = require('Legacy.Systems.Gen.ShapeLib.BasicShapes')
local SocketType  = require('Legacy.GameObjects.Entities.Ship.SocketType')

local shared
local rng         = RNG.FromTime()

local Inventory   = Subclass("Inventory", Entity, function(self)
    -- Required to be able to use plugs
    if not shared then
        shared = {}
        shared.mesh = BasicShapes.Prism(2, 3):finalize()
        shared.mesh:computeNormals()
        shared.mesh:computeAO(0.1)
    end

    self:addRigidBody(true, shared.mesh)
    self:addVisibleMesh(shared.mesh, Material.Debug())

    -- OK, back now to what Inventory actually requires
    self.name          = Config.gen.compInventoryStats.name
    self.healthCurr    = Config.gen.compInventoryStats.healthCurr
    self.healthMax     = Config.gen.compInventoryStats.healthMax
    self.inventoryCap  = Config.gen.compInventoryStats.capacity
    self.inventoryFree = Config.gen.compInventoryStats.capacity
    self.stateroom     = Config.gen.compInventoryStats.stateroom

    self.inventory     = {}
end)

function Inventory:getSocketType()
    return SocketType.Inventory
end

-- Inventory Slot Health --

function Inventory:damageHealth(amount)
    if self.healthCurr - amount < 1e-6 then
        self.healthCurr = 0.0
    else
        self.healthCurr = self.healthCurr - amount
    end
    --Log.Debug("Vessel %s inventory takes %s damage, %s remaining", self:getName(), amount, self.healthCurr)
    -- TODO: Add a chance to damage or remove inventory items in the transport pod that was damaged
end

function Inventory:getHealth()
    return self.healthCurr or 0.0
end

function Inventory:getHealthMax()
    return self.healthMax or 0.0
end

function Inventory:getHealthPercent()
    if self.healthMax < 1e-6 then return 0.0 end
    return 100.0 * self.healthCurr / self.healthMax
end

function Inventory:getName()
    return self.name
end

function Inventory:setHealth(value, max)
    if self.healthCurr > value then
        -- TODO: possibly remove inventory item(s) if the inventory capacity has become too small
    end
    self.healthCurr = value
    self.healthMax = floor(max)
end

function Inventory:setName(newName)
    self.name = newName
end

-- Inventory Slot Capacity --

function Inventory:getInventory()
    assert(self.inventory)
    return self.inventory
end

function Inventory:getInventoryCapacity()
    assert(self.inventory)
    return self.inventoryCap
end

function Inventory:getInventoryFree()
    assert(self.inventory)
    return self.inventoryFree
end

function Inventory:hasInventory()
    return self.inventory ~= nil
end

function Inventory:refreshInventory()
    assert(self.inventory)
    self.inventoryFree = self.inventoryCap
    for k, v in pairs(self.inventory) do
        self.inventoryFree = self.inventoryFree - v * k:getMass()
    end
end

function Inventory:setInventoryCapacity(capacity)
    assert(self.inventory)
    self.inventoryCap = capacity
    self:refreshInventory()
end

-- Items --

function Inventory:addItem(item, count)
    assert(self.inventory)
    assert(count >= 0)

    local mass = count * item:getMass()
    if mass > self.inventoryFree then return false end -- no more room!

    self.inventoryFree = self.inventoryFree - mass
    self.inventory[item] = self:getItemCount(item) + count

    --Log.Debug("Inventory:addItem() - Added %d units of item %s to inventory of object %s, count now = %d",
    --count, item:getName(), self:getName(), self.inventory[item])

    return true
end

function Inventory:getItemCount(item)
    assert(self.inventory)
    return self.inventory[item] or 0
end

function Inventory:hasItem(item, count)
    assert(self.inventory)
    return self:getItemCount(item) >= count
end

function Inventory:removeItem(item, count)
    assert(self.inventory)
    assert(count >= 0)
    if self:getItemCount(item) < count then
        return false
    end

    local mass = count * item:getMass()
    self.inventoryFree = self.inventoryFree + mass
    self.inventory[item] = self:getItemCount(item) - count

    --Log.Debug("Removed %d units of item %s to inventory of object %s, count now = %d",
    -- count, item:getName(), self:getName(), self.inventory[item])

    if self.inventory[item] == 0 then
        self.inventory[item] = nil
    end

    return true
end

return Inventory
