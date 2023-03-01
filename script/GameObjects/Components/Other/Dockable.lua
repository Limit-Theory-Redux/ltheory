-- NOTE: "Dockable" means "this object can have objects docked to it,"
--       not "this object can dock at other objects."

-- NOTE: Requires the Children component
local Entity = require('GameObjects.Entity')

local function destroyed (self, source)
  if self:getOwner() then self:getOwner():removeAsset(self) end

  local children = self:getChildren()
  for i = #children, 1, -1 do
    local e = children[i]
    e:damage(10, source)
    self:removeDocked(e)
  end
end

function Entity:addDockable ()
  assert(not self.dockable)
  self.dockable = true
  self:register(Event.Destroyed, destroyed)
end

function Entity:addDocked (e)
  assert(self.dockable)
  self:addChild(e)
  e:setShipDocked(true) -- mark ship as docked
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
  e:setShipDocked(false) -- mark ship as undocked
end
