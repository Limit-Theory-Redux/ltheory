-- TODO : Rename to 'Container' / 'Contained'

local Entity = require('GameObjects.Entity')

local function broadcast(self, event)
    for i = #self.children, 1, -1 do
        if self.children[i].parent == self then
            self.children[i]:send(event)
        else
            self.children[i] = self.children[#self.children]
            self.children[#self.children] = nil
        end
    end
end

function Entity:addChild(child)
    assert(self.children)
    if child.parent == self then return end
    if child.parent then child.parent:removeChild(child) end
    insert(self.children, child)
    child.parent = self
    self:send(Event.ChildAdded(child))
    child:send(Event.AddedToParent(self))
end

function Entity:addChildren()
    assert(not self.children)
    self.children = {}
    self:register(Event.Broadcast, broadcast)
end

function Entity:getChildren()
    -- Return non-component children
    assert(self.children)
    local realChildren = {}
    for _, child in ipairs(self.children) do
        if child:hasTrackable() and child:isTrackable() then
            insert(realChildren, child)
        end
    end
    return realChildren
end

function Entity:getParent()
    return self.parent
end

function Entity:getRoot()
    local root = self
    while root:getParent() do
        root = root:getParent()
    end
    return root
end

function Entity:hasChildren()
    return self.children ~= nil
end

function Entity:iterChildren()
    -- Return iterated non-component children
    assert(self.children)
    local realChildren = {}
    for _, child in ipairs(self.children) do
        if child:hasTrackable() and child:isTrackable() then
            insert(realChildren, child)
        end
    end
    return ipairs(realChildren)
end

function Entity:removeChild(child)
    assert(self.children)

    local parent

    if not child.parent == self then
        -- go to highest layer
        parent = child.parent
        while parent ~= nil do
            local foundParent = parent.parent
            if foundParent then parent = foundParent end
        end
    end

    child.parent = nil
    self:send(Event.ChildRemoved(child))
    child:send(Event.RemovedFromParent(self))

    -- this make sure that a child is also removed from the root / system if it has a different parent than the system e.g. ship docked at station
    if parent then
        self:send(Event.ChildRemoved(child))
        child:send(Event.RemovedFromParent(self))
    end
end
