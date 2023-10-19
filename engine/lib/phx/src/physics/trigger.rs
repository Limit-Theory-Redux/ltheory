use crate::math::{Box3, Vec3};
use crate::physics::*;
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
pub(crate) enum TriggerState {
    Removed { collider: rp::Collider },
    Added { collider_handle: rp::ColliderHandle },
}

pub struct Trigger {
    state: TriggerState,
}

#[luajit_ffi_gen::luajit_ffi(managed = true)]
impl Trigger {
    fn create_box(halfExtents: &Vec3) -> Trigger {
        let collider = rp::ColliderBuilder::cuboid(halfExtents.x, halfExtents.y, halfExtents.z)
            .sensor(true)
            .build();
        Trigger {
            state: TriggerState::Removed { collider: collider },
        }
    }

    /// When attached to a RigidBody Triggers will have 1 frame of latency in
    /// their position. The transform of the RigidBody is copied to the Trigger at
    /// the beginning of each Physics_Update. This will include manual
    /// RigidBody_SetPos, but will not not include the pending kinematics update.
    fn attach(&mut self, rb: &mut RigidBody, offset: &Vec3) {}

    fn detach(&mut self, rb: &mut RigidBody) {}

    #[bind(out_param = true)]
    fn get_bounding_box(&self) -> Box3 {
        Box3::default()
    }

    fn get_contents_count(&self) -> i32 {
        0
    }

    fn get_contents(&self, i: i32) -> Option<&mut RigidBody> {
        None
    }

    fn set_collision_mask(&mut self, i: i32) {}

    fn set_pos(&mut self, pos: &mut Vec3) {}

    fn set_pos_local(&mut self, pos: &mut Vec3) {}

    fn get_parent(&mut self) -> Option<&RigidBody> {
        None
    }

    fn update(&mut self) {}
}

// #[no_mangle]
// pub extern "C" fn Trigger_Attach(this: &mut Trigger, rb: &mut RigidBody, offset: &mut Vec3) {}

// #[no_mangle]
// pub extern "C" fn Trigger_Detach(this: &mut Trigger, rb: &mut RigidBody) {}

// #[no_mangle]
// pub extern "C" fn Trigger_GetBoundingBox(this: &mut Trigger, out: &mut Box3) {}

// #[no_mangle]
// pub extern "C" fn Trigger_GetContentsCount(this: &mut Trigger) -> i32 {
//     0
// }

// /// Will only include the parent object when a compound is within the trigger.
// #[no_mangle]
// pub extern "C" fn Trigger_GetContents(this: &mut Trigger, i: i32) -> Option<&mut RigidBody> {
//     None
// }

// #[no_mangle]
// pub extern "C" fn Trigger_SetCollisionMask(this: &mut Trigger, i: i32) {}

// #[no_mangle]
// pub extern "C" fn Trigger_SetPos(this: &mut Trigger, pos: &mut Vec3) {}

// #[no_mangle]
// pub extern "C" fn Trigger_SetPosLocal(this: &mut Trigger, pos: &mut Vec3) {}

// pub fn Trigger_GetParent(this: &mut Trigger) -> Option<&RigidBody> {
//     None
// }

// pub fn Trigger_Update(this: &mut Trigger) {}
