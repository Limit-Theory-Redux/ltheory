local Entity = require('Legacy.GameObjects.Entity')

local function onAddedToParent(self, event)
    -- TODO : Should probably move this to the parent.
    if event.parent.physics ~= nil then
        event.parent.physics:addTrigger(self.trigger)
    end
end

local function onRemovedFromParent(self, event)
    -- TODO : Should probably move this to the parent.
    if event.parent.physics ~= nil then
        event.parent.physics:removeTrigger(self.trigger)
    end
end

function Entity:addTrigger(halfExtents)
    assert(not self.trigger)
    self.trigger = Trigger.CreateBox(halfExtents)

    self:register(OldEvent.AddedToParent, onAddedToParent)
    self:register(OldEvent.RemovedFromParent, onRemovedFromParent)
end

-- TODO : Deal with naming conflicts
function Entity:triggerAttach(parent, pos)
    assert(self.trigger)
    self.trigger:attach(parent, pos)
end

function Entity:triggerDetach(parent)
    assert(self.trigger)
    self.trigger:detach(parent)
end

function Entity:getContentsCount()
    assert(self.trigger)
    return self.trigger:getContentsCount()
end

function Entity:getContents(index)
    assert(self.trigger)
    return self.trigger:getContents(index)
end

function Entity:triggerSetCollisionMask(mask)
    assert(self.trigger)
    self.trigger:setCollisionMask(mask)
end

function Entity:triggerSetPos(pos)
    assert(self.trigger)
    self.trigger:setPos(pos)
end

function Entity:triggerSetPosLocal(pos)
    assert(self.trigger)
    self.trigger:setPosLocal(pos)
end

--Returns all objects of given type within the zone
function Entity:getObjectsInTrigger()
    --Gets rigidbody count inside the zone's trigger
    local contentsCount = self:getContentsCount()
    local objects = {}

    --Loops through the trigger's objects for every found object and gets the rigidbody from libphx
    --To get the rigidbody we need to provide the rigidbody's index
    for i = 1, contentsCount do
        local rb = self:getContents(i - 1)
        if not rb then goto skip end
        local e = Entity.fromRigidBody(rb)

        if not e then goto skip end
        table.insert(objects, e)
        ::skip::
    end
    return objects
end

--Returns all objects of given type within the zone
function Entity:getObjectsInTriggerByType(type)
    --Gets rigidbody count inside the zone's trigger
    local contentsCount = self:getContentsCount()
    local objects = {}

    --Loops through the trigger's objects for every found object and gets the rigidbody from libphx
    --To get the rigidbody we need to provide the rigidbody's index
    for i = 1, contentsCount do
        local rb = self:getContents(i - 1)

        if not rb then goto skip end
        local e = Entity.fromRigidBody(rb)

        if not e then goto skip end

        if e:getType() == Config:getObjectTypeByName("object_types", type) then
            table.insert(objects, e)
        end
        ::skip::
    end
    return objects
end
