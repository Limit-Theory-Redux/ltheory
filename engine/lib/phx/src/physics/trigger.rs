use crate::math::{Box3, Vec3};
use crate::physics::*;
use rapier3d::prelude as rp;
use rapier3d::prelude::nalgebra as na;
use std::cell::RefCell;
use std::mem::replace;
use std::rc::Rc;

enum State {
    // Uninitialized.
    None,
    // Removed from physics.
    Removed {
        collider: rp::Collider,
    },
    // Added to physics.
    Added {
        collider_handle: rp::ColliderHandle,
        world: PhysicsWorldHandle,
    },
    // Added to physics, and attached to another rigid body.
    AttachedToParent {
        parent: *mut RigidBody, // Raw pointer to stable memory address of parent (as it's in a Box).
        collider_handle: rp::ColliderHandle,
        world: PhysicsWorldHandle,
    },
}

pub struct Trigger {
    state: State,
    collision_group: rp::InteractionGroups,
}

impl Trigger {
    pub(crate) fn add_to_world(
        &mut self,
        world: &Rc<RefCell<PhysicsWorld>>,
    ) -> Option<rp::ColliderHandle> {
        // It only makes sense to add to the world if we're removed.
        if let State::Removed { collider } = replace(&mut self.state, State::None) {
            let w = &mut *world.borrow_mut();
            let collider_handle = w.colliders.insert(collider);
            self.state = State::Added {
                collider_handle,
                world: PhysicsWorldHandle::from_rc(world),
            };
            Some(collider_handle)
        } else {
            None
        }
    }

    pub(crate) fn remove_from_world(&mut self) -> Option<rp::ColliderHandle> {
        if let State::Added {
            collider_handle,
            world,
        } = replace(&mut self.state, State::None)
        {
            let w = world.upgrade();
            let w = &mut *w.borrow_mut();
            let collider = w
                .colliders
                .remove(
                    collider_handle,
                    &mut w.island_manager,
                    &mut w.rigid_bodies,
                    false,
                )
                .unwrap();
            self.state = State::Removed { collider };
            Some(collider_handle)
        } else {
            None
        }
    }

    /// Executes a function f with a reference to the collider associated with this object.
    pub(crate) fn with_collider<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&rp::Collider) -> R,
    {
        match &self.state {
            State::None => panic!("Uninitialized Trigger."),
            State::Removed { collider, .. } => f(collider),
            State::Added {
                collider_handle,
                world,
                ..
            } => f(world.upgrade().borrow().get_collider(*collider_handle)),
            State::AttachedToParent {
                collider_handle,
                world,
                ..
            } => f(world.upgrade().borrow().get_collider(*collider_handle)),
        }
    }

    /// Executes a function f with a mutable reference to the collider associated with this object.
    fn with_collider_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut rp::Collider) -> R,
    {
        match &mut self.state {
            State::None => panic!("Uninitialized Trigger."),
            State::Removed { collider, .. } => f(collider),
            State::Added {
                collider_handle,
                world,
                ..
            } => f(world
                .upgrade()
                .borrow_mut()
                .get_collider_mut(*collider_handle)),
            State::AttachedToParent {
                collider_handle,
                world,
                ..
            } => f(world
                .upgrade()
                .borrow_mut()
                .get_collider_mut(*collider_handle)),
        }
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
            state: State::Removed { collider },
            collision_group: rp::InteractionGroups::default(),
        }
    }

    fn attach(&mut self, parent: &mut RigidBody, offset: &Vec3) {
        self.state = match replace(&mut self.state, State::None) {
            State::AttachedToParent { .. } => panic!("Trigger is already attached to an object."),
            State::None | State::Removed { .. } => panic!("Trigger is not added to the world."),
            State::Added {
                collider_handle,
                world,
            } => {
                let w = world.upgrade();
                let w = &mut *w.borrow_mut();

                // Update the parent link.
                let parent_handle = parent
                    .get_rigid_body_handle()
                    .expect("The parent needs to be added to the world");
                w.colliders
                    .set_parent(collider_handle, Some(parent_handle), &mut w.rigid_bodies);

                // Set the offset correctly. If the parent is itself a child,
                // then we need to append to its relative transform.
                let translation = rp::Isometry::translation(offset.x, offset.y, offset.z);
                let transform = if parent.is_child() {
                    parent.with_collider(|c| c.position_wrt_parent().unwrap() * translation)
                } else {
                    translation
                };
                w.get_collider_mut(collider_handle)
                    .set_position_wrt_parent(transform);

                State::AttachedToParent {
                    parent: parent as *mut RigidBody,
                    collider_handle,
                    world,
                }
            }
        }
    }

    fn detach(&mut self, parent: &mut RigidBody) {
        self.state = match replace(&mut self.state, State::None) {
            State::AttachedToParent {
                parent: current_parent,
                collider_handle,
                world,
            } => {
                // TODO: Remove this check and remove the parent parameter completely.
                if parent as *mut RigidBody != current_parent {
                    panic!("Trigger is attached to a different object.");
                }

                let w = world.upgrade();
                let w = &mut *w.borrow_mut();

                // Update the parent link.
                w.colliders
                    .set_parent(collider_handle, None, &mut w.rigid_bodies);

                State::Added {
                    collider_handle,
                    world,
                }
            }
            // TODO: Maybe log here instead of panic?
            _ => panic!("Trigger is not attached to an object."),
        }
    }

    #[bind(out_param = true)]
    fn get_bounding_box(&self) -> Box3 {
        let aabb = self.with_collider(|c| c.compute_aabb());
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
        self.with_collider_mut(|c| c.set_collision_groups(collision_group));
    }

    #[bind(name = "SetPos")]
    fn set_position(&mut self, pos: &mut Vec3) {
        self.with_collider_mut(|c| c.set_translation(pos.to_na()));
    }

    #[bind(name = "SetPosLocal")]
    fn set_position_local(&mut self, pos: &mut Vec3) {
        if let State::AttachedToParent {
            parent,
            collider_handle,
            world,
        } = &self.state
        {
            let w = world.upgrade();
            let w = &mut *w.borrow_mut();
            let parent = unsafe { &mut **parent };

            // Compute the new local transformation by taking the existing
            // rigid body hierarchy into account. If the parent is itself
            // a child, then we need to append to its relative transform.
            let translation = rp::Isometry::translation(pos.x, pos.y, pos.z);
            let transform = if parent.is_child() {
                parent.with_collider(|c| c.position_wrt_parent().unwrap() * translation)
            } else {
                translation
            };
            w.get_collider_mut(*collider_handle)
                .set_position_wrt_parent(transform);
        }
    }

    fn get_parent(&mut self) -> Option<&mut RigidBody> {
        match &self.state {
            State::AttachedToParent { parent, .. } => unsafe { Some(&mut **parent) },
            _ => None,
        }
    }
}
