use crate::common::*;
use crate::math::*;
use crate::physics::*;
use crate::render::*;
use rapier3d::prelude as rp;
use rapier3d::prelude::nalgebra as na;
use std::cell::Ref;
use std::cell::RefCell;
use std::mem::replace;
use std::rc::Rc;

pub enum PhysicsType {
    Null,
    RigidBody,
    Trigger,
}

#[derive(Clone)]
enum WorldState {
    // Uninitialized.
    None,
    // Removed from physics.
    Removed {
        rb: rp::RigidBody,
        collider: rp::Collider,
    },
    // Added to physics.
    Added {
        rb_handle: rp::RigidBodyHandle,
        collider_handle: rp::ColliderHandle,
        world: PhysicsWorldHandle,
        children: Vec<rp::ColliderHandle>,
    },
    // Added to physics, and attached to a compound shape.
    AttachedToCompound {
        parent: *mut RigidBody, // Raw pointer to stable memory address of parent (as it's in a Box).
        rb: rp::RigidBody, // Unused rapier RB, which we'd use again if the rigid body is detached.
        collider_handle: rp::ColliderHandle,
        world: PhysicsWorldHandle,
    },
}

/*
 * The following API functions are disabled for parent objects:
 * get_position_local, set_position_local, get_rotation_local, and
 * set_rotation_local.
 *
 * The following API functions are disabled for child objects:
 * apply_force, apply_torque, get_speed, get_velocity, get_angular_velocity,
 * set_position, and set_rotation.
 *
 * The following API functions only have an effect once the child is removed
 * from its parent: set_collidable, set_collision_group, set_collision_mask,
 * set_drag, set_friction, set_kinematic, set_restitution, and
 * set_sleep_threshold.
 *
 * The following API functions return information only about the current part
 * when the object is part of a compound: get_bounding_box_local,
 * get_bounding_box, and get_bounding_radius.
 *
 * The following API functions are only enabled for compound objects:
 * get_bounding_box_compound, get_bounding_box_local_compound,
 * and get_bounding_radius_compound.
 *
 * The local coordinate space of a child object is not scaled by the parent.
 * However, the position of the child will be multiplied by the parents scale.
 * Thus, the scale of the parent does not affect the size of the child and
 * local position is always 'relative to the parent'. A position of (1, 1, -1)
 * will always correspond to a point that will roughly coincide with the
 * right-top-front corner of the parents bounding box (assuming the vertices
 * of the mesh are contained in a cube that goes from (-1, -1, -1) to
 * (1, 1, 1)). When a parent is scaled the positions of children will be
 * multiplied in order to maintain the same relative position.
 */

// TODO: Implement Free semantics: Automatically frees all attached Triggers when called on a parent. Automatically frees all attached children and their Triggers when called on a parent. This function is O(M*N) for parents.
pub struct RigidBody {
    ty: PhysicsType,

    // Stores the rigid body / collider state machine.
    state: WorldState,

    // Fields to allow us to reconstruct the collision shape object.
    shape_type: CollisionShapeType,
    shape_scale: f32,

    collidable: bool,
    collision_group: rp::InteractionGroups,
    mass: f32,
}

// Functions to add and remove the rigid body from physics.
impl RigidBody {
    pub(crate) fn add_to_world(
        &mut self,
        world: &Rc<RefCell<PhysicsWorld>>,
    ) -> Option<(rp::ColliderHandle, rp::RigidBodyHandle)> {
        // It only makes sense to add to the world if we're removed.
        if let WorldState::Removed { rb, collider } = replace(&mut self.state, WorldState::None) {
            let w = &mut *world.borrow_mut();
            let rb_handle = w.rigid_bodies.insert(rb);
            let collider_handle =
                w.colliders
                    .insert_with_parent(collider, rb_handle, &mut w.rigid_bodies);
            self.state = WorldState::Added {
                rb_handle,
                collider_handle,
                children: vec![],
                world: PhysicsWorldHandle::from_rc(world),
            };
            Some((collider_handle, rb_handle))
        } else {
            None
        }
    }

    pub(crate) fn remove_from_world(
        &mut self,
        impulse_joint_set: &mut rp::ImpulseJointSet,
        multibody_joint_set: &mut rp::MultibodyJointSet,
    ) -> Option<(rp::ColliderHandle, rp::RigidBodyHandle)> {
        if let WorldState::Added {
            rb_handle,
            collider_handle,
            world,
            ..
        } = replace(&mut self.state, WorldState::None)
        {
            let world_rc = world.upgrade();
            let w = &mut *world_rc.borrow_mut();
            let collider = w
                .colliders
                .remove(
                    collider_handle,
                    &mut w.island_manager,
                    &mut w.rigid_bodies,
                    false,
                )
                .unwrap();
            let rigid_body = w
                .rigid_bodies
                .remove(
                    rb_handle,
                    &mut w.island_manager,
                    &mut w.colliders,
                    impulse_joint_set,
                    multibody_joint_set,
                    false,
                )
                .unwrap();
            self.state = WorldState::Removed {
                rb: rigid_body,
                collider,
            };
            Some((collider_handle, rb_handle))
        } else {
            None
        }
    }

    /// Executes a function f with a reference to the RigidBody associated with this object.
    fn with_rigid_body<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&rp::RigidBody) -> R,
    {
        match &self.state {
            WorldState::None => panic!("Uninitialized RigidBody."),
            WorldState::Removed { rb, .. } => f(rb),
            WorldState::Added {
                rb_handle, world, ..
            } => f(world
                .upgrade()
                .borrow()
                .get_rigid_body(*rb_handle)),
            WorldState::AttachedToCompound { .. } => panic!("Not supported on children."),
        }
    }

    /// Executes a function f with a mutable reference to the RigidBody associated with this object.
    fn with_rigid_body_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut rp::RigidBody) -> R,
    {
        match &mut self.state {
            WorldState::None => panic!("Uninitialized RigidBody."),
            WorldState::Removed { rb, .. } => f(rb),
            WorldState::Added {
                rb_handle, world, ..
            } => f(world
                .upgrade()
                .borrow_mut()
                .get_rigid_body_mut(*rb_handle)),
            WorldState::AttachedToCompound { .. } => panic!("Not supported on children."),
        }
    }

    /// Executes a function f with a reference to the collider associated with this object.
    fn with_collider<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&rp::Collider) -> R,
    {
        match &self.state {
            WorldState::None => panic!("Uninitialized RigidBody."),
            WorldState::Removed { collider, .. } => f(collider),
            WorldState::Added {
                collider_handle,
                world,
                ..
            } => f(world
                .upgrade()
                .borrow()
                .get_collider(*collider_handle)),
            WorldState::AttachedToCompound {
                collider_handle,
                world,
                ..
            } => f(world
                .upgrade()
                .borrow()
                .get_collider(*collider_handle)),
        }
    }

    /// Executes a function f with a mutable reference to the collider associated with this object.
    fn with_collider_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut rp::Collider) -> R,
    {
        match &mut self.state {
            WorldState::None => panic!("Uninitialized RigidBody."),
            WorldState::Removed { collider, .. } => f(collider),
            WorldState::Added {
                collider_handle,
                world,
                ..
            } => f(world
                .upgrade()
                .borrow_mut()
                .get_collider_mut(*collider_handle)),
            WorldState::AttachedToCompound {
                collider_handle,
                world,
                ..
            } => f(world
                .upgrade()
                .borrow_mut()
                .get_collider_mut(*collider_handle)),
        }
    }

    pub fn new(shape: CollisionShape) -> Box<RigidBody> {
        let rigidBody = rp::RigidBodyBuilder::dynamic().build();
        Box::new(RigidBody {
            ty: PhysicsType::RigidBody,
            state: WorldState::Removed {
                rb: rigidBody,
                collider: shape.collider,
            },
            shape_type: shape.shape,
            shape_scale: shape.scale,
            collidable: true,
            collision_group: rp::InteractionGroups::default(),
            mass: 1.0,
        })
    }

    // /// Is this rigid body part of a compound shape?
    // pub fn is_in_compound(&self) -> bool {
    //     self.is_root_in_compound() || self.is_child()
    // }

    // /// Is this rigid body a child of the root in a compound shape?
    // pub fn is_child(&self) -> bool {
    //     if let WorldState::AttachedToCompound { .. } = &self.state {
    //         true
    //     } else {
    //         false
    //     }
    // }

    // /// Is this rigid body part of a compound shape, and is also the root?
    // pub fn is_root_in_compound(&self) -> bool {
    //     // TODO: The collider is a compound shape
    //     false
    // }
    
    /// Returns the unscaled world matrix of this rigid body.
    fn get_world_matrix_unscaled(&self) -> Matrix {
        let global_transform = if let WorldState::AttachedToCompound { parent, .. } = &self.state {
            self.with_collider(|c| {
                unsafe { &**parent }.with_rigid_body(|parent_rb| {
                    let transform = c.position_wrt_parent().unwrap();
                    let parent_transform = parent_rb.position();
                    parent_transform * transform
                })
            })
        } else {
            self.with_rigid_body(|rb| rb.position().clone())
        };
        Matrix::from_rp(&global_transform)
    }
}

#[luajit_ffi_gen::luajit_ffi(managed = true)]
impl RigidBody {
    #[bind(name = "CreateBox")]
    pub fn new_box() -> Box<RigidBody> {
        Self::new(CollisionShape::new_box(&Vec3::ONE))
    }

    #[bind(name = "CreateBoxFromMesh")]
    pub fn new_box_from_mesh(mesh: &mut Mesh) -> Box<RigidBody> {
        Self::new(CollisionShape::new_box_from_mesh(mesh))
    }

    #[bind(name = "CreateSphere")]
    pub fn new_sphere() -> Box<RigidBody> {
        Self::new(CollisionShape::new_sphere(1.0))
    }

    #[bind(name = "CreateSphereFromMesh")]
    pub fn new_sphere_from_mesh(mesh: &mut Mesh) -> Box<RigidBody> {
        Self::new(CollisionShape::new_sphere_from_mesh(mesh))
    }

    #[bind(name = "CreateHullFromMesh")]
    pub fn new_hull_from_mesh(mesh: Box<Mesh>) -> Box<RigidBody> {
        Self::new(CollisionShape::new_hull_from_mesh(mesh))
    }

    /// Return a reference to the parent rigid body, that we can guarantee
    /// has a lifetime as long as self.
    #[bind(name = "GetParentBody")]
    pub fn get_parent(&self) -> Option<&mut RigidBody> {
        match &self.state {
            WorldState::AttachedToCompound { parent, .. } => {
                // SAFETY: This only works if the pointer to the RigidBody is
                // stable, i.e. it's never moved outside the Box<...>.
                // SAFETY: The parent must outlive this rigid body.
                Some(unsafe { &mut **parent })
            }
            _ => None,
        }
    }

    pub fn apply_force(&mut self, force: &Vec3) {
        self.with_rigid_body_mut(|rb| rb.add_force(force.to_na(), true))
    }

    pub fn apply_torque(&mut self, torque: &Vec3) {
        self.with_rigid_body_mut(|rb| rb.add_torque(torque.to_na(), true))
    }

    /// Adds another rigid body as a child of this rigid body. This means that
    /// the child's position will be controlled by `self`.
    ///
    /// Only a single level of attachment is supported. Child objects do not
    /// affect the mass or inertia of the parent. Position is relative to the
    /// unscaled parent. i.e. it will be multiplied by the current scale. This
    /// function is O(1). Warning: if one object is attached to another and a
    /// third object happens to be between them this may trap the third object.
    /// The same issue may occur when spawning one compound inside another.
    ///
    /// This function assumes that `self` is not already a child.
    pub fn attach(&mut self, child: &mut RigidBody, pos: &Vec3, rot: &Quat) {
        let parent_ptr = self as *mut _;
        match &self.state {
            WorldState::None => {}
            WorldState::AttachedToCompound { .. } => {
                panic!("Recursive attachment is not supported. Parent is already attached to something.");
            }
            WorldState::Removed { .. } => {
                panic!("Parent has been removed from physics.");
            }
            WorldState::Added {
                rb_handle: parent_handle,
                world,
                ..
            } => {
                child.state = match child.state.clone() {
                    WorldState::None => {
                        panic!("Child is not initialised");
                    }
                    WorldState::Added { .. } => {
                        panic!("Child has not been removed from physics.");
                    }
                    WorldState::AttachedToCompound { .. } => {
                        panic!("Child is already attached to a parent.");
                    }
                    WorldState::Removed { rb, mut collider } => {
                        let world_rc = world.upgrade();
                        let w = &mut *world_rc.borrow_mut();

                        // Multiple colliders in Rapier can be attached to a single
                        // rigid body, so there's no need to create a "compound shape"

                        // Add the collider to the parent rigid body.
                        let collider_handle = w.colliders.insert_with_parent(
                            collider,
                            *parent_handle,
                            &mut w.rigid_bodies,
                        );
                        
                        // Set the colliders relative position, scaled by the scale of the parent shape.
                        let scaled_pos = *pos * self.shape_scale;
                        w.get_collider_mut(collider_handle).set_position_wrt_parent(na::Isometry3::from_parts(
                            scaled_pos.to_na().into(),
                            rot.to_na(),
                        ));
                        WorldState::AttachedToCompound {
                            parent: parent_ptr,
                            rb,
                            collider_handle,
                            world: world.clone(),
                        }
                    }
                }
            }
        }
    }

    /// Removes a shape from this compound shape, and changes it back to a singular shape if
    /// no children are left.
    ///
    /// This function assumes that `self` is not already a child.
    pub fn detach(&mut self, child: &mut RigidBody) {
        // TODO: Introduce some kind of state transition function for &mut RigidBody that does something like
        /*
        child.transition_state(|state| match state {
        });
        */
        child.state = match replace(&mut child.state, WorldState::None) {
            WorldState::None => {
                panic!("Child is not initialised");
            }
            WorldState::Added { .. } | WorldState::Removed { .. } => {
                panic!("Child is not attached to parent.");
            }
            WorldState::AttachedToCompound { mut rb, parent, collider_handle, world } => {
                if parent != (self as *mut RigidBody) {
                    panic!("Child is not attached to parent.");
                }

                // Convert current transform to world coordinates.
                let parent_transform = self.with_rigid_body(|rb| rb.position().clone());
                
                // Get a mutable ref to the physics world.
                let world_rc = world.upgrade();
                let w = &mut *world_rc.borrow_mut();

                // Compute the combined transform.
                let child_transform = w.colliders.get(collider_handle).unwrap().position_wrt_parent().unwrap();
                let combined_transform = parent_transform * child_transform;

                // Detach from parent by removing from the collider set.
                let collider = w.colliders.remove(collider_handle, &mut w.island_manager, &mut w.rigid_bodies, true).unwrap();
                rb.set_position(combined_transform, true);
                WorldState::Removed {
                    rb,
                    collider,
                }
            }
        };
    }

    /// Calculates the bounding box, and assigns it to `out`.
    pub fn get_bounding_box(&self, out: &mut Box3) {
        let aabb = self.with_collider(|c| c.compute_aabb());
        out.lower = Vec3::from_na_point(&aabb.mins);
        out.upper = Vec3::from_na_point(&aabb.maxs);
    }

    /// Calculates the compoind bounding box, and assigns it to `out`.
    pub fn get_bounding_box_compound(&self, out: &mut Box3) {
        // TODO: Get the AABB of the compound shape i.e. the root of a compound tree.
        let aabb = self.with_collider(|c| c.compute_aabb());
        out.lower = Vec3::from_na_point(&aabb.mins);
        out.upper = Vec3::from_na_point(&aabb.maxs);
    }

    /// Calculates the local bounding box, and assigns it to `out`.
    pub fn get_bounding_box_local(&self, out: &mut Box3) {
        let aabb = self.with_collider(|c| c.shape().compute_local_aabb());
        out.lower = Vec3::from_na_point(&aabb.mins);
        out.upper = Vec3::from_na_point(&aabb.maxs);
    }

    /// Calculates the local compound bounding box, and assigns it to `out`.
    pub fn get_bounding_box_local_compound(&self, out: &mut Box3) {
        if let WorldState::Added { collider_handle, children, world, .. } = &self.state {
            let world_rc = world.upgrade();
            let mut w = world_rc.borrow_mut();

            // Get AABB of the main collider.
            let mut aabb = w.get_collider(*collider_handle).shape().compute_local_aabb();

            // Incorporate the AABBs of the compound shapes.
            for child_collider_handle in children.iter() {
                let collider = w.get_collider_mut(*child_collider_handle);
                let child_aabb = collider.shape().compute_aabb(collider.position_wrt_parent().unwrap());
                aabb.mins = aabb.mins.inf(&child_aabb.mins);
                aabb.maxs = aabb.maxs.sup(&child_aabb.maxs);
            }
            
            out.lower = Vec3::from_na_point(&aabb.mins);
            out.upper = Vec3::from_na_point(&aabb.maxs);
        }
    }

    pub fn get_bounding_radius(&self) -> f32 {
        self.with_collider(|c| c.shape().compute_local_bounding_sphere().radius)
    }

    pub fn get_bounding_radius_compound(&self) -> f32 {
        // TODO: Get the AABB of the compound shape i.e. the root of a compound tree.
        self.with_collider(|c| c.shape().compute_local_bounding_sphere().radius)
    }

    pub fn get_speed(&self) -> f32 {
        self.with_rigid_body(|rb| rb.linvel().norm())
    }

    /// Returns the local -> world matrix for this rigid body.
    pub fn get_to_world_matrix(&self) -> Matrix {
        self.get_world_matrix_unscaled() * Matrix::from_scale(Vec3::splat(self.get_scale()))
    }

    /// Returns the world -> local matrix for this rigid body.
    pub fn get_to_local_matrix(&self) -> Matrix {
        self.get_to_world_matrix().inverse()
    }

    #[bind(out_param = true)]
    pub fn get_velocity(&self) -> Vec3 {
        self.with_rigid_body(|rb| Vec3::from_na(rb.linvel()))
    }

    #[bind(name = "GetVelocityA", out_param = true)]
    pub fn get_angular_velocity(&self) -> Vec3 {
        self.with_rigid_body(|rb| Vec3::from_na(rb.angvel()))
    }

    /// When disabled, the object will pass through others without colliding
    /// and will not be returned from ray or shape casts.
    pub fn set_collidable(&mut self, collidable: bool) {
        self.collidable = collidable;
        let collision_group = if self.collidable {
            self.collision_group
        } else {
            rp::InteractionGroups::none()
        };
        self.with_collider_mut(|c| c.set_collision_groups(collision_group));
    }

    pub fn set_collision_group(&mut self, group: u32) {
        self.collision_group.memberships = group.into();
        let collision_group = if self.collidable {
            self.collision_group
        } else {
            rp::InteractionGroups::none()
        };
        self.with_collider_mut(|c| c.set_collision_groups(collision_group));
    }

    pub fn set_collision_mask(&mut self, mask: u32) {
        self.collision_group.filter = mask.into();
        let collision_group = if self.collidable {
            self.collision_group
        } else {
            rp::InteractionGroups::none()
        };
        self.with_collider_mut(|c| c.set_collision_groups(collision_group));
    }

    pub fn set_drag(&mut self, linear: f32, angular: f32) {
        self.with_rigid_body_mut(|rb| {
            rb.set_linear_damping(linear);
            rb.set_angular_damping(angular);
        });
    }

    pub fn set_friction(&mut self, friction: f32) {
        self.with_collider_mut(|c| c.set_friction(friction));
    }

    pub fn set_kinematic(&mut self, kinematic: bool) {
        self.with_rigid_body_mut(|rb| {
            if kinematic {
                rb.set_body_type(rp::RigidBodyType::KinematicPositionBased, true);
            } else {
                rb.set_body_type(rp::RigidBodyType::Dynamic, true);
            }
        });
    }

    pub fn set_restitution(&mut self, restitution: f32) {
        self.with_collider_mut(|c| c.set_restitution(restitution));
    }

    pub fn set_sleep_threshold(&mut self, linear: f32, angular: f32) {
        self.with_rigid_body_mut(|rb| {
            rb.activation_mut().linear_threshold = linear;
            rb.activation_mut().angular_threshold = angular;
        });
    }

    pub fn get_mass(&self) -> f32 {
        self.with_rigid_body(|rb| rb.mass())
    }

    /// The mass of child objects does not affect the mass or inertia of the parent
    pub fn set_mass(&mut self, mass: f32) {
        self.with_rigid_body_mut(|rb| rb.set_additional_mass(mass, true));
    }

    /// Children return the parent position.
    #[bind(name = "GetPos", out_param = true)]
    pub fn get_position(&self) -> Vec3 {
        self.get_world_matrix_unscaled().get_translation()
    }

    /// Local coordinates are relative to the parent *before* scaling.
    #[bind(name = "GetPosLocal", out_param = true)]
    pub fn get_position_local(&self) -> Vec3 {
        if let WorldState::AttachedToCompound { .. } = &self.state {
            let translation =
                self.with_collider(|c| c.position_wrt_parent().unwrap().translation.vector);
            Vec3::from_na(&translation)
        } else {
            Vec3::ZERO
        }
    }

    #[bind(name = "SetPos")]
    pub fn set_position(&mut self, pos: &Vec3) {
        self.with_rigid_body_mut(|rb| rb.set_translation(pos.to_na(), true));
    }

    /// Local coordinates are relative to the parent *before* scaling. The
    /// given position will be multiplied by the parent's scale.
    #[bind(name = "SetPosLocal")]
    pub fn set_position_local(&mut self, pos: &Vec3) {
        if let WorldState::AttachedToCompound { .. } = &self.state {
            let mut isometry = self.with_collider(|c| c.position_wrt_parent().unwrap().clone());
            isometry.translation.vector = Vec3::to_na(pos);
            self.with_collider_mut(|c| c.set_position_wrt_parent(isometry));
        }
    }

    #[bind(name = "GetRot", out_param = true)]
    pub fn get_rotation(&self) -> Quat {
        Quat::from_mat4(&self.get_world_matrix_unscaled())
    }

    #[bind(name = "GetRotLocal", out_param = true)]
    pub fn get_rotation_local(&mut self) -> Quat {
        if let WorldState::AttachedToCompound { .. } = &self.state {
            let rotation = self.with_collider(|c| c.position_wrt_parent().unwrap().rotation);
            Quat::from_na(&rotation)
        } else {
            Quat::IDENTITY
        }
    }

    #[bind(name = "SetRot")]
    pub fn set_rotation(&mut self, rot: &mut Quat) {
        self.with_rigid_body_mut(|rb| rb.set_rotation(rot.to_na(), true));
    }

    #[bind(name = "SetRotLocal")]
    pub fn set_rotation_local(&mut self, rot: &Quat) {
        if let WorldState::AttachedToCompound { .. } = &self.state {
            let mut isometry = self.with_collider(|c| c.position_wrt_parent().unwrap().clone());
            isometry.rotation = Quat::to_na(rot);
            self.with_collider_mut(|c| c.set_position_wrt_parent(isometry));
        }
    }

    pub fn get_scale(&self) -> f32 {
        self.shape_scale
    }

    /// When called on a parent object the positions of all children will be
    /// multiplied such that they retain the same relative position. Child
    /// scale is not affected by parent scale (i.e. it is not inherited). This
    /// function is O(3N).
    pub fn set_scale(&mut self, scale: f32) {
        let scaled_shape = CollisionShape::new(scale, self.shape_type.clone());
        self.with_collider_mut(|c| {
            // Replace the shape of the current collider by cloning the reference counted Shape object.
            c.set_shape(rp::SharedShape(
                scaled_shape.collider.shared_shape().0.clone(),
            ));
        });
        self.shape_type = scaled_shape.shape;
        self.shape_scale = scale;
    }
}
