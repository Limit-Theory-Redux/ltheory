/*

PHX_API Trigger*    _cppTrigger_CreateBox         (Vec3f* halfExtents);
PHX_API void        _cppTrigger_Free              (Trigger*);

PHX_API void        _cppTrigger_Attach            (Trigger*, RigidBody*, Vec3f*);
PHX_API void        _cppTrigger_Detach            (Trigger*, RigidBody*);

PHX_API void        _cppTrigger_GetBoundingBox    (Trigger*, Box3f*);
PHX_API int         _cppTrigger_GetContentsCount  (Trigger*);
PHX_API RigidBody*  _cppTrigger_GetContents       (Trigger*, int);
PHX_API void        _cppTrigger_SetCollisionMask  (Trigger*, int);

PHX_API void        _cppTrigger_SetPos            (Trigger*, Vec3f*);
PHX_API void        _cppTrigger_SetPosLocal       (Trigger*, Vec3f*);

 */

use crate::Math::Vec3;

extern "C" {
    pub type Trigger;
    pub type RigidBody;
    pub type Box3f;
    fn _cppTrigger_CreateBox(halfExtents: *mut Vec3) -> *mut Trigger;
    fn _cppTrigger_Free(this: *mut Trigger);
    fn _cppTrigger_Attach(this: *mut Trigger, rb: *mut RigidBody, offset: *mut Vec3);
    fn _cppTrigger_Detach(this: *mut Trigger, rb: *mut RigidBody);
    fn _cppTrigger_GetBoundingBox(this: *mut Trigger, out: *mut Box3f);
    fn _cppTrigger_GetContentsCount(this: *mut Trigger) -> i32;
    fn _cppTrigger_GetContents(this: *mut Trigger, i: i32) -> *mut RigidBody;
    fn _cppTrigger_SetCollisionMask(this: *mut Trigger, i: i32);
    fn _cppTrigger_SetPos(this: *mut Trigger, pos: *mut Vec3);
    fn _cppTrigger_SetPosLocal(this: *mut Trigger, pos: *mut Vec3);
}

#[no_mangle]
pub unsafe extern "C" fn Trigger_CreateBox(halfExtents: *mut Vec3) -> *mut Trigger {
    _cppTrigger_CreateBox(halfExtents)
}

#[no_mangle]
pub unsafe extern "C" fn Trigger_Free(this: *mut Trigger) {
    _cppTrigger_Free(this)
}

#[no_mangle]
pub unsafe extern "C" fn Trigger_Attach(this: *mut Trigger, rb: *mut RigidBody, offset: *mut Vec3) {
    _cppTrigger_Attach(this, rb, offset)
}

#[no_mangle]
pub unsafe extern "C" fn Trigger_Detach(this: *mut Trigger, rb: *mut RigidBody) {
    _cppTrigger_Detach(this, rb)
}

#[no_mangle]
pub unsafe extern "C" fn Trigger_GetBoundingBox(this: *mut Trigger, out: *mut Box3f) {
    _cppTrigger_GetBoundingBox(this, out)
}

#[no_mangle]
pub unsafe extern "C" fn Trigger_GetContentsCount(this: *mut Trigger) -> i32 {
    _cppTrigger_GetContentsCount(this)
}

#[no_mangle]
pub unsafe extern "C" fn Trigger_GetContents(this: *mut Trigger, i: i32) -> *mut RigidBody {
    _cppTrigger_GetContents(this, i)
}

#[no_mangle]
pub unsafe extern "C" fn Trigger_SetCollisionMask(this: *mut Trigger, i: i32) {
    _cppTrigger_SetCollisionMask(this, i)
}

#[no_mangle]
pub unsafe extern "C" fn Trigger_SetPos(this: *mut Trigger, pos: *mut Vec3) {
    _cppTrigger_SetPos(this, pos)
}

#[no_mangle]
pub unsafe extern "C" fn Trigger_SetPosLocal(this: *mut Trigger, pos: *mut Vec3) {
    _cppTrigger_SetPosLocal(this, pos)
}
