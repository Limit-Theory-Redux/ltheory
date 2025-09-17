--[[ TODO : Distinguish between attachment & containment; this should only
            happen on containment changes. ]]

--[[ TODO : Should we separate RB dynamics from collision detection? What would
            that look like? ]]

local Entity = require("Legacy.GameObjects.Entity")
local RigidBodyComponent = require("Modules.Physics.Components.RigidBodyComponent")

local bodyToEntity = {}

local function onAddedToParent(self, event)
    -- TODO : Should probably move this to the parent.
    if event.parent.physics ~= nil then
        event.parent.physics:addRigidBody(self.body)
        bodyToEntity[ptrToKey(self.body)] = self
    end
end

local function onRemovedFromParent(self, event)
    -- TODO : Should probably move this to the parent.
    if event.parent.physics ~= nil then
        event.parent.physics:removeRigidBody(self.body)
        bodyToEntity[ptrToKey(self.body)] = nil
    end
end

function Entity.fromRigidBody(body)
    return bodyToEntity[ptrToKey(body)]
end

function Entity:addRigidBody(isCollider, collisionMesh, colliderType)
    assert(not self.body)

    local body = nil
    if colliderType == Enums.ColliderType.Box then
        body = RigidBody.CreateBoxFromMesh(collisionMesh)
    elseif colliderType == Enums.ColliderType.ConvexHull then
        body = RigidBody.CreateConvexHullFromMesh(collisionMesh)
    elseif colliderType == Enums.ColliderType.ConvexDecomposition then
        body = RigidBody.CreateConvexDecompositionFromMesh(collisionMesh)
    elseif colliderType == Enums.ColliderType.Trimesh then
        body = RigidBody.CreateTrimeshFromMesh(collisionMesh)
    else
        -- Default case.
        body = RigidBody.CreateSphereFromMesh(collisionMesh)
    end

    self.entity:add(RigidBodyComponent(body))
    self.body = body

    self:register(OldEvent.AddedToParent, onAddedToParent)
    self:register(OldEvent.RemovedFromParent, onRemovedFromParent)
end

function Entity:applyForce(force)
    assert(self.body)
    self.body:applyForce(force)
end

function Entity:applyForceLocal(force)
    assert(self.body)
    force = self:getRot():mulV(force)
    self.body:applyForce(force)
end

function Entity:applyTorque(torque)
    assert(self.body)
    self.body:applyTorque(torque)
end

function Entity:applyTorqueLocal(torque)
    assert(self.body)
    torque = self:getRot():mulV(torque)
    self.body:applyTorque(torque)
end

function Entity:attach(child, pos, rot)
    assert(self.body)
    assert(child.body)
    self.body:attach(child.body, Vec3f(pos.x, pos.y, pos.z), rot)
    self:addChild(child)
end

function Entity:detach(child)
    assert(self.body)
    assert(child.body)
    self:removeChild(child)
    self.body:detach(child.body)
end

function Entity:getBoundingBoxLocal()
    assert(self.body)
    local box = Box3f()
    self.body:getBoundingBoxLocal(box)
    return box
end

function Entity:getBoundingBoxLocalCompound()
    assert(self.body)
    local box = Box3f()
    self.body:getBoundingBoxLocalCompound(box)
    return box
end

function Entity:getBoundingBox()
    assert(self.body)
    local box = Box3f()
    self.body:getBoundingBox(box)
    return box
end

function Entity:getBoundingBoxCompound()
    assert(self.body)
    local box = Box3f()
    self.body:getBoundingBoxCompound(box)
    return box
end

function Entity:getDistance(other)
    assert(self.body)
    assert(other.body)
    return self.body:distanceTo(other.body)
end

function Entity:getForward()
    assert(self.body)
    return self:getRot():getForward()
end

function Entity:getMass()
    assert(self.body)
    return self.body:getMass()
end

function Entity:getMinDistance(other)
    assert(self.body)
    assert(other.body)
    return math.max(0.0, self:getDistance(other.body)
        - self:getRadius()
        - other:getRadius())
end

function Entity:getPos()
    assert(self.body)
    local pos = Position()
    self.body:getPos(pos)
    return pos
end

function Entity:getPosLocal()
    assert(self.body)
    local pos = Position()
    self.body:getPosLocal(pos)
    return pos
end

function Entity:getRadius()
    assert(self.body)
    return self.body:getBoundingRadius()
end

function Entity:getRadiusCompound()
    assert(self.body)
    return self.body:getBoundingRadiusCompound()
end

function Entity:getRight()
    assert(self.body)
    return self:getRot():getRight()
end

function Entity:getRot()
    assert(self.body)
    local rot = Quat()
    self.body:getRot(rot)
    return rot
end

function Entity:getRotLocal()
    assert(self.body)
    local rot = Quat()
    self.body:getRotLocal(rot)
    return rot
end

function Entity:getScale()
    assert(self.body)
    return self.body:getScale()
end

function Entity:getSpeed()
    assert(self.body)
    return self.body:getSpeed()
end

function Entity:getToLocalMatrix(eye)
    assert(self.body)
    return self.body:getToLocalMatrix(eye)
end

function Entity:getToWorldMatrix(eye)
    assert(self.body)
    return self.body:getToWorldMatrix(eye)
end

function Entity:getParentBody()
    assert(self.body)
    return self.body:getParentBody()
end

function Entity:getUp()
    assert(self.body)
    return self:getRot():getUp()
end

function Entity:getVelocity()
    assert(self.body)
    local velocity = Vec3f()
    self.body:getVelocity(velocity)
    return velocity
end

function Entity:getVelocityLocal()
    assert(self.body)
    local velocity = Vec3f()
    self.body:getVelocity(velocity)
    velocity = self:getRot():inverse():mulV(velocity)
    return velocity
end

function Entity:getVelocityA()
    assert(self.body)
    local velocityA = Vec3f()
    self.body:getVelocityA(velocityA)
    return velocityA
end

function Entity:getVelocityALocal()
    assert(self.body)
    local velocityA = Vec3f()
    self.body:getVelocityA(velocityA)
    velocityA = self:getRot():inverse():mulV(velocityA)
    return velocityA
end

function Entity:modPos(dp)
    assert(self.body)
    local pos = self:getPos()
    self.body:setPos(pos + dp)
end

function Entity:modPosLocal(dp)
    assert(self.body)
    local pos = self:getPosLocal()
    self.body:setPosLocal(pos + dp)
end

function Entity:setCollidable(collidable)
    assert(self.body)
    self.body:setCollidable(collidable)
end

function Entity:setCollisionGroup(group)
    assert(self.body)
    self.body:setCollisionGroup(group)
end

function Entity:setCollisionMask(mask)
    assert(self.body)
    self.body:setCollisionMask(mask)
end

function Entity:setDrag(linear, angular)
    assert(self.body)
    self.body:setDrag(linear, angular)
end

function Entity:setFriction(friction)
    assert(self.body)
    self.body:setFriction(friction)
end

function Entity:setKinematic(kinematic)
    assert(self.body)
    self.body:setKinematic(kinematic)
end

function Entity:setMass(mass)
    assert(self.body)
    self.body:setMass(mass)
end

function Entity:setPos(pos)
    assert(self.body)
    self.body:setPos(pos)
end

function Entity:setPosLocal(pos)
    assert(self.body)
    self.body:setPosLocal(pos)
end

function Entity:setRestitution(restitution)
    assert(self.body)
    self.body:setRestitution(restitution)
end

function Entity:setRot(rot)
    assert(self.body)
    self.body:setRot(rot)
end

function Entity:setRotLocal(rot)
    assert(self.body)
    self.body:setRotLocal(rot)
end

function Entity:setSleepThreshold(linear, angular)
    assert(self.body)
    self.body:setSleepThreshold(linear, angular)
end

function Entity:setScale(scale)
    assert(self.body)
    self.body:setScale(scale)
end

function Entity:toLocal(pos)
    assert(self.body)
    local toLocal = self:getToLocalMatrix(pos)
    return toLocal:mulPoint(pos:toVec3f())
end

function Entity:toWorld(pos)
    assert(self.body)
    local ePos = self:getPos()
    local eRot = self:getRot()
    return
        ePos +
        eRot:getRight():scale(pos.x) +
        eRot:getUp():scale(pos.y) +
        eRot:getForward():scale(pos.z)
end

function Entity:toWorldScaled(pos)
    assert(self.body)
    local ePos = self:getPos()
    local eRot = self:getRot()
    local eScl = self:getScale()
    return
        ePos +
        eRot:getRight():scale(eScl * pos.x) +
        eRot:getUp():scale(eScl * pos.y) +
        eRot:getForward():scale(eScl * pos.z)
end

function Entity:getZone()
    assert(self.body)
    local system = self:getRoot()
    for index, zone in ipairs(system.zones) do
        for index, object in ipairs(zone:getObjectsInTrigger()) do
            if object == self then
                return zone
            end
        end
    end
end
