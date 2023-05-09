#include "CollisionShape.h"
#include "PhysicsDefs.h"
#include "RigidBody.h"
#include "Trigger.h"

inline static bool _cppTrigger_IsAttached (Trigger* self) {
  return self->parent != 0;
}

inline static void _cppTrigger_SetFlag (Trigger* self, int flag, bool enable) {
  int flags = self->handle->getCollisionFlags();
  flags = enable ? flags | flag : flags & ~flag;
  self->handle->setCollisionFlags(flags);
}

inline static Trigger* _cppTrigger_Create (CollisionShape* shape) {
  Trigger* self = MemNewZero(Trigger);
  self->type           = PhysicsType_Trigger;
  self->handle         = new GhostObject();
  self->iShape         = shape->iShape;
  self->collisionGroup = CollisionGroup_Trigger;
  self->collisionMask  = CollisionMask_NoTriggers;

  self->handle->setCollisionShape(shape->base.handle);
  self->handle->setUserPointer(self);

  typedef btCollisionObject::CollisionFlags Flags;
  _cppTrigger_SetFlag(self, Flags::CF_NO_CONTACT_RESPONSE, true);

  return self;
}

Trigger* _cppTrigger_CreateBox (Vec3f* halfExtents) {
  CollisionShape* shape = _cppCollisionShape_CreateBox(halfExtents);
  return _cppTrigger_Create(shape);
}

void _cppTrigger_Free (Trigger* self) {
  if (self->physics)
    Fatal("_cppTrigger_Free: Object is still present in Physics.");

  CollisionShape* shape = _cppCollisionShape_GetCached(self->iShape);
  _cppCollisionShape_Free(shape);
  MemFree(self);
}

void _cppTrigger_GetBoundingBox (Trigger* self, Box3f* box) {
  CollisionShape* shape     = _cppCollisionShape_GetCached(self->iShape);
  btTransform     transform = self->handle->getWorldTransform();

  btVector3 min, max;
  shape->base.handle->getAabb(transform, min, max);
  box->lower = Vec3f_FromBullet(min);
  box->upper = Vec3f_FromBullet(max);
}

int _cppTrigger_GetContentsCount (Trigger* self) {
  return self->handle->getNumOverlappingObjects();
}

RigidBody* _cppTrigger_GetContents (Trigger* self, int index) {
  btCollisionObject* collisionObject = self->handle->getOverlappingObject(index);
  RigidBody*         rigidBody       = (RigidBody*) collisionObject->getUserPointer();
  return rigidBody;
}

void _cppTrigger_Attach (Trigger* self, RigidBody* parent, Vec3f* pos) {
  if (self->parent)
    Fatal("_cppTrigger_Attach: Trigger is already attached to an object.");

  self->handle->setIgnoreCollisionCheck(parent->handle, true);
  self->parent         = parent;
  self->transformLocal = btTransform(btQuaternion::getIdentity(), Vec3f_ToBullet(pos));

  self->next = parent->triggers;
  parent->triggers = self;
}

void _cppTrigger_Detach (Trigger* self, RigidBody* parent) {
  if (!self->parent)
    Fatal("_cppTrigger_Detach: Trigger is not attached to an object.");
  if (self->parent != parent)
    Fatal("_cppTrigger_Detach: Trigger is attached to a different object.");

  parent->triggers = self->next;
  self->next = 0;

  self->handle->setIgnoreCollisionCheck(parent->handle, false);
  self->parent         = 0;
  self->transformLocal = btTransform::getIdentity();
}

void _cppTrigger_SetCollisionMask (Trigger* self, int mask) {
  if (HAS_FLAG(mask, CollisionGroup_Trigger))
    Fatal("_cppTrigger_SetCollisionMask: Triggers may not collide with other CollisionGroup_Triggers");

  self->collisionMask = mask;

  if (self->physics) {
    btBroadphaseProxy* proxy = self->handle->getBroadphaseHandle();
    proxy->m_collisionFilterMask = mask;
  }
}

void _cppTrigger_SetPos (Trigger* self, Vec3f* pos) {
  if (_cppTrigger_IsAttached(self))
    Fatal("_cppTrigger_SetPos: Not allowed when attached to a RigidBody.");

  btTransform transform = self->handle->getWorldTransform();
  transform.setOrigin(Vec3f_ToBullet(pos));
  self->handle->setWorldTransform(transform);
}

void _cppTrigger_SetPosLocal (Trigger* self, Vec3f* pos) {
  if (!_cppTrigger_IsAttached(self))
    Fatal("_cppTrigger_SetPosLocal: Only allowed when attached to a RigidBody.");

  self->transformLocal.setOrigin(Vec3f_ToBullet(pos));
}

RigidBody* _cppTrigger_GetParent (Trigger* self) {
  return self->parent;
}

void _cppTrigger_Update (Trigger* self) {
  if (_cppTrigger_IsAttached(self)) {
    Assert(self->parent->physics);
    /* TODO: Should only be setting position! Whoops! */
    btTransform transform = self->parent->handle->getWorldTransform();
    self->handle->setWorldTransform(transform * self->transformLocal);
  }
}
