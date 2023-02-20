#ifndef PHX_Physics
#define PHX_Physics

#include "Common.h"
#include "ArrayList.h"
#include "Vec3.h"

/* --- Physics -----------------------------------------------------------------
 *
 *   Ray/shape casts/overlaps will return RigidBodys but not Triggers.
 *
 *     Physics_Free             : Removes RigidBodys but does not free them.
 *     Physics_AddRigidBody     : Automatically adds all attached Triggers.
 *                                Automatically adds all attached children and
 *                                their Triggers.
 *     Physics_RemoveRigidBody  : Automatically removes all attached Triggers.
 *                                Automatically removes all attached children
 *                                and their Triggers.
 *     Physics_GetNextCollision : Will include results for both child and parent
 *                                RigidBodys that are colliding. Will not
 *                                include Triggers.
 *     Physics_SphereCast       : Results are unsorted and will include child
 *                                objects.
 *     Physics_BoxCast          : Results are unsorted and will include child
 *                                objects.
 *
 * -------------------------------------------------------------------------- */

struct Collision {
  int        index;
  int        count;
  RigidBody* body0;
  RigidBody* body1;
};

struct RayCastResult {
  RigidBody* body;
  Vec3f      norm;
  Vec3f      pos;
  float      t;
};

struct ShapeCastResult {
  ArrayList(RigidBody*, hits);
};

PHX_API Physics*  _cppPhysics_Create                  ();
PHX_API void      _cppPhysics_Free                    (Physics*);

PHX_API void      _cppPhysics_AddRigidBody            (Physics*, RigidBody*);
PHX_API void      _cppPhysics_RemoveRigidBody         (Physics*, RigidBody*);
PHX_API void      _cppPhysics_AddTrigger              (Physics*, Trigger*);
PHX_API void      _cppPhysics_RemoveTrigger           (Physics*, Trigger*);

PHX_API bool      _cppPhysics_GetNextCollision        (Physics*, Collision*);
PHX_API void      _cppPhysics_Update                  (Physics*, float dt);

PHX_API void      _cppPhysics_RayCast                 (Physics*, Ray*, RayCastResult*);
PHX_API void      _cppPhysics_SphereCast              (Physics*, Sphere*, ShapeCastResult*);
PHX_API void      _cppPhysics_BoxCast                 (Physics*, Vec3f* pos, Quat* rot, Vec3f* halfExtents, ShapeCastResult*);
PHX_API bool      _cppPhysics_SphereOverlap           (Physics*, Sphere*);
PHX_API bool      _cppPhysics_BoxOverlap              (Physics*, Vec3f* pos, Quat* rot, Vec3f* halfExtents);

PHX_API void      _cppPhysics_PrintProfiling          (Physics*);
PHX_API void      _cppPhysics_DrawBoundingBoxesLocal  (Physics*);
PHX_API void      _cppPhysics_DrawBoundingBoxesWorld  (Physics*);
PHX_API void      _cppPhysics_DrawTriggers            (Physics*);
PHX_API void      _cppPhysics_DrawWireframes          (Physics*);

PRIVATE void  _cppPhysics_FlushCachedRigidBodyData  (Physics*, RigidBody*);

#endif
