/*
PHX_API RigidBody*  _cppRigidBody_CreateBox                    ();
PHX_API RigidBody*  _cppRigidBody_CreateBoxFromMesh            (Mesh*);
PHX_API RigidBody*  _cppRigidBody_CreateSphere                 ();
PHX_API RigidBody*  _cppRigidBody_CreateSphereFromMesh         (Mesh*);
PHX_API RigidBody*  _cppRigidBody_CreateHullFromMesh           (Mesh*);
PHX_API void        _cppRigidBody_Free                         (RigidBody*);

PHX_API void        _cppRigidBody_ApplyForce                   (RigidBody*, Vec3f*);
PHX_API void        _cppRigidBody_ApplyTorque                  (RigidBody*, Vec3f*);

PHX_API void        _cppRigidBody_Attach                       (RigidBody*, RigidBody* other, Vec3f*, Quat*);
PHX_API void        _cppRigidBody_Detach                       (RigidBody*, RigidBody* other);

PHX_API void        _cppRigidBody_GetBoundingBox               (RigidBody*, Box3*);
PHX_API void        _cppRigidBody_GetBoundingBoxCompound       (RigidBody*, Box3*);
PHX_API void        _cppRigidBody_GetBoundingBoxLocal          (RigidBody*, Box3*);
PHX_API void        _cppRigidBody_GetBoundingBoxLocalCompound  (RigidBody*, Box3*);
PHX_API float       _cppRigidBody_GetBoundingRadius            (RigidBody*);
PHX_API float       _cppRigidBody_GetBoundingRadiusCompound    (RigidBody*);

PHX_API RigidBody*  _cppRigidBody_GetParentBody                (RigidBody*);
PHX_API float       _cppRigidBody_GetSpeed                     (RigidBody*);
PHX_API Matrix*     _cppRigidBody_GetToLocalMatrix             (RigidBody*);
PHX_API Matrix*     _cppRigidBody_GetToWorldMatrix             (RigidBody*);
PHX_API void        _cppRigidBody_GetVelocity                  (RigidBody*, Vec3f*);
PHX_API void        _cppRigidBody_GetVelocityA                 (RigidBody*, Vec3f*);

PHX_API void        _cppRigidBody_SetCollidable                (RigidBody*, bool);
PHX_API void        _cppRigidBody_SetCollisionGroup            (RigidBody*, int);
PHX_API void        _cppRigidBody_SetCollisionMask             (RigidBody*, int);
PHX_API void        _cppRigidBody_SetDrag                      (RigidBody*, float linear, float angular);
PHX_API void        _cppRigidBody_SetFriction                  (RigidBody*, float);
PHX_API void        _cppRigidBody_SetKinematic                 (RigidBody*, bool);
PHX_API void        _cppRigidBody_SetRestitution               (RigidBody*, float);
PHX_API void        _cppRigidBody_SetSleepThreshold            (RigidBody*, float linear, float angular);

PHX_API float       _cppRigidBody_GetMass                      (RigidBody*);
PHX_API void        _cppRigidBody_SetMass                      (RigidBody*, float);
PHX_API void        _cppRigidBody_GetPos                       (RigidBody*, Vec3f*);
PHX_API void        _cppRigidBody_GetPosLocal                  (RigidBody*, Vec3f*);
PHX_API void        _cppRigidBody_SetPos                       (RigidBody*, Vec3f*);
PHX_API void        _cppRigidBody_SetPosLocal                  (RigidBody*, Vec3f*);
PHX_API void        _cppRigidBody_GetRot                       (RigidBody*, Quat*);
PHX_API void        _cppRigidBody_GetRotLocal                  (RigidBody*, Quat*);
PHX_API void        _cppRigidBody_SetRot                       (RigidBody*, Quat*);
PHX_API void        _cppRigidBody_SetRotLocal                  (RigidBody*, Quat*);
PHX_API float       _cppRigidBody_GetScale                     (RigidBody*);
PHX_API void        _cppRigidBody_SetScale                     (RigidBody*, float);

*/

use crate::math::*;

extern "C" {
    pub type RigidBody;
    pub type _Mesh; // Opaque mesh type to squash the warnings, as mesh::Mesh is not FFI safe, but it doesn't matter.
    fn _cppRigidBody_CreateBox() -> *mut RigidBody;
    fn _cppRigidBody_CreateBoxFromMesh(mesh: *mut _Mesh) -> *mut RigidBody;
    fn _cppRigidBody_CreateSphere() -> *mut RigidBody;
    fn _cppRigidBody_CreateSphereFromMesh(mesh: *mut _Mesh) -> *mut RigidBody;
    fn _cppRigidBody_CreateHullFromMesh(mesh: *mut _Mesh) -> *mut RigidBody;
    fn _cppRigidBody_Free(this: &mut RigidBody);
    fn _cppRigidBody_ApplyForce(this: &mut RigidBody, force: *mut Vec3);
    fn _cppRigidBody_ApplyTorque(this: &mut RigidBody, torque: *mut Vec3);
    fn _cppRigidBody_Attach(
        this: &mut RigidBody,
        other: *mut RigidBody,
        offset: *mut Vec3,
        rot: *mut Quat,
    );
    fn _cppRigidBody_Detach(this: &mut RigidBody, other: *mut RigidBody);
    fn _cppRigidBody_GetBoundingBox(this: &mut RigidBody, out: *mut Box3);
    fn _cppRigidBody_GetBoundingBoxCompound(this: &mut RigidBody, out: *mut Box3);
    fn _cppRigidBody_GetBoundingBoxLocal(this: &mut RigidBody, out: *mut Box3);
    fn _cppRigidBody_GetBoundingBoxLocalCompound(this: &mut RigidBody, out: *mut Box3);
    fn _cppRigidBody_GetBoundingRadius(this: &mut RigidBody) -> f32;
    fn _cppRigidBody_GetBoundingRadiusCompound(this: &mut RigidBody) -> f32;
    fn _cppRigidBody_GetParentBody(this: &mut RigidBody) -> *mut RigidBody;
    fn _cppRigidBody_GetSpeed(this: &mut RigidBody) -> f32;
    fn _cppRigidBody_GetToLocalMatrix(this: &mut RigidBody) -> *mut Matrix;
    fn _cppRigidBody_GetToWorldMatrix(this: &mut RigidBody) -> *mut Matrix;
    fn _cppRigidBody_GetVelocity(this: &mut RigidBody, out: *mut Vec3);
    fn _cppRigidBody_GetVelocityA(this: &mut RigidBody, out: *mut Vec3);
    fn _cppRigidBody_SetCollidable(this: &mut RigidBody, collidable: bool);
    fn _cppRigidBody_SetCollisionGroup(this: &mut RigidBody, group: i32);
    fn _cppRigidBody_SetCollisionMask(this: &mut RigidBody, mask: i32);
    fn _cppRigidBody_SetDrag(this: &mut RigidBody, linear: f32, angular: f32);
    fn _cppRigidBody_SetFriction(this: &mut RigidBody, friction: f32);
    fn _cppRigidBody_SetKinematic(this: &mut RigidBody, kinematic: bool);
    fn _cppRigidBody_SetRestitution(this: &mut RigidBody, restitution: f32);
    fn _cppRigidBody_SetSleepThreshold(this: &mut RigidBody, linear: f32, angular: f32);
    fn _cppRigidBody_GetMass(this: &mut RigidBody) -> f32;
    fn _cppRigidBody_SetMass(this: &mut RigidBody, mass: f32);
    fn _cppRigidBody_GetPos(this: &mut RigidBody, out: *mut Vec3);
    fn _cppRigidBody_GetPosLocal(this: &mut RigidBody, out: *mut Vec3);
    fn _cppRigidBody_SetPos(this: &mut RigidBody, pos: *mut Vec3);
    fn _cppRigidBody_SetPosLocal(this: &mut RigidBody, pos: *mut Vec3);
    fn _cppRigidBody_GetRot(this: &mut RigidBody, out: *mut Quat);
    fn _cppRigidBody_GetRotLocal(this: &mut RigidBody, out: *mut Quat);
    fn _cppRigidBody_SetRot(this: &mut RigidBody, rot: *mut Quat);
    fn _cppRigidBody_SetRotLocal(this: &mut RigidBody, rot: *mut Quat);
    fn _cppRigidBody_GetScale(this: &mut RigidBody) -> f32;
    fn _cppRigidBody_SetScale(this: &mut RigidBody, scale: f32);
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_CreateBox() -> *mut RigidBody {
    _cppRigidBody_CreateBox()
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_CreateBoxFromMesh(mesh: *mut _Mesh) -> *mut RigidBody {
    _cppRigidBody_CreateBoxFromMesh(mesh)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_CreateSphere() -> *mut RigidBody {
    _cppRigidBody_CreateSphere()
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_CreateSphereFromMesh(mesh: *mut _Mesh) -> *mut RigidBody {
    _cppRigidBody_CreateSphereFromMesh(mesh)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_CreateHullFromMesh(mesh: *mut _Mesh) -> *mut RigidBody {
    _cppRigidBody_CreateHullFromMesh(mesh)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_Free(this: &mut RigidBody) {
    _cppRigidBody_Free(this)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_ApplyForce(this: &mut RigidBody, force: *mut Vec3) {
    _cppRigidBody_ApplyForce(this, force)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_ApplyTorque(this: &mut RigidBody, torque: *mut Vec3) {
    _cppRigidBody_ApplyTorque(this, torque)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_Attach(
    this: &mut RigidBody,
    other: *mut RigidBody,
    offset: *mut Vec3,
    rot: *mut Quat,
) {
    _cppRigidBody_Attach(this, other, offset, rot)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_Detach(this: &mut RigidBody, other: *mut RigidBody) {
    _cppRigidBody_Detach(this, other)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_GetBoundingBox(this: &mut RigidBody, out: *mut Box3) {
    _cppRigidBody_GetBoundingBox(this, out)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_GetBoundingBoxCompound(this: &mut RigidBody, out: *mut Box3) {
    _cppRigidBody_GetBoundingBoxCompound(this, out)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_GetBoundingBoxLocal(this: &mut RigidBody, out: *mut Box3) {
    _cppRigidBody_GetBoundingBoxLocal(this, out)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_GetBoundingBoxLocalCompound(
    this: &mut RigidBody,
    out: *mut Box3,
) {
    _cppRigidBody_GetBoundingBoxLocalCompound(this, out)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_GetBoundingRadius(this: &mut RigidBody) -> f32 {
    _cppRigidBody_GetBoundingRadius(this)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_GetBoundingRadiusCompound(this: &mut RigidBody) -> f32 {
    _cppRigidBody_GetBoundingRadiusCompound(this)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_GetParentBody(this: &mut RigidBody) -> *mut RigidBody {
    _cppRigidBody_GetParentBody(this)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_GetSpeed(this: &mut RigidBody) -> f32 {
    _cppRigidBody_GetSpeed(this)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_GetToLocalMatrix(this: &mut RigidBody) -> *mut Matrix {
    let result = _cppRigidBody_GetToLocalMatrix(this);
    Matrix_ITranspose(&mut *result);
    result
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_GetToWorldMatrix(this: &mut RigidBody) -> *mut Matrix {
    let result = _cppRigidBody_GetToWorldMatrix(this);
    Matrix_ITranspose(&mut *result);
    result
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_GetVelocity(this: &mut RigidBody, out: *mut Vec3) {
    _cppRigidBody_GetVelocity(this, out)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_GetVelocityA(this: &mut RigidBody, out: *mut Vec3) {
    _cppRigidBody_GetVelocityA(this, out)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_SetCollidable(this: &mut RigidBody, collidable: bool) {
    _cppRigidBody_SetCollidable(this, collidable)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_SetCollisionGroup(this: &mut RigidBody, group: i32) {
    _cppRigidBody_SetCollisionGroup(this, group)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_SetCollisionMask(this: &mut RigidBody, mask: i32) {
    _cppRigidBody_SetCollisionMask(this, mask)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_SetDrag(this: &mut RigidBody, linear: f32, angular: f32) {
    _cppRigidBody_SetDrag(this, linear, angular)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_SetFriction(this: &mut RigidBody, friction: f32) {
    _cppRigidBody_SetFriction(this, friction)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_SetKinematic(this: &mut RigidBody, kinematic: bool) {
    _cppRigidBody_SetKinematic(this, kinematic)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_SetRestitution(this: &mut RigidBody, restitution: f32) {
    _cppRigidBody_SetRestitution(this, restitution)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_SetSleepThreshold(
    this: &mut RigidBody,
    linear: f32,
    angular: f32,
) {
    _cppRigidBody_SetSleepThreshold(this, linear, angular)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_GetMass(this: &mut RigidBody) -> f32 {
    _cppRigidBody_GetMass(this)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_SetMass(this: &mut RigidBody, mass: f32) {
    _cppRigidBody_SetMass(this, mass)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_GetPos(this: &mut RigidBody, out: *mut Vec3) {
    _cppRigidBody_GetPos(this, out)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_GetPosLocal(this: &mut RigidBody, out: *mut Vec3) {
    _cppRigidBody_GetPosLocal(this, out)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_SetPos(this: &mut RigidBody, pos: *mut Vec3) {
    _cppRigidBody_SetPos(this, pos)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_SetPosLocal(this: &mut RigidBody, pos: *mut Vec3) {
    _cppRigidBody_SetPosLocal(this, pos)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_GetRot(this: &mut RigidBody, out: *mut Quat) {
    _cppRigidBody_GetRot(this, out)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_GetRotLocal(this: &mut RigidBody, out: *mut Quat) {
    _cppRigidBody_GetRotLocal(this, out)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_SetRot(this: &mut RigidBody, rot: *mut Quat) {
    _cppRigidBody_SetRot(this, rot)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_SetRotLocal(this: &mut RigidBody, rot: *mut Quat) {
    _cppRigidBody_SetRotLocal(this, rot)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_GetScale(this: &mut RigidBody) -> f32 {
    _cppRigidBody_GetScale(this)
}

#[no_mangle]
pub unsafe extern "C" fn RigidBody_SetScale(this: &mut RigidBody, scale: f32) {
    _cppRigidBody_SetScale(this, scale)
}
