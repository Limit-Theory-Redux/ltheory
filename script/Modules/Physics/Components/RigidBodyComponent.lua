local Component = require("Core.ECS.Component")

---@class RigidBodyComponent: Component
---@overload fun(self:RigidBodyComponent, rigidBody: RigidBody): RigidBodyComponent subclass internal
---@overload fun(): RigidBodyComponent subclass external
local RigidBodyComponent = Subclass("RigidBodyComponent", Component, function(self, rigidBody)
    self:setComponentName("PhysicsRigidBody")
    self:setRigidBody(rigidBody)
end)

---@param rigidBody RigidBody
function RigidBodyComponent:setRigidBody(rigidBody)
    self.rigidBody = rigidBody
end

---@return RigidBody
function RigidBodyComponent:getRigidBody()
    return self.rigidBody
end

function RigidBodyComponent:applyForce(force)
    assert(self.rigidBody)
    self.rigidBody:applyForce(force)
end

function RigidBodyComponent:applyForceLocal(force)
    assert(self.rigidBody)
    force = self:getRot():mulV(force)
    self.rigidBody:applyForce(force)
end

function RigidBodyComponent:applyTorque(torque)
    assert(self.rigidBody)
    self.rigidBody:applyTorque(torque)
end

function RigidBodyComponent:applyTorqueLocal(torque)
    assert(self.rigidBody)
    torque = self:getRot():mulV(torque)
    self.rigidBody:applyTorque(torque)
end

function RigidBodyComponent:attach(child, pos, rot)
    assert(self.rigidBody)
    assert(child.body)
    self.rigidBody:attach(child.body, Vec3f(pos.x, pos.y, pos.z), rot)
    self:addChild(child)
end

function RigidBodyComponent:detach(child)
    assert(self.rigidBody)
    assert(child.body)
    self:removeChild(child)
    self.rigidBody:detach(child.body)
end

function RigidBodyComponent:getBoundingBoxLocal()
    assert(self.rigidBody)
    local box = Box3f()
    self.rigidBody:getBoundingBoxLocal(box)
    return box
end

function RigidBodyComponent:getBoundingBoxLocalCompound()
    assert(self.rigidBody)
    local box = Box3f()
    self.rigidBody:getBoundingBoxLocalCompound(box)
    return box
end

function RigidBodyComponent:getBoundingBox()
    assert(self.rigidBody)
    local box = Box3f()
    self.rigidBody:getBoundingBox(box)
    return box
end

function RigidBodyComponent:getBoundingBoxCompound()
    assert(self.rigidBody)
    local box = Box3f()
    self.rigidBody:getBoundingBoxCompound(box)
    return box
end

function RigidBodyComponent:getDistance(other)
    assert(self.rigidBody)
    assert(other.body)
    return self.rigidBody:distanceTo(other.body)
end

function RigidBodyComponent:getForward()
    assert(self.rigidBody)
    return self:getRot():getForward()
end

function RigidBodyComponent:getMass()
    assert(self.rigidBody)
    return self.rigidBody:getMass()
end

function RigidBodyComponent:getMinDistance(other)
    assert(self.rigidBody)
    assert(other.body)
    return math.max(0.0, self:getDistance(other.body)
        - self:getRadius()
        - other:getRadius())
end

function RigidBodyComponent:getPos()
    assert(self.rigidBody)
    local pos = Position()
    self.rigidBody:getPos(pos)
    return pos
end

function RigidBodyComponent:getPosLocal()
    assert(self.rigidBody)
    local pos = Position()
    self.rigidBody:getPosLocal(pos)
    return pos
end

function RigidBodyComponent:getRadius()
    assert(self.rigidBody)
    return self.rigidBody:getBoundingRadius()
end

function RigidBodyComponent:getRadiusCompound()
    assert(self.rigidBody)
    return self.rigidBody:getBoundingRadiusCompound()
end

function RigidBodyComponent:getRight()
    assert(self.rigidBody)
    return self:getRot():getRight()
end

function RigidBodyComponent:getRot()
    assert(self.rigidBody)
    local rot = Quat()
    self.rigidBody:getRot(rot)
    return rot
end

function RigidBodyComponent:getRotLocal()
    assert(self.rigidBody)
    local rot = Quat()
    self.rigidBody:getRotLocal(rot)
    return rot
end

function RigidBodyComponent:getScale()
    assert(self.rigidBody)
    return self.rigidBody:getScale()
end

function RigidBodyComponent:getSpeed()
    assert(self.rigidBody)
    return self.rigidBody:getSpeed()
end

function RigidBodyComponent:getToLocalMatrix(eye)
    assert(self.rigidBody)
    return self.rigidBody:getToLocalMatrix(eye)
end

function RigidBodyComponent:getToWorldMatrix(eye)
    assert(self.rigidBody)
    return self.rigidBody:getToWorldMatrix(eye)
end

function RigidBodyComponent:getParentBody()
    assert(self.rigidBody)
    return self.rigidBody:getParentBody()
end

function RigidBodyComponent:getUp()
    assert(self.rigidBody)
    return self:getRot():getUp()
end

function RigidBodyComponent:getVelocity()
    assert(self.rigidBody)
    local velocity = Vec3f()
    self.rigidBody:getVelocity(velocity)
    return velocity
end

function RigidBodyComponent:getVelocityLocal()
    assert(self.rigidBody)
    local velocity = Vec3f()
    self.rigidBody:getVelocity(velocity)
    velocity = self:getRot():inverse():mulV(velocity)
    return velocity
end

function RigidBodyComponent:getVelocityA()
    assert(self.rigidBody)
    local velocityA = Vec3f()
    self.rigidBody:getVelocityA(velocityA)
    return velocityA
end

function RigidBodyComponent:getVelocityALocal()
    assert(self.rigidBody)
    local velocityA = Vec3f()
    self.rigidBody:getVelocityA(velocityA)
    velocityA = self:getRot():inverse():mulV(velocityA)
    return velocityA
end

function RigidBodyComponent:modPos(dp)
    assert(self.rigidBody)
    local pos = self:getPos()
    self.rigidBody:setPos(pos + dp)
end

function RigidBodyComponent:modPosLocal(dp)
    assert(self.rigidBody)
    local pos = self:getPosLocal()
    self.rigidBody:setPosLocal(pos + dp)
end

function RigidBodyComponent:setCollidable(collidable)
    assert(self.rigidBody)
    self.rigidBody:setCollidable(collidable)
end

function RigidBodyComponent:setCollisionGroup(group)
    assert(self.rigidBody)
    self.rigidBody:setCollisionGroup(group)
end

function RigidBodyComponent:setCollisionMask(mask)
    assert(self.rigidBody)
    self.rigidBody:setCollisionMask(mask)
end

function RigidBodyComponent:setDrag(linear, angular)
    assert(self.rigidBody)
    self.rigidBody:setDrag(linear, angular)
end

function RigidBodyComponent:setFriction(friction)
    assert(self.rigidBody)
    self.rigidBody:setFriction(friction)
end

function RigidBodyComponent:setKinematic(kinematic)
    assert(self.rigidBody)
    self.rigidBody:setKinematic(kinematic)
end

function RigidBodyComponent:setMass(mass)
    assert(self.rigidBody)
    self.rigidBody:setMass(mass)
end

function RigidBodyComponent:setPos(pos)
    assert(self.rigidBody)
    self.rigidBody:setPos(pos)
end

function RigidBodyComponent:setPosLocal(pos)
    assert(self.rigidBody)
    self.rigidBody:setPosLocal(pos)
end

function RigidBodyComponent:setRestitution(restitution)
    assert(self.rigidBody)
    self.rigidBody:setRestitution(restitution)
end

function RigidBodyComponent:setRot(rot)
    assert(self.rigidBody)
    self.rigidBody:setRot(rot)
end

function RigidBodyComponent:setRotLocal(rot)
    assert(self.rigidBody)
    self.rigidBody:setRotLocal(rot)
end

function RigidBodyComponent:setSleepThreshold(linear, angular)
    assert(self.rigidBody)
    self.rigidBody:setSleepThreshold(linear, angular)
end

function RigidBodyComponent:setScale(scale)
    assert(self.rigidBody)
    self.rigidBody:setScale(scale)
end

function RigidBodyComponent:toLocal(pos)
    assert(self.rigidBody)
    local toLocal = self:getToLocalMatrix(pos)
    return toLocal:mulPoint(pos:toVec3f())
end

function RigidBodyComponent:toWorld(pos)
    assert(self.rigidBody)
    local ePos = self:getPos()
    local eRot = self:getRot()
    return
        ePos +
        eRot:getRight():scale(pos.x) +
        eRot:getUp():scale(pos.y) +
        eRot:getForward():scale(pos.z)
end

function RigidBodyComponent:toWorldScaled(pos)
    assert(self.rigidBody)
    local ePos = self:getPos()
    local eRot = self:getRot()
    local eScl = self:getScale()
    return
        ePos +
        eRot:getRight():scale(eScl * pos.x) +
        eRot:getUp():scale(eScl * pos.y) +
        eRot:getForward():scale(eScl * pos.z)
end

return RigidBodyComponent
