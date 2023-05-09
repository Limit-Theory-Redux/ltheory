#include "Bullet.h"
#include "CollisionShape.h"
#include "PhxMemory.h"
#include "Physics.h"
#include "PhysicsDefs.h"
#include "RigidBody.h"
#include "Trigger.h"

/* --- Helper Functions ----------------------------------------------------- */

bool _cppRigidBody_IsChild (RigidBody* self) {
  return self->parent != 0 && self->parent != self;
}

bool _cppRigidBody_IsCompound (RigidBody* self) {
  return self->parent != 0;
}

bool _cppRigidBody_IsParent (RigidBody* self) {
  return self->parent != 0 && self->parent == self;
}

inline static void _cppRigidBody_SetFlag (RigidBody* self, int flag, bool enable) {
  btRigidBody* rigidBody = self->handle;

  int flags = rigidBody->getCollisionFlags();
  flags = enable ? flags | flag : flags & ~flag;
  rigidBody->setCollisionFlags(flags);
}

RigidBody* _cppRigidBody_GetPart (RigidBody* self, int iCompound) {
  Assert(_cppRigidBody_IsCompound(self));
  self = self->parent;
  while (self->iCompound != iCompound) self = self->next;
  return self;
}

btTransform _cppRigidBody_GetWorldTransform (RigidBody* self) {
  btTransform transform;

  if (!_cppRigidBody_IsChild(self)) {
    btRigidBody* rigidBody = self->handle;
    transform = rigidBody->getWorldTransform();
  } else {
    btRigidBody*    rigidBody = self->parent->handle;
    CollisionShape* cmpShape  = _cppCollisionShape_GetCached(self->iCompoundShape);
    transform = rigidBody->getWorldTransform();
    transform *= cmpShape->compound.handle->getChildTransform(self->iCompound);
  }

  return transform;
}

inline static void _cppRigidBody_RecalculateInertia (RigidBody* self) {
  /* NOTE : We use the parent's shape to calculate inertia. Compound shapes
            calculate inertia from the bounding box, not the actual shapes, so
            if we e.g. attach something to a ship using a sphere collision shape
            all of a sudden it will switch to inertia for a box and drastically
            change the controls. It looks like the proper solution would be to
            use btCompoundShape::calculatePrincipalAxisTransform to get the
            proper inertia and offset the center of mass. In turn, this means we
            have to get rid of our assumption that the parent has identity
            position within the compound. However, this seems unlikely to add
            much value to the game, so we aren't going to bother. */

  btRigidBody*    rigidBody = self->handle;
  CollisionShape* shape     = _cppCollisionShape_GetCached(self->iShape);

  btVector3 inertia(0, 0, 0);
  shape->base.handle->calculateLocalInertia(self->mass, inertia);
  /* HACK: Hulls are wrapped in btUniformScalingShape. When calculating inertia,
           first it's calculated for a unit sized version of the underlying
           shape, then the scaling wrapper multiplies is by the scale. However,
           inertia is rougly proportional to r^2 and this means the final
           calculation is off by an additional factor of scale. We fix this here
           to ensure wrapped hull has approximately the same inertia a plain
           scaled hull would. Otherwise inertia is flat out wrong and the
           presence of a btUniformScalingShape wrapper leads to wildly different
           ship controls for the same model. */
  if (shape->type == CollisionShapeType_Hull)
    inertia *= shape->hull.handle->getUniformScalingFactor();

  rigidBody->setMassProps(self->mass, inertia);
  rigidBody->updateInertiaTensor();
}

inline static RigidBody* _cppRigidBody_Create (CollisionShape* shape) {
  /* NOTE : We only create compounds through attaching. */
  Assert(shape->type != CollisionShapeType_Compound);

  RigidBody* self = MemNewZero(RigidBody);
  self->type           = PhysicsType_RigidBody;
  self->mass           = 1.0f;
  self->iCompound      = -1;
  self->iCompoundShape = -1;

  btVector3 inertia(0, 0, 0);
  shape->base.handle->calculateLocalInertia(self->mass, inertia);

  btRigidBody* rigidBody = new btRigidBody(self->mass, 0, shape->base.handle, inertia);
  rigidBody->setFlags(BT_DISABLE_WORLD_GRAVITY);
  rigidBody->setRestitution(0.4f);
  rigidBody->setUserPointer(self);

  self->handle         = rigidBody;
  self->iShape         = shape->iShape;
  self->collisionGroup = CollisionGroup_Default;
  self->collisionMask  = CollisionMask_All;
  return self;
}

/* --- Implementation ------------------------------------------------------- */

RigidBody* _cppRigidBody_CreateBox () {
  Vec3f halfExtents = { 1, 1, 1 };
  CollisionShape* shape = _cppCollisionShape_CreateBox(&halfExtents);
  return _cppRigidBody_Create(shape);
}

RigidBody* _cppRigidBody_CreateBoxFromMesh (Mesh* mesh) {
  CollisionShape* shape = _cppCollisionShape_CreateBoxFromMesh(mesh);
  return _cppRigidBody_Create(shape);
}

RigidBody* _cppRigidBody_CreateSphere () {
  CollisionShape* shape = _cppCollisionShape_CreateSphere(1);
  return _cppRigidBody_Create(shape);
}

RigidBody* _cppRigidBody_CreateSphereFromMesh (Mesh* mesh) {
  CollisionShape* shape = _cppCollisionShape_CreateSphereFromMesh(mesh);
  return _cppRigidBody_Create(shape);
}

RigidBody* _cppRigidBody_CreateHullFromMesh (Mesh* mesh) {
  CollisionShape* shape = _cppCollisionShape_CreateHullFromMesh(mesh);
  return _cppRigidBody_Create(shape);
}

inline static void _cppRigidBody_FreeImpl (RigidBody* self) {
  Trigger* trigger = self->triggers;
  while (trigger) {
    Trigger* toFree = trigger;
    trigger = trigger->next;
    _cppTrigger_Free(toFree);
  }

  CollisionShape* shape = _cppCollisionShape_GetCached(self->iShape);
  _cppCollisionShape_Free(shape);
  delete self->handle;
  MemFree(self);
}

void _cppRigidBody_Free (RigidBody* self) {
  if (_cppRigidBody_IsChild(self))
    Fatal("_cppRigidBody_Free: Not supported on children.");
  if (self->physics)
    Fatal("_cppRigidBody_Free: Object is still present in Physics.");

  if (_cppRigidBody_IsParent(self)) {
    RigidBody* child = self->next;
    while (child) {
      RigidBody* toFree = child;
      child = child->next;
      _cppRigidBody_FreeImpl(toFree);
    }

    CollisionShape* shape = _cppCollisionShape_GetCached(self->iCompoundShape);
    _cppCollisionShape_Free(shape);
  }

  _cppRigidBody_FreeImpl(self);
}

void _cppRigidBody_ApplyForce (RigidBody* self, Vec3f* force) {
  if (_cppRigidBody_IsChild(self))
    Fatal("_cppRigidBody_ApplyForce: Not supported on children.");

  btRigidBody* rigidBody = self->handle;
  rigidBody->applyCentralForce(Vec3f_ToBullet(force));
  rigidBody->activate();
}

void _cppRigidBody_ApplyTorque (RigidBody* self, Vec3f* torque) {
  if (_cppRigidBody_IsChild(self))
    Fatal("_cppRigidBody_ApplyTorque: Not supported on children.");

  btRigidBody* rigidBody = self->handle;
  rigidBody->applyTorque(Vec3f_ToBullet(torque));
  rigidBody->activate();
}

void _cppRigidBody_Attach (RigidBody* parent, RigidBody* child, Vec3f* pos, Quat* rot) {
  if (_cppRigidBody_IsChild(parent))
    Fatal("_cppRigidBody_Attach: Recursive attachment is not supported. Parent is already attached to something.");
  if (_cppRigidBody_IsCompound(child))
    Fatal("_cppRigidBody_Attach: Child is already part of a compound.");
  if (parent == child)
    Fatal("_cppRigidBody_Attach: Cannot attach object to itself.");
  if (child->physics)
    Fatal("_cppRigidBody_Attach: Child has not been removed from physics.");

  //Convert parent to a compound
  if (!_cppRigidBody_IsParent(parent)) {
    CollisionShape shapeDef = {};
    shapeDef.scale = 1.0f;
    shapeDef.type  = CollisionShapeType_Compound;
    CollisionShape* cmpShape = _cppCollisionShape_Create(shapeDef);

    btRigidBody*    pBody  = parent->handle;
    CollisionShape* pShape = _cppCollisionShape_GetCached(parent->iShape);
    btTransform     pTrans = btTransform::getIdentity();

    cmpShape->compound.handle->addChildShape(pTrans, pShape->base.handle);
    pBody->setCollisionShape(cmpShape->compound.handle);
    parent->iCompound      = 0;
    parent->iCompoundShape = cmpShape->iShape;
    parent->parent         = parent;
    _cppPhysics_FlushCachedRigidBodyData(parent->physics, parent);
  }

  /* NOTE : Position is relative to the unscaled parent. */
  CollisionShape* pShape   = _cppCollisionShape_GetCached(parent->iShape);
  CollisionShape* cmpShape = _cppCollisionShape_GetCached(parent->iCompoundShape);
  CollisionShape* cShape   = _cppCollisionShape_GetCached(child->iShape);
  btTransform     cTrans   = btTransform(Quat_ToBullet(rot), Vec3f_ToBullet(pos) * pShape->scale);

  //Insert child into the compound list
  child->parent = parent;
  child->next   = parent->next;
  parent->next  = child;

  //Add child to the compound
  cmpShape->compound.handle->addChildShape(cTrans, cShape->base.handle);
  child->iCompound      = cmpShape->compound.handle->getNumChildShapes() - 1;
  child->iCompoundShape = cmpShape->iShape;
}

void _cppRigidBody_Detach (RigidBody* parent, RigidBody* child) {
  if (child->parent != parent)
    Fatal("_cppRigidBody_Detach: Child is not attached to parent.");
  if (child == parent)
    Fatal("_cppRigidBody_Detach: Cannot detach object from itself.");

  btRigidBody*     pBody       = parent->handle;
  btRigidBody*     cBody       = child->handle;
  btCompoundShape* compound    = (btCompoundShape*) pBody->getCollisionShape();
  btTransform      cLocalTrans = compound->getChildTransform(child->iCompound);

  //Remove child from the compound
  { /* HACK : btCompoundShape does a 'remove fast' internally. */
    int iLast = compound->getNumChildShapes() - 1;
    RigidBody* last = parent;
    while (last->iCompound != iLast) last = last->next;
    last->iCompound = child->iCompound;
  }
  compound->removeChildShapeByIndex(child->iCompound);
  child->iCompound      = -1;
  child->iCompoundShape = -1;

  //Remove child from the compound list
  RigidBody* prev = parent;
  while (prev->next != child) prev = prev->next;
  prev->next    = child->next;
  child->next   = 0;
  child->parent = 0;

  //Apply current position, rotation, and velocity
  btTransform  cTrans = pBody->getWorldTransform() * cLocalTrans;
  btVector3    cVel   = pBody->getVelocityInLocalPoint(cLocalTrans.getOrigin());
  cBody->setWorldTransform(cTrans);
  cBody->setLinearVelocity(cVel);

  //Convert parent to single object
  if (parent->next == 0) {
    CollisionShape* pShape = _cppCollisionShape_GetCached(parent->iShape);

    compound->removeChildShapeByIndex(parent->iCompound);
    pBody->setCollisionShape(pShape->base.handle);
    parent->iCompound      = -1;
    parent->iCompoundShape = -1;
    parent->parent         = 0;
    _cppPhysics_FlushCachedRigidBodyData(parent->physics, parent);
  } else {
    compound->recalculateLocalAabb();
  }
}

/* TODO: Should scale be inverted? */
Matrix* _cppRigidBody_GetToLocalMatrix (RigidBody* self) {
  float       scale     = _cppRigidBody_GetScale(self);
  btTransform transform = _cppRigidBody_GetWorldTransform(self).inverse();
  Matrix_FromTransform(&transform, &self->mat, scale);
  return &self->mat;
}

Matrix* _cppRigidBody_GetToWorldMatrix (RigidBody* self) {
  float       scale     = _cppRigidBody_GetScale(self);
  btTransform transform = _cppRigidBody_GetWorldTransform(self);
  Matrix_FromTransform(&transform, &self->mat, scale);
  return &self->mat;
}

void _cppRigidBody_SetCollidable (RigidBody* self, bool collidable) {
  typedef btCollisionObject::CollisionFlags Flags;
  _cppRigidBody_SetFlag(self, Flags::CF_NO_CONTACT_RESPONSE, !collidable);

  if (self->physics) {
    btRigidBody*       rigidBody = self->handle;
    btBroadphaseProxy* proxy     = rigidBody->getBroadphaseHandle();

    proxy->m_collisionFilterGroup = collidable ? self->collisionGroup : 0;
  }
}

void _cppRigidBody_SetCollisionGroup (RigidBody* self, int group) {
  btRigidBody* rigidBody = self->handle;
  self->collisionGroup = group;

  if (self->physics) {
    typedef btCollisionObject::CollisionFlags Flags;
    btBroadphaseProxy* proxy = rigidBody->getBroadphaseHandle();

    int  flags      = rigidBody->getCollisionFlags();
    bool collidable = !HAS_FLAG(flags, Flags::CF_NO_CONTACT_RESPONSE);
    proxy->m_collisionFilterGroup = collidable ? self->collisionGroup : 0;
  }
}

void _cppRigidBody_SetCollisionMask (RigidBody* self, int mask) {
  btRigidBody* rigidBody = self->handle;
  self->collisionMask = mask;

  if (self->physics) {
    btBroadphaseProxy* proxy = rigidBody->getBroadphaseHandle();
    proxy->m_collisionFilterMask = mask;
  }
}

void _cppRigidBody_SetDrag (RigidBody* self, float linear, float angular) {
  btRigidBody* rigidBody = self->handle;
  rigidBody->setDamping(linear, angular);
}

void _cppRigidBody_SetFriction (RigidBody* self, float friction) {
  btRigidBody* rigidBody = self->handle;
  rigidBody->setFriction(friction);
}

void _cppRigidBody_SetKinematic (RigidBody* self, bool kinematic) {
  typedef btCollisionObject::CollisionFlags Flags;
  _cppRigidBody_SetFlag(self, Flags::CF_KINEMATIC_OBJECT, kinematic);
}

float _cppRigidBody_GetMass (RigidBody* self) {
  return self->mass;
}

void _cppRigidBody_SetMass (RigidBody* self, float mass) {
  self->mass = mass;
  _cppRigidBody_RecalculateInertia(self);
}

void _cppRigidBody_GetPos (RigidBody* self, Vec3f* pos) {
  btTransform transform = _cppRigidBody_GetWorldTransform(self);
  *pos = Vec3f_FromBullet(transform.getOrigin());
}

void _cppRigidBody_GetPosLocal (RigidBody* self, Vec3f* pos) {
  if (!_cppRigidBody_IsChild(self))
    Fatal("_cppRigidBody_GetPosLocal: Only allowed on children.");

  float            pScale   = _cppRigidBody_GetScale(self->parent);
  CollisionShape*  cmpShape = _cppCollisionShape_GetCached(self->iCompoundShape);
  btCompoundShape* compound = cmpShape->compound.handle;
  btTransform& transform = compound->getChildTransform(self->iCompound);
  *pos = Vec3f_FromBullet((1.0f / pScale) * transform.getOrigin());
}

void _cppRigidBody_SetPos (RigidBody* self, Vec3f* pos) {
  if (_cppRigidBody_IsChild(self))
    Fatal("_cppRigidBody_SetPos: Not allowed on children.");

  btRigidBody* rigidBody = self->handle;
  btTransform  transform = rigidBody->getWorldTransform();
  transform.setOrigin(Vec3f_ToBullet(pos));
  rigidBody->setWorldTransform(transform);
}

void _cppRigidBody_SetPosLocal (RigidBody* self, Vec3f* pos) {
  if (!_cppRigidBody_IsChild(self))
    Fatal("_cppRigidBody_SetPosLocal: Only allowed on children.");

  float            pScale   = _cppRigidBody_GetScale(self->parent);
  CollisionShape*  cmpShape = _cppCollisionShape_GetCached(self->iCompoundShape);
  btCompoundShape* compound = cmpShape->compound.handle;
  btTransform transform = compound->getChildTransform(self->iCompound);
  transform.setOrigin(pScale * Vec3f_ToBullet(pos));
  compound->updateChildTransform(self->iCompound, transform);
}

void _cppRigidBody_GetBoundingBox (RigidBody* self, Box3f* box) {
  CollisionShape* shape     = _cppCollisionShape_GetCached(self->iShape);
  btTransform     transform = _cppRigidBody_GetWorldTransform(self);

  btVector3 min, max;
  shape->base.handle->getAabb(transform, min, max);
  box->lower = Vec3f_FromBullet(min);
  box->upper = Vec3f_FromBullet(max);
}

void _cppRigidBody_GetBoundingBoxCompound (RigidBody* self, Box3f* box) {
  if (!_cppRigidBody_IsParent(self))
    Fatal("_cppRigidBody_GetBoundingBoxCompound: Only enabled for parents.");

  CollisionShape* cmpShape  = _cppCollisionShape_GetCached(self->iCompoundShape);
  btTransform     transform = _cppRigidBody_GetWorldTransform(self);

  btVector3 min, max;
  cmpShape->base.handle->getAabb(transform, min, max);
  box->lower = Vec3f_FromBullet(min);
  box->upper = Vec3f_FromBullet(max);
}

void _cppRigidBody_GetBoundingBoxLocal (RigidBody* self, Box3f* box) {
  CollisionShape* shape     = _cppCollisionShape_GetCached(self->iShape);
  btTransform     transform = btTransform::getIdentity();

  btVector3 min, max;
  shape->base.handle->getAabb(transform, min, max);
  box->lower = Vec3f_FromBullet(min);
  box->upper = Vec3f_FromBullet(max);
}

void _cppRigidBody_GetBoundingBoxLocalCompound (RigidBody* self, Box3f* box) {
  if (!_cppRigidBody_IsParent(self))
    Fatal("_cppRigidBody_GetBoundingBoxLocalCompound: Only enabled for parents.");

  CollisionShape* cmpShape  = _cppCollisionShape_GetCached(self->iCompoundShape);
  btTransform     transform = btTransform::getIdentity();

  btVector3 min, max;
  cmpShape->base.handle->getAabb(transform, min, max);
  box->lower = Vec3f_FromBullet(min);
  box->upper = Vec3f_FromBullet(max);
}

float _cppRigidBody_GetBoundingRadius (RigidBody* self) {
  CollisionShape* shape = _cppCollisionShape_GetCached(self->iShape);

  float radius = 0;
  switch (shape->type) {
    /* NOTE : btSphereShape doesn't override the virtual function
              btCollisionShape::getBoundingSphere so it falls back to
              calculating a sphere that encompases the local bounding box that
              encompases the original sphere. Lovely. */
    case CollisionShapeType_Sphere: {
      radius = shape->scale * shape->sphere.radius;
      break;
    }

    default: {
      btVector3 center;
      shape->base.handle->getBoundingSphere(center, radius);
      radius += center.length();
      break;
    }
  }
  return radius;
}

float _cppRigidBody_GetBoundingRadiusCompound (RigidBody* self) {
  if (!_cppRigidBody_IsParent(self))
    Fatal("_cppRigidBody_GetBoundingBoxCompound: Only enabled for parents.");

  CollisionShape* cmpShape = _cppCollisionShape_GetCached(self->iCompoundShape);
  float radius;
  btVector3 center;
  cmpShape->base.handle->getBoundingSphere(center, radius);
  radius += center.length();
  return radius;
}

RigidBody* _cppRigidBody_GetParentBody (RigidBody* self) {
  return self->parent == self ? 0 : self->parent;
}

void _cppRigidBody_SetRestitution (RigidBody* self, float restitution) {
  btRigidBody* rigidBody = self->handle;
  rigidBody->setRestitution(restitution);
}

void _cppRigidBody_GetRot (RigidBody* self, Quat* rot) {
  btTransform transform = _cppRigidBody_GetWorldTransform(self);
  *rot = Quat_FromBullet(transform.getRotation());
}

void _cppRigidBody_GetRotLocal (RigidBody* self, Quat* rot) {
  if (!_cppRigidBody_IsChild(self))
    Fatal("_cppRigidBody_GetRotLocal: Only allowed on children.");

  CollisionShape*  cmpShape = _cppCollisionShape_GetCached(self->iCompoundShape);
  btCompoundShape* compound = cmpShape->compound.handle;
  btTransform& transform = compound->getChildTransform(self->iCompound);
  *rot = Quat_FromBullet(transform.getRotation());
}

void _cppRigidBody_SetRot (RigidBody* self, Quat* rot) {
  if (_cppRigidBody_IsChild(self))
    Fatal("_cppRigidBody_SetRot: Not allowed on children.");

  btRigidBody* rigidBody = self->handle;
  btTransform  transform = rigidBody->getWorldTransform();
  transform.setRotation(Quat_ToBullet(rot));
  rigidBody->setWorldTransform(transform);
}

void _cppRigidBody_SetRotLocal (RigidBody* self, Quat* rot) {
  if (!_cppRigidBody_IsChild(self))
    Fatal("_cppRigidBody_SetRotLocal: Only allowed on children.");

  CollisionShape*  cmpShape = _cppCollisionShape_GetCached(self->iCompoundShape);
  btCompoundShape* compound = cmpShape->compound.handle;
  btTransform transform = compound->getChildTransform(self->iCompound);
  transform.setRotation(Quat_ToBullet(rot));
  compound->updateChildTransform(self->iCompound, transform);
}

float _cppRigidBody_GetScale (RigidBody* self) {
  /* NOTE : Only uniform scale is supported. */
  CollisionShape* shape = _cppCollisionShape_GetCached(self->iShape);
  return shape->scale;
}

void _cppRigidBody_SetScale (RigidBody* self, float scale) {
  /* NOTE : Only uniform scale is supported. */

  /* NOTE : Since scale is not inherited and the Bullet API for scaling a
            compound shape sucks, when a parent object is rescaled we only
            rescale its individual shape. */

  /* NOTE : We scale the positions of children to maintain their position
            relative to the parent. */

  btRigidBody* rigidBody = self->handle;

  CollisionShape shapeDef = *_cppCollisionShape_GetCached(self->iShape);
  float scaleRatio = scale / shapeDef.scale;
  shapeDef.scale = scale;
  CollisionShape* shape = _cppCollisionShape_Create(shapeDef);

  if (!_cppRigidBody_IsCompound(self)) {
    rigidBody->setCollisionShape(shape->base.handle);
    self->iShape = shape->iShape;
    _cppRigidBody_RecalculateInertia(self);

  } else {
    CollisionShape*  cmpShape = _cppCollisionShape_GetCached(self->iCompoundShape);
    btCompoundShape* compound = cmpShape->compound.handle;

    //Children keep the same relative position
    if (_cppRigidBody_IsParent(self)) {
      RigidBody* child = self->next;
      while (child) {
        btTransform& cTrans = compound->getChildTransform(child->iCompound);
        cTrans.getOrigin() *= scaleRatio;
        compound->updateChildTransform(child->iCompound, cTrans, false);
        child = child->next;
      }
    }

    btTransform transform = compound->getChildTransform(self->iCompound);
    compound->removeChildShapeByIndex(self->iCompound);
    compound->addChildShape(transform, shape->base.handle);
    int iLast = compound->getNumChildShapes() - 1;
    { /* HACK : btCompoundShape does a 'remove fast' internally. */
      RigidBody* last = self->parent;
      while (last->iCompound != iLast) last = last->next;
      last->iCompound = self->iCompound;
    }
    self->iCompound = iLast;
    self->iShape    = shape->iShape;

    /* NOTE: removeChildShape calls recalculateLocalAabb but
             removeChildShapeByIndex does not. addChildShape updates the AABB
             with the new shape only, it is not a full recalculation. So, if our
             new scalle is smaller than our old scale the AABB won't shrink as
             expected. Thus, we need to force a full recalculate. */
    compound->recalculateLocalAabb();
    if (_cppRigidBody_IsParent(self))
      _cppRigidBody_RecalculateInertia(self);
  }
}

void _cppRigidBody_SetSleepThreshold (RigidBody* self, float linear, float angular) {
  btRigidBody* rigidBody = self->handle;
  rigidBody->setSleepingThresholds(linear, angular);
}

float _cppRigidBody_GetSpeed (RigidBody* self) {
  if (_cppRigidBody_IsChild(self))
    Fatal("_cppRigidBody_GetSpeed: Not supported on children.");

  btRigidBody* rigidBody = self->handle;
  float speed = rigidBody->getLinearVelocity().length();
  return speed;
}

void _cppRigidBody_GetVelocity (RigidBody* self, Vec3f* velocity) {
  if (_cppRigidBody_IsChild(self))
    Fatal("_cppRigidBody_GetVelocity: Not supported on children.");

  btRigidBody* rigidBody = self->handle;
  *velocity = Vec3f_FromBullet(rigidBody->getLinearVelocity());
}

void _cppRigidBody_GetVelocityA (RigidBody* self, Vec3f* velocityA) {
  if (_cppRigidBody_IsChild(self))
    Fatal("_cppRigidBody_GetVelocityA: Not supported on children.");

  btRigidBody* rigidBody = self->handle;
  *velocityA = Vec3f_FromBullet(rigidBody->getAngularVelocity());
}

/* NOTE : We assume the parent has identity position and rotation within the
          compound. */

/* NOTE : Detach could be made O(1) if a doubly linked list were used in place
          of the singly linked list. (Attach is O(1) in both cases.) */

/* NOTE : Free strategy: Freed parents will free all children. Children cannot
          be freed directly. Rigidbodies will not be removed from physics when
          freed, this must be done manually. */

/* NOTE : Coumans says btConvexHullShape is more efficient than
          btConvexPointCloudShape.
          https://pybullet.org/Bullet/phpBB3/viewtopic.php?t=3102 */

/* NOTE : btConvexHullShape duplicates the vertex positions.
          btConvexTriangleMeshShape does not, but the class comment says
          it's less efficient. */

/* NOTE : btShapeHull can be used to reduce the number of hull vertices.
          http://www.bulletphysics.org/mediawiki-1.5.8/index.php?title=BtShapeHull_vertex_reduction_utility */

/* NOTE : btConvexHull, btCompoundShape, and btUniformScaling all have shit
          implementations and APIs. I'm quite sure they are hurting performance
          and generally being obtuse pains in the dick. Josh, when you get
          annoyed with Bullet and start trying to optimize it, I'd strongly
          consider implementing these from scratch. */

/* TODO : Pool allocate Bullet objects if possible. Does Bullet have a pooling
          mechanism? Use our own arrays? */

/* TODO : btUniformScalingShape::getAabb looks like it might be quite slow. It's
          not clear why it doesn't simply forward the call to the wrapped shape
          and scale the result. */
