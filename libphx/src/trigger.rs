use crate::math::{Box3, Vec3};
use crate::physics::*;
use crate::rigid_body::*;
use rapier3d::prelude as rp;
use rapier3d::prelude::nalgebra as na;

/*
struct Trigger {
  PhysicsType  type;
  GhostObject* handle;
  int          iShape;
  int          collisionGroup;
  int          collisionMask;

  RigidBody*   parent;
  Trigger*     next;
  btTransform  transformLocal;

  Physics*     physics;
};
 */
pub struct Trigger {}

#[no_mangle]
pub extern "C" fn Trigger_CreateBox(halfExtents: &mut Vec3) -> Box<Trigger> {
    Box::new(Trigger {})
}

#[no_mangle]
pub extern "C" fn Trigger_Free(_: Box<Trigger>) {}

/// When attached to a RigidBody Triggers will have 1 frame of latency in
/// their position. The transform of the RigidBody is copied to the Trigger at
/// the beginning of each Physics_Update. This will include manual
/// RigidBody_SetPos, but will not not include the pending kinematics update.
#[no_mangle]
pub extern "C" fn Trigger_Attach(this: &mut Trigger, rb: &mut RigidBody, offset: &mut Vec3) {}

#[no_mangle]
pub extern "C" fn Trigger_Detach(this: &mut Trigger, rb: &mut RigidBody) {}

#[no_mangle]
pub extern "C" fn Trigger_GetBoundingBox(this: &mut Trigger, out: &mut Box3) {}

#[no_mangle]
pub extern "C" fn Trigger_GetContentsCount(this: &mut Trigger) -> i32 {
    0
}

/// Will only include the parent object when a compound is within the trigger.
#[no_mangle]
pub extern "C" fn Trigger_GetContents(this: &mut Trigger, i: i32) -> Option<&mut RigidBody> {
    None
}

#[no_mangle]
pub extern "C" fn Trigger_SetCollisionMask(this: &mut Trigger, i: i32) {}

#[no_mangle]
pub extern "C" fn Trigger_SetPos(this: &mut Trigger, pos: &mut Vec3) {}

#[no_mangle]
pub extern "C" fn Trigger_SetPosLocal(this: &mut Trigger, pos: &mut Vec3) {}

pub fn Trigger_GetParent(this: &mut Trigger) -> Option<&RigidBody> {
    None
}

pub fn Trigger_Update(this: &mut Trigger) {}
