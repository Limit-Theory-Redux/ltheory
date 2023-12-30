use crate::common::*;
use crate::math::*;
use crate::physics::*;
use crate::render::*;
use crate::rf::Rf;
use rapier3d::prelude as rp;
use rapier3d::prelude::nalgebra as na;
use std::cell::RefCell;
use std::mem::replace;
use std::rc::Rc;
use tracing::debug;

// TODO: Remove this state transition thing in favour of
// states for RigidBody/RigidBodyHandle (depending on
// whether it's in the world or not). Then use that
// primitive instead.
enum State {
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
        world: Rf<PhysicsWorld>,
        children: Vec<rp::ColliderHandle>,
    },
    // Added to physics, and attached to another rigid body.
    AttachedToParent {
        parent: *mut RigidBody, // Raw pointer to stable memory address of parent (as it's in a Box).
        rb: rp::RigidBody, // Unused rapier RB, which we'd use again if the rigid body is detached.
        collider_handle: rp::ColliderHandle,
        world: Rf<PhysicsWorld>,
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
    // Stores the rigid body / collider state machine.
    state: State,

    // Fields to allow us to reconstruct the collision shape object.
    shape_type: CollisionShapeType,
    shape_scale: f32,

    collidable: bool,
    collision_group: rp::InteractionGroups,
    mass: f32,
}

// Functions to add and remove the rigid body from physics.
impl RigidBody {
    /// Links a RigidBody to a Rapier Collider, which we can later retrieve
    /// using linked_with_collider and linked_with_collider_mut.
    pub(crate) fn link_with_collider(rb: &mut Box<RigidBody>, collider: &mut rp::Collider) {
        collider.user_data = &mut **rb as *mut RigidBody as u128;
    }

    /// Retrieves a reference to the RigidBody linked to a Rapier Collider.
    ///
    /// The rest of the physics module guarantees that as long as a given
    /// collider exists, it's corresponding linked RigidBody exists as well.
    pub(crate) fn linked_with_collider(collider: &rp::Collider) -> Option<&'_ RigidBody> {
        if collider.user_data != 0 {
            let raw_ptr = collider.user_data as *const RigidBody;
            Some(unsafe { &*raw_ptr })
        } else {
            None
        }
    }

    /// Retrieves a mutable reference to the RigidBody linked to a Rapier Collider.
    ///
    /// The rest of the physics module guarantees that as long as a given
    /// collider exists, it's corresponding linked RigidBody exists as well.
    pub(crate) fn linked_with_collider_mut(collider: &rp::Collider) -> Option<&'_ mut RigidBody> {
        if collider.user_data != 0 {
            let raw_ptr = collider.user_data as *mut RigidBody;
            Some(unsafe { &mut *raw_ptr })
        } else {
            None
        }
    }

    pub(crate) fn add_to_world(
        &mut self,
        world: Rf<PhysicsWorld>,
    ) -> Option<(rp::ColliderHandle, rp::RigidBodyHandle)> {
        // It only makes sense to add to the world if we're removed.
        if let State::Removed { rb, collider } = replace(&mut self.state, State::None) {
            let (rb_handle, collider_handle) = {
                let w = &mut *world.as_mut();
                let rb_handle = w.rigid_bodies.insert(rb);
                let collider_handle =
                    w.colliders
                        .insert_with_parent(collider, rb_handle, &mut w.rigid_bodies);
                (rb_handle, collider_handle)
            };
            self.state = State::Added {
                rb_handle,
                collider_handle,
                children: vec![],
                world,
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
        if let State::Added {
            rb_handle,
            collider_handle,
            world,
            children,
        } = replace(&mut self.state, State::None)
        {
            let w = &mut *world.as_mut();
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
            debug!(
                "Removing rigid body {:?} with {:?} children",
                self as *mut _, children
            );
            self.state = State::Removed {
                rb: rigid_body,
                collider,
            };
            Some((collider_handle, rb_handle))
        } else {
            None
        }
    }

    /// Executes a function f with a reference to the RigidBody associated with this object.
    pub(crate) fn with_rigid_body<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&rp::RigidBody) -> R,
    {
        match &self.state {
            State::None => panic!("Uninitialized RigidBody."),
            State::Removed { rb, .. } => f(rb),
            State::Added {
                rb_handle, world, ..
            } => f(world.as_ref().get(*rb_handle)),
            State::AttachedToParent { .. } => panic!("Not supported on children."),
        }
    }

    /// Executes a function f with a mutable reference to the RigidBody associated with this object.
    fn with_rigid_body_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut rp::RigidBody) -> R,
    {
        match &mut self.state {
            State::None => panic!("Uninitialized RigidBody."),
            State::Removed { rb, .. } => f(rb),
            State::Added {
                rb_handle, world, ..
            } => f(world.as_mut().get_mut(*rb_handle)),
            State::AttachedToParent { .. } => panic!("Not supported on children."),
        }
    }

    /// Executes a function f with a reference to the collider associated with this object.
    pub(crate) fn with_collider<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&rp::Collider) -> R,
    {
        match &self.state {
            State::None => panic!("Uninitialized RigidBody."),
            State::Removed { collider, .. } => f(collider),
            State::Added {
                collider_handle,
                world,
                ..
            } => f(world.as_ref().get(*collider_handle)),
            State::AttachedToParent {
                collider_handle,
                world,
                ..
            } => f(world.as_ref().get(*collider_handle)),
        }
    }

    /// Executes a function f with a mutable reference to the collider associated with this object.
    fn with_collider_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut rp::Collider) -> R,
    {
        match &mut self.state {
            State::None => panic!("Uninitialized RigidBody."),
            State::Removed { collider, .. } => f(collider),
            State::Added {
                collider_handle,
                world,
                ..
            } => f(world.as_mut().get_mut(*collider_handle)),
            State::AttachedToParent {
                collider_handle,
                world,
                ..
            } => f(world.as_mut().get_mut(*collider_handle)),
        }
    }

    pub fn new(mut shape: CollisionShape) -> Box<RigidBody> {
        let mut rigid_body = Box::new(RigidBody {
            state: State::None,
            shape_type: shape.shape,
            shape_scale: shape.scale,
            collidable: true,
            collision_group: rp::InteractionGroups::default(),
            mass: 1.0,
        });

        // The collider stores a reference to the handle for this rigid body
        // in its user data, which currently is just the stable raw pointer.
        RigidBody::link_with_collider(&mut rigid_body, &mut shape.collider);

        // Set initial state and return.
        rigid_body.state = State::Removed {
            rb: rp::RigidBodyBuilder::dynamic().build(),
            collider: shape.collider,
        };
        rigid_body
    }

    /// Is this rigid body part of a compound shape?
    pub fn is_in_compound(&self) -> bool {
        self.is_root_in_compound() || self.is_child()
    }

    /// Is this rigid body a child of another?
    pub fn is_child(&self) -> bool {
        if let State::AttachedToParent { .. } = &self.state {
            true
        } else {
            false
        }
    }

    /// Is this rigid body part of a compound shape, and is also the root?
    pub fn is_root_in_compound(&self) -> bool {
        if let State::Added { children, .. } = &self.state {
            !children.is_empty()
        } else {
            false
        }
    }

    /// Returns the unscaled world matrix of this rigid body.
    fn get_world_matrix_unscaled(&self) -> Matrix {
        let global_transform = if let State::AttachedToParent { parent, .. } = &self.state {
            self.with_collider(|c| {
                unsafe { &**parent }.with_rigid_body(|parent_rb| {
                    let default = rp::Isometry::default();
                    let transform = c.position_wrt_parent().unwrap_or(&default); //.expect(format!("child {:?} does not have a position_wrt parent {:?}", self as *const _, parent).as_str());
                    let parent_transform = parent_rb.position();
                    parent_transform * transform
                })
            })
        } else {
            self.with_rigid_body(|rb| rb.position().clone())
        };
        Matrix::from_rp(&global_transform)
    }

    // Returns the rapier handle of the root rigid body if this rigid body is
    // added to the world or is attached to another, None otherwise.
    pub fn get_rigid_body_handle(&self) -> Option<rp::RigidBodyHandle> {
        match &self.state {
            State::Added { rb_handle, .. } => Some(*rb_handle),
            State::AttachedToParent { parent, .. } => unsafe { (**parent).get_rigid_body_handle() },
            _ => None,
        }
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
    pub fn new_hull_from_mesh(mesh: &Mesh) -> Box<RigidBody> {
        Self::new(CollisionShape::new_hull_from_mesh(mesh))
    }

    /// Return a reference to the parent rigid body, that we can guarantee
    /// has a lifetime as long as self.
    #[bind(name = "GetParentBody")]
    pub fn get_parent(&self) -> Option<&mut RigidBody> {
        match &self.state {
            State::AttachedToParent { parent, .. } => {
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
        match &mut self.state {
            State::None => {}
            State::AttachedToParent { .. } => {
                panic!("Recursive attachment is not supported. Parent is already attached to something.");
            }
            State::Removed { .. } => {
                panic!("Parent has been removed from physics.");
            }
            State::Added {
                rb_handle: parent_handle,
                world,
                children,
                ..
            } => {
                child.state = match replace(&mut child.state, State::None) {
                    State::None => {
                        panic!("Child is not initialised");
                    }
                    State::Added { .. } => {
                        panic!("Child has not been removed from physics.");
                    }
                    State::AttachedToParent { .. } => {
                        panic!("Child is already attached to a parent.");
                    }
                    State::Removed { rb, collider } => {
                        let w = &mut *world.as_mut();

                        debug!(
                            "Attaching rigid body {:?} to {:?}",
                            child as *mut _, parent_ptr
                        );

                        // Multiple colliders in Rapier can be attached to a single
                        // rigid body, so there's no need to create a "compound shape"

                        // Add the collider to the parent rigid body.
                        let collider_handle = w.colliders.insert_with_parent(
                            collider,
                            *parent_handle,
                            &mut w.rigid_bodies,
                        );
                        children.push(collider_handle);

                        // Set the colliders relative position, scaled by the scale of the parent shape.
                        let scaled_pos = *pos * self.shape_scale;
                        w.get_mut(collider_handle).set_position_wrt_parent(
                            na::Isometry3::from_parts(scaled_pos.to_na().into(), rot.to_na()),
                        );
                        State::AttachedToParent {
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
        child.state = match replace(&mut child.state, State::None) {
            State::None => {
                panic!("Child is not initialised");
            }
            State::Added { .. } | State::Removed { .. } => {
                panic!("Child is not attached to parent.");
            }
            State::AttachedToParent {
                mut rb,
                parent,
                collider_handle,
                world,
            } => {
                if parent != (self as *mut RigidBody) {
                    panic!("Child is not attached to this rigid body.");
                }

                // Convert current transform to world coordinates.
                let parent_transform = self.with_rigid_body(|rb| rb.position().clone());

                // Get a mutable ref to the physics world.
                let w = &mut *world.as_mut();

                // Compute the combined transform.
                let child_transform = w.get(collider_handle).position_wrt_parent().unwrap();
                let combined_transform = parent_transform * child_transform;

                // TODO: Store child transform state so we can reconstruct when re-adding to the world.

                // Detach from parent by removing from the collider set.
                let collider = w
                    .colliders
                    .remove(
                        collider_handle,
                        &mut w.island_manager,
                        &mut w.rigid_bodies,
                        true,
                    )
                    .unwrap();
                rb.set_position(combined_transform, true);

                // Now we're in the 'removed' state.
                State::Removed { rb, collider }
            }
        };
    }

    /// Calculates the bounding box.
    #[bind(out_param = true)]
    pub fn get_bounding_box(&self) -> Box3 {
        let aabb = self.with_collider(|c| c.compute_aabb());
        Box3::new(
            Vec3::from_na_point(&aabb.mins),
            Vec3::from_na_point(&aabb.maxs),
        )
    }

    /// Calculates the compound bounding box.
    #[bind(out_param = true)]
    pub fn get_bounding_box_compound(&self) -> Box3 {
        if let State::Added {
            collider_handle,
            children,
            world,
            ..
        } = &self.state
        {
            let w = &mut *world.as_mut();

            // Get AABB of the main collider.
            let mut aabb = w.get(*collider_handle).compute_aabb();
            let parent_transform = w.get(*collider_handle).position().clone();

            // Incorporate the AABBs of the compound shapes.
            for child_collider_handle in children.iter() {
                let collider = w.get_mut(*child_collider_handle);
                let child_global_transform =
                    parent_transform * collider.position_wrt_parent().unwrap();
                let child_aabb = collider.shape().compute_aabb(&child_global_transform);
                aabb.mins = aabb.mins.inf(&child_aabb.mins);
                aabb.maxs = aabb.maxs.sup(&child_aabb.maxs);
            }

            Box3::new(
                Vec3::from_na_point(&aabb.mins),
                Vec3::from_na_point(&aabb.maxs),
            )
        } else {
            Box3::default()
        }
    }

    /// Calculates the local bounding box.
    #[bind(out_param = true)]
    pub fn get_bounding_box_local(&self) -> Box3 {
        let aabb = self.with_collider(|c| c.shape().compute_local_aabb());
        Box3::new(
            Vec3::from_na_point(&aabb.mins),
            Vec3::from_na_point(&aabb.maxs),
        )
    }

    /// Calculates the local compound bounding box.
    #[bind(out_param = true)]
    pub fn get_bounding_box_local_compound(&self) -> Box3 {
        if let State::Added {
            collider_handle,
            children,
            world,
            ..
        } = &self.state
        {
            let w = &mut *world.as_mut();

            // Get AABB of the main collider.
            let mut aabb = w.get(*collider_handle).shape().compute_local_aabb();

            // Incorporate the AABBs of the compound shapes.
            for child_collider_handle in children.iter() {
                let collider = w.get_mut(*child_collider_handle);
                let child_aabb = collider
                    .shape()
                    .compute_aabb(collider.position_wrt_parent().unwrap());
                aabb.mins = aabb.mins.inf(&child_aabb.mins);
                aabb.maxs = aabb.maxs.sup(&child_aabb.maxs);
            }

            Box3::new(
                Vec3::from_na_point(&aabb.mins),
                Vec3::from_na_point(&aabb.maxs),
            )
        } else {
            Box3::default()
        }
    }

    pub fn get_bounding_radius(&self) -> f32 {
        self.get_bounding_box_local().half_extents().length()
    }

    pub fn get_bounding_radius_compound(&self) -> f32 {
        self.get_bounding_box_local_compound()
            .half_extents()
            .length()
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
        if let State::AttachedToParent { .. } = &self.state {
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
        if let State::AttachedToParent { .. } = &self.state {
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
        if let State::AttachedToParent { .. } = &self.state {
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
        if let State::AttachedToParent { .. } = &self.state {
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
