#ifndef PHX_CollisionShape
#define PHX_CollisionShape
#define __FFI_IGNORE__

#include "Common.h"
#include "Bullet.h"
#include "Vec3.h"

/* --- CollisionShape ----------------------------------------------------------
 *
 *   This was split out of RigidBody to support Triggers of different shapes.
 *   However, since Triggers are currently AABB tests only this isn't really
 *   used by Trigger. But that may change in the future and splitting up shape
 *   and rigidbodies is a good idea anyway.
 *
 * -------------------------------------------------------------------------- */

typedef uint8 CollisionShapeType;
const CollisionShapeType CollisionShapeType_Null     = 0;
const CollisionShapeType CollisionShapeType_Box      = 1;
const CollisionShapeType CollisionShapeType_Sphere   = 2;
const CollisionShapeType CollisionShapeType_Hull     = 3;
const CollisionShapeType CollisionShapeType_Compound = 4;

const CollisionGroup CollisionGroup_Null    = 0 << 0;
const CollisionGroup CollisionGroup_Default = 1 << 0;
const CollisionGroup CollisionGroup_Trigger = 1 << 1;

const CollisionMask CollisionMask_Null       = 0 << 0;
const CollisionMask CollisionMask_All        = ~CollisionGroup_Null;
const CollisionMask CollisionMask_NoTriggers = ~CollisionGroup_Trigger;

struct CollisionShape {
  int                iShape;
  float              scale;
  CollisionShapeType type;

  union {
    struct {
      btCollisionShape* handle;
    } base;

    struct {
      btBoxShape* handle;
      Vec3f       halfExtents;
    } box;

    struct {
      btSphereShape* handle;
      float          radius;
    } sphere;

    struct {
      btUniformScalingShape* handle;
      btConvexHullShape*     hullHandle;
      /* TODO : Ref-count the mesh. */
      Mesh*                  mesh;
    } hull;

    struct {
      btCompoundShape*  handle;
      /* TODO : Use for ref-counting/freeing. */
      //ArrayList(int,    subShapes);
    } compound;
  };
};

PHX_API CollisionShape*  _cppCollisionShape_Create                (CollisionShape);
PHX_API void             _cppCollisionShape_Free                  (CollisionShape*);

PHX_API CollisionShape*  _cppCollisionShape_CreateBox             (Vec3f* halfExtents);
PHX_API CollisionShape*  _cppCollisionShape_CreateBoxFromMesh     (Mesh*);
PHX_API CollisionShape*  _cppCollisionShape_CreateSphere          (float);
PHX_API CollisionShape*  _cppCollisionShape_CreateSphereFromMesh  (Mesh*);
PHX_API CollisionShape*  _cppCollisionShape_CreateHullFromMesh    (Mesh*);
PHX_API CollisionShape*  _cppCollisionShape_GetCached             (int);

#endif
