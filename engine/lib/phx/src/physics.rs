/*

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

 */

use crate::math::Vec3;
use crate::ray::*;

extern "C" {
    pub type Physics;
    pub type RigidBody;
    pub type Trigger;
    pub type Collision;
    pub type Sphere;
    pub type RayCastResult;
    pub type ShapeCastResult;
    pub type Quat;
    fn _cppPhysics_Create() -> *mut Physics;
    fn _cppPhysics_Free(_: *mut Physics);
    fn _cppPhysics_AddRigidBody(_: *mut Physics, _: *mut RigidBody);
    fn _cppPhysics_RemoveRigidBody(_: *mut Physics, _: *mut RigidBody);
    fn _cppPhysics_AddTrigger(_: *mut Physics, _: *mut Trigger);
    fn _cppPhysics_RemoveTrigger(_: *mut Physics, _: *mut Trigger);
    fn _cppPhysics_GetNextCollision(_: *mut Physics, _: *mut Collision) -> bool;
    fn _cppPhysics_Update(_: *mut Physics, dt: f32);
    fn _cppPhysics_RayCast(_: *mut Physics, _: *mut Ray, _: *mut RayCastResult);
    fn _cppPhysics_SphereCast(_: *mut Physics, _: *mut Sphere, _: *mut ShapeCastResult);
    fn _cppPhysics_BoxCast(
        _: *mut Physics,
        pos: *mut Vec3,
        rot: *mut Quat,
        halfExtents: *mut Vec3,
        _: *mut ShapeCastResult,
    );
    fn _cppPhysics_SphereOverlap(_: *mut Physics, _: *mut Sphere) -> bool;
    fn _cppPhysics_BoxOverlap(
        _: *mut Physics,
        pos: *mut Vec3,
        rot: *mut Quat,
        halfExtents: *mut Vec3,
    ) -> bool;
    fn _cppPhysics_PrintProfiling(_: *mut Physics);
    fn _cppPhysics_DrawBoundingBoxesLocal(_: *mut Physics);
    fn _cppPhysics_DrawBoundingBoxesWorld(_: *mut Physics);
    fn _cppPhysics_DrawTriggers(_: *mut Physics);
    fn _cppPhysics_DrawWireframes(_: *mut Physics);
}

#[no_mangle]
pub unsafe extern "C" fn Physics_Create() -> *mut Physics {
    _cppPhysics_Create()
}

#[no_mangle]
pub unsafe extern "C" fn Physics_Free(this: &mut Physics) {
    _cppPhysics_Free(this)
}

#[no_mangle]
pub unsafe extern "C" fn Physics_AddRigidBody(this: &mut Physics, rb: *mut RigidBody) {
    _cppPhysics_AddRigidBody(this, rb)
}

#[no_mangle]
pub unsafe extern "C" fn Physics_RemoveRigidBody(this: &mut Physics, rb: *mut RigidBody) {
    _cppPhysics_RemoveRigidBody(this, rb)
}

#[no_mangle]
pub unsafe extern "C" fn Physics_AddTrigger(this: &mut Physics, t: *mut Trigger) {
    _cppPhysics_AddTrigger(this, t)
}

#[no_mangle]
pub unsafe extern "C" fn Physics_RemoveTrigger(this: &mut Physics, t: *mut Trigger) {
    _cppPhysics_RemoveTrigger(this, t)
}

#[no_mangle]
pub unsafe extern "C" fn Physics_GetNextCollision(this: &mut Physics, c: *mut Collision) -> bool {
    _cppPhysics_GetNextCollision(this, c)
}

#[no_mangle]
pub unsafe extern "C" fn Physics_Update(this: &mut Physics, dt: f32) {
    _cppPhysics_Update(this, dt)
}

#[no_mangle]
pub unsafe extern "C" fn Physics_RayCast(
    this: &mut Physics,
    ray: *mut Ray,
    result: *mut RayCastResult,
) {
    _cppPhysics_RayCast(this, ray, result)
}

#[no_mangle]
pub unsafe extern "C" fn Physics_SphereCast(
    this: &mut Physics,
    sphere: *mut Sphere,
    result: *mut ShapeCastResult,
) {
    _cppPhysics_SphereCast(this, sphere, result)
}

#[no_mangle]
pub unsafe extern "C" fn Physics_BoxCast(
    this: &mut Physics,
    pos: *mut Vec3,
    rot: *mut Quat,
    halfExtents: *mut Vec3,
    result: *mut ShapeCastResult,
) {
    _cppPhysics_BoxCast(this, pos, rot, halfExtents, result)
}

#[no_mangle]
pub unsafe extern "C" fn Physics_SphereOverlap(this: &mut Physics, sphere: *mut Sphere) -> bool {
    _cppPhysics_SphereOverlap(this, sphere)
}

#[no_mangle]
pub unsafe extern "C" fn Physics_BoxOverlap(
    this: &mut Physics,
    pos: *mut Vec3,
    rot: *mut Quat,
    halfExtents: *mut Vec3,
) -> bool {
    _cppPhysics_BoxOverlap(this, pos, rot, halfExtents)
}

#[no_mangle]
pub unsafe extern "C" fn Physics_PrintProfiling(this: &mut Physics) {
    _cppPhysics_PrintProfiling(this)
}

#[no_mangle]
pub unsafe extern "C" fn Physics_DrawBoundingBoxesLocal(this: &mut Physics) {
    _cppPhysics_DrawBoundingBoxesLocal(this)
}

#[no_mangle]
pub unsafe extern "C" fn Physics_DrawBoundingBoxesWorld(this: &mut Physics) {
    _cppPhysics_DrawBoundingBoxesWorld(this)
}

#[no_mangle]
pub unsafe extern "C" fn Physics_DrawTriggers(this: &mut Physics) {
    _cppPhysics_DrawTriggers(this)
}

#[no_mangle]
pub unsafe extern "C" fn Physics_DrawWireframes(this: &mut Physics) {
    _cppPhysics_DrawWireframes(this)
}
