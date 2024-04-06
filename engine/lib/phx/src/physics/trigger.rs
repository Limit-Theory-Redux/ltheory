use std::ptr::NonNull;

use rapier3d_f64::prelude as rp;
use rapier3d_f64::prelude::nalgebra as na;

use crate::math::{Box3, Vec3};
use crate::physics::*;
use crate::rf::Rf;

struct TriggerParent {
    rigid_body: NonNull<RigidBody>,
    translation: na::Isometry3<rp::Real>,
}

pub struct Trigger {
    collider: ColliderWrapper,
    parent: Option<TriggerParent>,
    collision_group: rp::InteractionGroups,
    contents_cache: Vec<*mut RigidBody>,
}

impl Trigger {
    pub(crate) fn add_to_world(&mut self, world: Rf<PhysicsWorld>) {
        if self.collider.is_added() {
            return;
        }

        self.collider.set_added(world, |collider, w| {
            let collider = w.colliders.insert(collider);

            // If we're attached to a rigid body, set the collider's parent.
            if let Some(parent) = &self.parent {
                let parent_handle = unsafe { parent.rigid_body.as_ref() }
                    .get_rigid_body_handle()
                    .expect("The parent needs to be added to the world");
                w.colliders
                    .set_parent(collider, Some(parent_handle), &mut w.rigid_bodies);
            }

            collider
        });

        self.refresh_collider_offset();
    }

    pub(crate) fn remove_from_world(&mut self) {
        if self.collider.is_removed() {
            return;
        }

        self.collider.set_removed(|handle, w| {
            w.colliders
                .remove(handle, &mut w.island_manager, &mut w.rigid_bodies, false)
                .unwrap()
        });
    }

    pub(crate) fn refresh_collider_offset(&mut self) {
        if let Some(parent) = &self.parent {
            let parent_rb = unsafe { parent.rigid_body.as_ref() };

            // Set the offset correctly. If the parent is itself a child,
            // then we need to append to its relative transform.
            let transform = if parent_rb.is_child() {
                // TODO: If the child's relative position gets changed, we need to update our own offset.
                parent_rb.get_parent_internal().unwrap().offset * parent.translation
            } else {
                parent.translation
            };

            self.collider.as_mut().set_position_wrt_parent(transform);
        }
    }

    pub fn is_attached(&self) -> bool {
        self.parent.is_some()
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl Trigger {
    pub fn create_box(half_extents: &Vec3) -> Trigger {
        let collider = rp::ColliderBuilder::cuboid(
            half_extents.x as rp::Real,
            half_extents.y as rp::Real,
            half_extents.z as rp::Real,
        )
        .sensor(true)
        .density(0.0)
        .build();
        Trigger {
            collider: ColliderWrapper::Removed(collider),
            collision_group: rp::InteractionGroups::all(),
            parent: None,
            contents_cache: vec![],
        }
    }

    pub fn attach(&mut self, parent: &mut RigidBody, offset: &Vec3) {
        if self.is_attached() {
            panic!("Trigger is already attached to an object.");
        }

        self.parent = Some(TriggerParent {
            rigid_body: NonNull::new(parent as *mut _).expect("parent cannot be null"),
            translation: rp::Isometry::translation(
                offset.x as rp::Real,
                offset.y as rp::Real,
                offset.z as rp::Real,
            ),
        });
        parent.add_trigger(self);

        // If we're already added to the world, remove and re-add with the new parent information.
        if self.collider.is_added() {
            let world = self.collider.added_as_ref().unwrap().1.clone();
            self.remove_from_world();
            self.add_to_world(world)
        }
    }

    pub fn detach(&mut self, parent: &mut RigidBody) {
        if !self.is_attached() {
            panic!("Trigger is not attached to an object.");
        }

        parent.remove_trigger(self);
        self.parent = None;

        // If we're already added to the world, remove and re-add with the new parent information.
        if self.collider.is_added() {
            let world = self.collider.added_as_ref().unwrap().1.clone();
            self.remove_from_world();
            self.add_to_world(world)
        }
    }

    #[bind(out_param = true)]
    pub fn get_bounding_box(&self) -> Box3 {
        let aabb = self.collider.as_ref().compute_aabb();
        Box3::new(
            Vec3::from_na_point(&aabb.mins),
            Vec3::from_na_point(&aabb.maxs),
        )
    }

    pub fn get_contents_count(&mut self) -> i32 {
        if self.collider.is_removed() {
            return 0;
        }

        // Update the contacts list.
        let (collider, world) = self.collider.added_as_ref().unwrap();
        let world = &*world.as_ref();

        self.contents_cache.clear();
        self.contents_cache
            .extend(
                world
                    .narrow_phase
                    .intersections_with(*collider)
                    .filter_map(|pair| {
                        let other_collider = if pair.0 == *collider { pair.1 } else { pair.0 };

                        RigidBody::linked_with_collider_mut(
                            world.colliders.get(other_collider).unwrap(),
                        )
                    }),
            );

        self.contents_cache.len() as i32
    }

    /// Will only include the parent object when a compound is within the trigger.
    pub fn get_contents(&self, i: i32) -> Option<&mut RigidBody> {
        self.contents_cache
            .get(i as usize)
            .map(|ptr| unsafe { &mut **ptr })
    }

    pub fn set_collision_mask(&mut self, mask: u32) {
        self.collision_group.filter = mask.into();
        let collision_group = self.collision_group;
        self.collider.as_mut().set_collision_groups(collision_group);
    }

    #[bind(name = "SetPos")]
    pub fn set_position(&mut self, pos: &Vec3) {
        if self.is_attached() {
            panic!("Not allowed when attached to a RigidBody.");
        }

        self.collider.as_mut().set_translation(pos.to_na());
    }

    #[bind(name = "SetPosLocal")]
    pub fn set_position_local(&mut self, pos: &Vec3) {
        if !self.is_attached() {
            panic!("Only allowed when attached to a RigidBody.");
        }

        let parent = unsafe { self.parent.as_mut().unwrap().rigid_body.as_mut() };

        // Compute the new local transformation by taking the existing
        // rigid body hierarchy into account. If the parent is itself
        // a child, then we need to append to its relative transform.
        let translation =
            rp::Isometry::translation(pos.x as rp::Real, pos.y as rp::Real, pos.z as rp::Real);
        let transform = if parent.is_child() {
            parent.get_collider_ref().position_wrt_parent().unwrap() * translation
        } else {
            translation
        };
        self.collider.as_mut().set_position_wrt_parent(transform);
    }

    #[bind(name = "GetPos", out_param = true)]
    pub fn get_position(&self) -> Vec3 {
        if self.is_attached() {
            panic!("Not allowed when attached to a RigidBody.");
        }

        Vec3::from_na(&self.collider.as_ref().position().translation.vector)
    }

    #[bind(name = "GetPosLocal", out_param = true)]
    pub fn get_position_local(&self) -> Vec3 {
        if let Some(parent) = &self.parent {
            Vec3::from_na(&parent.translation.translation.vector)
        } else {
            Vec3::ZERO
        }
    }

    pub fn get_parent(&mut self) -> Option<&mut RigidBody> {
        self.parent
            .as_mut()
            .map(|parent: &mut TriggerParent| unsafe { parent.rigid_body.as_mut() })
    }
}
