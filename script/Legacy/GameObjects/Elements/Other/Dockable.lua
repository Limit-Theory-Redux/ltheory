-- NOTE: "Dockable" means "this object can have objects docked to it,"
-- not "this object can dock at other objects."

-- NOTE: Requires the Children component
local Entity = require('Legacy.GameObjects.Entity')

local function destroyed(self, source)
    if self:getOwner() then self:getOwner():removeAsset(self) end

    -- Damage all ships docked here (and tell them who did it), then undock them
    local children = self:getChildren()
    for i = #children, 1, -1 do
        local e = children[i]
        if e:getType() and Config:getObjectInfo("object_types", e:getType()) == "Ship" then
            e:applyDamage(10, source)
            self:removeDocked(e)
            Log.Debug("%s forcibly undocked from Station %s", e:getName(), self:getName())
        end
    end
end

function Entity:addBannedShip(e)
    assert(self.dockable)
    insert(self.bannedShips, e)
end

function Entity:addDockable()
    assert(not self.dockable)
    self.dockable = true
    self.bannedShips = {}
    self:register(OldEvent.Destroyed, destroyed)
end

function Entity:addDocked(e)
    assert(self.dockable)

    -- for i, v in ipairs(e.actions) do
    --     Log.Debug("Pre-Dock %s Actions %d : %s", e:getName(), i, v:getName(e))
    -- end

    self:getParent():removeChild(e)
    self:addChild(e)

    -- for i, v in ipairs(e.actions) do
    --     Log.Debug("Post-Dock %s Actions %d : %s", e:getName(), i, v:getName(e))
    -- end

    e:setShipDocked(self) -- mark ship as docked to this entity
end

function Entity:getDockable()
    assert(self.dockable)
    return self.dockable
end

function Entity:getDocked(e)
    assert(self.dockable)
    return self:getChildren()
end

function Entity:hasDockable()
    return self.dockable ~= nil
end

function Entity:isBanned(e)
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

function Entity:isDockable()
    return self.dockable
end

function Entity:removeDocked(e)
    assert(self.dockable)

    -- for i, v in ipairs(e.actions) do
    --     Log.Debug("Pre-Undock %s Actions %d : %s", e:getName(), i, v:getName(e))
    -- end

    self:removeChild(e)
    self:getParent():addChild(e)

    -- for i, v in ipairs(e.actions) do
    --     Log.Debug("Post-Undock %s Actions %d : %s", e:getName(), i, v:getName(e))
    -- end

    e:setShipDocked(nil) -- mark ship as undocked
end

function Entity:setDockable()
    self.dockable = true
    Log.Debug("%s %s is now dockable", Config:getObjectInfo("object_types", self:getType()), self:getName())
end

function Entity:setUndockable()
    self.dockable = false
    Log.Debug("%s %s is now undockable", Config:getObjectInfo("object_types", self:getType()), self:getName())
end
