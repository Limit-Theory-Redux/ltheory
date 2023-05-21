-- NOTE: "Dockable" means "this object can have objects docked to it,"
--       not "this object can dock at other objects."

-- NOTE: Requires the Children component
local Entity = require('GameObjects.Entity')

local function destroyed (self, source)
  if self:getOwner() then self:getOwner():removeAsset(self) end

  local children = self:getChildren()
  for i = #children, 1, -1 do
    local e = children[i]
    if e:getType() and Config:getObjectInfo("object_types", e:getType()) == "Ship" then
      e:damage(10, source)
      self:removeDocked(e)
printf("%s forcibly undocked from Station %s", e:getName(), self:getName())
    end
  end
end

function Entity:addBannedShip (e)
  assert(self.dockable)
  insert(self.bannedShips, e)
end

function Entity:addDockable ()
  assert(not self.dockable)
  self.dockable = true
  self.bannedShips = {}
  self:register(Event.Destroyed, destroyed)
end

function Entity:addDocked (e)
  assert(self.dockable)
  self:addChild(e)
  e:setShipDocked(self) -- mark ship as docked to this entity
end

function Entity:getDockable ()
  assert(self.dockable)
  return self.dockable
end

function Entity:getDocked (e)
  assert(self.dockable)
  return self:getChildren()
end

function Entity:hasDockable ()
  return self.dockable ~= nil
end

function Entity:isBanned (e)
  local isBanned = false

  -- TODO: Extend this to cover whether an entire faction (and all its ships) is banned
  for _, ship in ipairs(self.bannedShips) do
    if ship == e then
      isBanned = true
      break
    end
  end

  return isBanned
end

function Entity:isDockable ()
  return self.dockable
end

function Entity:setDockable ()
  self.dockable = true
printf("%s %s is now dockable", Config:getObjectInfo("object_types", self:getType()), self:getName())
end

function Entity:setUndockable ()
  self.dockable = false
printf("%s %s is now undockable", Config:getObjectInfo("object_types", self:getType()), self:getName())
end

function Entity:removeDocked (e)
  assert(self.dockable)
  self:getParent():addChild(e)
  e:setShipDocked(nil) -- mark ship as undocked
  e:setPos(self:getPos())
end
