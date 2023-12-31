use crate::math::{Box3, Vec3};
use crate::physics::*;
use crate::rf::Rf;
use rapier3d::prelude as rp;
use rapier3d::prelude::nalgebra as na;
use std::ptr::NonNull;

pub struct Trigger {
    collider: ColliderWrapper,
    // Raw pointer to stable memory address of parent (as it's in a Box).
    parent: Option<NonNull<RigidBody>>,
    collision_group: rp::InteractionGroups,
}

impl Trigger {
    pub(crate) fn add_to_world(&mut self, world: Rf<PhysicsWorld>) {
        if self.collider.is_added() {
            return;
        }

        self.collider.set_added(|collider| {
            let handle = world.as_mut().colliders.insert(collider);
            (handle, world)
        });
    }

    pub(crate) fn remove_from_world(&mut self) {
        if self.collider.is_removed() {
            return;
        }

        self.collider.set_removed(|handle, world| {
            let w = &mut *world.as_mut();
            w.colliders
                .remove(handle, &mut w.island_manager, &mut w.rigid_bodies, false)
                .unwrap()
        });
    }

    pub fn is_attached(&self) -> bool {
        self.parent.is_some()
    }
}

#[luajit_ffi_gen::luajit_ffi(managed = true)]
impl Trigger {
    fn create_box(half_extents: &Vec3) -> Trigger {
        let collider = rp::ColliderBuilder::cuboid(half_extents.x, half_extents.y, half_extents.z)
            .sensor(true)
            .density(0.0)
            .build();
        Trigger {
            collider: ColliderWrapper::Removed(collider),
            collision_group: rp::InteractionGroups::default(),
            parent: None,
        }
    }

    fn attach(&mut self, parent: &mut RigidBody, offset: &Vec3) {
        if self.is_attached() {
            panic!("Trigger is already attached to an object.");
        }

        if self.collider.is_removed() {
            panic!("Trigger is not added to the world.");
        }

        let (collider_handle, world) = self.collider.added_as_ref().unwrap();

        {
            let w = &mut *world.as_mut();

            // Update the parent link.
            let parent_handle = parent
                .get_rigid_body_handle()
                .expect("The parent needs to be added to the world");
            w.colliders
                .set_parent(*collider_handle, Some(parent_handle), &mut w.rigid_bodies);
        }

        // Set the offset correctly. If the parent is itself a child,
        // then we need to append to its relative transform.
        let translation = rp::Isometry::translation(offset.x, offset.y, offset.z);
        let transform: na::Isometry<f32, na::Unit<na::Quaternion<f32>>, 3> = if parent.is_child() {
            parent.get_collider_ref().position_wrt_parent().unwrap() * translation
        } else {
            translation
        };
        world
            .as_mut()
            .get_mut(*collider_handle)
            .set_position_wrt_parent(transform);

        self.parent = Some(NonNull::new(parent as *mut _).expect("parent cannot be null"));
    }

    fn detach(&mut self, _parent: &mut RigidBody) {
        if !self.is_attached() {
            // TODO: Maybe log here instead of panic?
            panic!("Trigger is not attached to an object.");
        }

        if self.collider.is_removed() {
            panic!("Trigger is not added to the world.");
        }

        let (collider_handle, world) = self.collider.added_as_ref().unwrap();
        let w = &mut *world.as_mut();

        // Update the parent link.
        w.colliders
            .set_parent(*collider_handle, None, &mut w.rigid_bodies);
    }

    #[bind(out_param = true)]
    fn get_bounding_box(&self) -> Box3 {
        let aabb = self.collider.as_ref().compute_aabb();
        Box3::new(
            Vec3::from_na_point(&aabb.mins),
            Vec3::from_na_point(&aabb.maxs),
        )
    }

    fn get_contents_count(&self) -> i32 {
        0
    }

    /// Will only include the parent object when a compound is within the trigger.
    fn get_contents(&self, _i: i32) -> Option<&mut RigidBody> {
        None
    }

    fn set_collision_mask(&mut self, mask: u32) {
        self.collision_group.filter = mask.into();
        let collision_group = self.collision_group;
        self.collider.as_mut().set_collision_groups(collision_group);
    }

    #[bind(name = "SetPos")]
    fn set_position(&mut self, pos: &mut Vec3) {
        if self.is_attached() {
            panic!("Not allowed when attached to a RigidBody.");
        }

        self.collider.as_mut().set_translation(pos.to_na());
    }

    #[bind(name = "SetPosLocal")]
    fn set_position_local(&mut self, pos: &mut Vec3) {
        if self.is_attached() {
            panic!("Only allowed when attached to a RigidBody.");
        }

        let parent = unsafe { self.parent.as_mut().unwrap().as_mut() };

        // Compute the new local transformation by taking the existing
        // rigid body hierarchy into account. If the parent is itself
        // a child, then we need to append to its relative transform.
        let translation = rp::Isometry::translation(pos.x, pos.y, pos.z);
        let transform = if parent.is_child() {
            parent.get_collider_ref().position_wrt_parent().unwrap() * translation
        } else {
            translation
        };
        self.collider.as_mut().set_position_wrt_parent(transform);
    }

    fn get_parent(&mut self) -> Option<&mut RigidBody> {
        self.parent.as_mut().map(|ptr| unsafe { ptr.as_mut() })
    }
}
