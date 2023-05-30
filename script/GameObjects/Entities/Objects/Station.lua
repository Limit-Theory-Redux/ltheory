local Entity = require('GameObjects.Entity')
local Material = require('GameObjects.Material')

local function damaged (self, event)
  local shipEntry = self:findInDamageList(event.source)
  if shipEntry ~= nil then
    shipEntry.damage = shipEntry.damage + event.amount
  else
    shipEntry = {
      ship = event.source,
      damage = event.amount
    }
    table.insert(self.shipDamageList, shipEntry)
  end

  if shipEntry.damage > 100 then
    if not self:isDestroyed() and self:getOwner() ~= shipEntry.ship then
      -- Nobody enjoys getting shot
      self:modDisposition(shipEntry.ship, -0.2)

      -- Possibly make this station undockable to its attacker
      if self:hasDockable() and self:isDockable() then
        if self:isHostileTo(shipEntry.ship) and not self:isBanned(shipEntry.ship) then
          self:distressCall(shipEntry.ship, 15000)
          --self:undockAndAttack(shipEntry.ship)
          self:addBannedShip(shipEntry.ship)
  printf("Station %s bans attacker %s", self:getName(), shipEntry.ship:getName())
        end
      end
    end
  end
end


local Station = subclass(Entity, function (self, seed)
  local mesh = Gen.StationOld(seed):managed()
  self:addActions()
  self:addAttackable(true)
  self:addCapacitor(10000, 10000, 100)
  self:addChildren()
  self:addDispositions()
  self:addDockable()
  self:addExplodable()
  self:addFlows()
  self:addHealth(1000, 10) -- 10000, 20
  self:addInventory(1e8)
  self:addMinable(false)
  self:addRigidBody(true, mesh)
  self:addTrackable(true)
  self:addVisibleMesh(mesh, Material.Metal())

  self:setDrag(100, 100) -- fix station in place
  self:setScale(100)
  self:setMass(1e7)

  self.explosionSize = 512 -- destroyed stations have visually larger explosions than ships
  self.shipDamageList = {}
  self.lastClearDamageTime = 0
  self.timer = 0
  self:register(Event.Update, Entity.updateStation)
  self:register(Event.Damaged, damaged)
end)

function Station:findInDamageList(ship)
  for i, shipEntry in ipairs(self.shipDamageList) do
    if shipEntry.ship == ship then
      return shipEntry
    end
  end
  return nil
end

function Station:distressCall (target, range)
  local owner = self:getOwner()
  for asset in owner:iterAssets() do
    if asset:getType() == Config:getObjectTypeByName("object_types", "Ship") and self:isHostileTo(target) and self:getDistance(asset) < range then
      local currentAction = asset:getCurrentAction()

      if (currentAction and not string.find(currentAction:getName(),"Attack")) or not currentAction then
        asset:pushAction(Actions.Attack(target))
        --print(asset:getName() .. " answering distress call of " .. self:getName())
      end
    end
  end
end

function Station:undockAndAttack(target)
  for key, ship in pairs(self:getDocked(self)) do
    self:removeDocked(ship)
    ship:pushAction(Actions.Attack(target))
  end
end

function Station:attackedBy (target)
  -- This station has been attacked, probably by a band of ragtag rebel scum who pose no threat
  -- TODO: Allow a number of "grace" hits that decay over time
  -- TODO: If and when stations are armed, modify this method to let the station shoot back
end

function Entity:updateStation(state)

  if self.timer > self.lastClearDamageTime + 30 then
    self.shipDamageList = {}
    self.lastClearDamageTime = self.timer
  end

  self.timer = self.timer + state.dt
end

return Station
