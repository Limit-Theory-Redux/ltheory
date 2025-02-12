#![allow(unsafe_code)] // TODO: remove

use std::ptr::NonNull;
use std::sync::{Mutex, MutexGuard, OnceLock};

use glam::Mat4;
use rapier3d_f64::prelude as rp;
use rapier3d_f64::prelude::nalgebra as na;

use crate::math::*;
use crate::physics::*;
use crate::render::*;
use crate::rf::Rf;

pub type CollisionGroup = u32;
pub type CollisionMask = u32;

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

pub(crate) struct RigidBodyParent {
    pub rigid_body: NonNull<RigidBody>,
    pub offset: na::Isometry3<rp::Real>,
}

pub struct RigidBody {
    rigid_body: RigidBodyWrapper,
    collider: ColliderWrapper,

    // These contain raw pointers to stable memory addresses of RigidBody's.
    // TODO: Replace these with an index into the PhysicsWorld.
    parent: Option<RigidBodyParent>,
    children: Vec<NonNull<RigidBody>>,
    triggers: Vec<NonNull<Trigger>>,

    // Fields to allow us to reconstruct the collision shape object.
    shape_type: CollisionShapeType,
    shape_scale: f32,

    mass: f32,

    collidable: bool,
    collision_group: rp::InteractionGroups,
}

impl RigidBody {
    pub fn new(
        shape_scale: f32,
        shape_type: CollisionShapeType,
        shape: rp::SharedShape,
    ) -> Box<RigidBody> {
        let mut rigid_body = Box::new(RigidBody {
            rigid_body: RigidBodyWrapper::Removed(rp::RigidBodyBuilder::dynamic().build()),
            collider: ColliderWrapper::Removed(
                rp::ColliderBuilder::new(shape)
                    .restitution(0.4)
                    .mass(1.0)
                    .build(),
            ),
            parent: None,
            children: vec![],
            triggers: vec![],
            shape_type,
            shape_scale,
            mass: 1.0,
            collidable: true,
            collision_group: rp::InteractionGroups::default(),
        });

        // The collider stores a reference to the handle for this rigid body
        // in its user data, which currently is just the stable raw pointer.
        rigid_body.collider.as_mut().user_data = RigidBody::encode_as_user_data(&rigid_body);

        rigid_body
    }

    // Adds this rigid body and any children to the world.
    pub(crate) fn add_to_world(&mut self, world: Rf<PhysicsWorld>) {
        if self.collider.is_added() {
            return;
        }

        // If we have a parent, just add the collider as a child of it.
        if let Some(parent) = &self.parent {
            // Assumption: The parent is already in the world.
            let parent_ref = unsafe { parent.rigid_body.as_ref() };
            let parent_handle = parent_ref.rigid_body.added_as_ref().unwrap().0;

            // Add the collider to the parent rigid body.
            self.collider.set_added(world.clone(), |collider, w| {
                // Add the collider, then position it correctly.
                let handle =
                    w.colliders
                        .insert_with_parent(collider, *parent_handle, &mut w.rigid_bodies);
                w.get_mut(handle).set_position_wrt_parent(parent.offset);

                handle
            });
        } else {
            // Add rigid body.
            let rb_handle = self
                .rigid_body
                .set_added(world.clone(), |rb, w| w.rigid_bodies.insert(rb));

            // Add collider.
            self.collider.set_added(world.clone(), |collider, w| {
                w.colliders
                    .insert_with_parent(collider, rb_handle, &mut w.rigid_bodies)
            });

            // Recurse on children.
            for child in self.children.iter_mut() {
                unsafe { child.as_mut() }.add_to_world(world.clone());
            }
        }

        // Add triggers.
        for trigger in self.triggers.iter_mut() {
            unsafe { trigger.as_mut() }.add_to_world(world.clone());
        }
    }

    // Removes this rigid body and any children from the physics world.
    pub(crate) fn remove_from_world(
        &mut self,
        impulse_joint_set: &mut rp::ImpulseJointSet,
        multibody_joint_set: &mut rp::MultibodyJointSet,
    ) {
        if self.collider.is_removed() {
            return;
        }

        // Remove triggers from the world.
        for trigger in self.triggers.iter_mut() {
            unsafe { trigger.as_mut() }.remove_from_world();
        }

        // Remove children from the world.
        for child in self.children.iter_mut() {
            unsafe { child.as_mut() }.remove_collider_from_world();
        }

        // Remove this collider.
        self.remove_collider_from_world();

        // Remove the rigid body if this is a parent.
        if self.is_parent() {
            self.rigid_body.set_removed(|handle, w| {
                w.rigid_bodies
                    .remove(
                        handle,
                        &mut w.island_manager,
                        &mut w.colliders,
                        impulse_joint_set,
                        multibody_joint_set,
                        false,
                    )
                    .unwrap()
            });
        }
    }

    // Like remove_from_world, but just for the collider, as it doesn't need the impulse and multibody joint sets.
    // Used as a helper function in remove_from_world and detach.
    //
    // Valid for both parents and children.
    fn remove_collider_from_world(&mut self) {
        if self.collider.is_removed() {
            return;
        }

        // Remove collider.
        self.collider.set_removed(|handle, w| {
            w.colliders
                .remove(handle, &mut w.island_manager, &mut w.rigid_bodies, false)
                .unwrap()
        });
    }

    /// Links a RigidBody to a Rapier Collider, which we can later retrieve
    /// using linked_with_collider and linked_with_collider_mut.
    #[allow(clippy::borrowed_box)]
    pub(crate) fn encode_as_user_data(rb: &Box<RigidBody>) -> u128 {
        // TODO: Replace this with an arena index into the PhysicsWorld.
        &**rb as *const RigidBody as *mut RigidBody as u128
    }

    /// Retrieves a reference to the RigidBody linked to a Rapier Collider.
    ///
    /// The rest of the physics module guarantees that as long as a given
    /// collider exists, it's corresponding linked RigidBody exists as well.
    #[allow(dead_code)]
    pub(crate) fn linked_with_collider(collider: &rp::Collider) -> Option<*const RigidBody> {
        if collider.user_data != 0 {
            Some(collider.user_data as *const RigidBody)
        } else {
            None
        }
    }

    /// Retrieves a mutable reference to the RigidBody linked to a Rapier Collider.
    ///
    /// The rest of the physics module guarantees that as long as a given
    /// collider exists, it's corresponding linked RigidBody exists as well.
    pub(crate) fn linked_with_collider_mut(collider: &rp::Collider) -> Option<*mut RigidBody> {
        if collider.user_data != 0 {
            Some(collider.user_data as *mut RigidBody)
        } else {
            None
        }
    }

    /// Is this rigid body a child of another?
    pub fn is_child(&self) -> bool {
        self.parent.is_some()
    }

    /// Is this rigid body a parent?
    pub fn is_parent(&self) -> bool {
        !self.children.is_empty()
    }

    pub(crate) fn get_parent_internal(&self) -> Option<&RigidBodyParent> {
        self.parent.as_ref()
    }

    pub(crate) fn add_trigger(&mut self, trigger: &mut Trigger) {
        self.triggers.push(type_to_non_null(trigger));
    }

    pub(crate) fn remove_trigger(&mut self, trigger: &mut Trigger) {
        self.triggers.swap_remove(
            self.triggers
                .iter()
                .position(|t| t.as_ptr() == trigger as *mut _)
                .expect("trigger missing from trigger list"),
        );
    }

    /// Returns the unscaled world matrix of this rigid body.
    fn get_world_transform_unscaled(&self) -> rp::Isometry<rp::Real> {
        if let Some(parent) = self.get_parent() {
            let collider = self.collider.as_ref();
            let parent_rb = parent.rigid_body.as_ref();

            let default = rp::Isometry::default();
            let transform = collider.position_wrt_parent().unwrap_or(&default); //.expect(format!("child {:?} does not have a position_wrt parent {:?}", self as *const _, parent).as_str());
            let parent_transform = parent_rb.position();
            parent_transform * transform
        } else {
            *self.rigid_body.as_ref().position()
        }
    }

    // Returns a reference to the collider object.
    pub fn get_collider_ref(&self) -> RefOrBorrow<'_, rp::Collider> {
        self.collider.as_ref()
    }

    // Returns the rapier handle of the root rigid body if this rigid body is
    // added to the world or is attached to another, None otherwise.
    pub fn get_rigid_body_handle(&self) -> Option<rp::RigidBodyHandle> {
        if self.is_child() {
            unsafe {
                self.parent
                    .as_ref()
                    .unwrap()
                    .rigid_body
                    .as_ref()
                    .get_rigid_body_handle()
            }
        } else if self.rigid_body.is_added() {
            Some(*self.rigid_body.added_as_ref().unwrap().0)
        } else {
            None
        }
    }

    fn shape_cache() -> MutexGuard<'static, ShapeCache> {
        static INST: OnceLock<Mutex<ShapeCache>> = OnceLock::new();
        INST.get_or_init(|| Mutex::new(ShapeCache::new()))
            .lock()
            .unwrap()
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl RigidBody {
    #[bind(name = "CreateBox")]
    pub fn new_box() -> Box<RigidBody> {
        let shape_type = CollisionShapeType::Box {
            half_extents: Vec3::ONE,
        };
        let shape = Self::shape_cache().get(1.0, &shape_type);
        Self::new(1.0, shape_type, shape)
    }

    #[bind(name = "CreateBoxFromMesh")]
    pub fn new_box_from_mesh(mesh: &mut Mesh) -> Box<RigidBody> {
        let mut bounds = Box3::default();
        mesh.get_bound(&mut bounds);
        let shape_type = CollisionShapeType::Box {
            half_extents: Vec3::new(
                f32::max(f32::abs(bounds.upper.x), f32::abs(bounds.lower.x)),
                f32::max(f32::abs(bounds.upper.y), f32::abs(bounds.lower.y)),
                f32::max(f32::abs(bounds.upper.z), f32::abs(bounds.lower.z)),
            ),
        };
        let shape = Self::shape_cache().get(1.0, &shape_type);
        Self::new(1.0, shape_type, shape)
    }

    #[bind(name = "CreateSphere")]
    pub fn new_sphere() -> Box<RigidBody> {
        let shape_type = CollisionShapeType::Sphere { radius: 1.0 };
        let shape = Self::shape_cache().get(1.0, &shape_type);
        Self::new(1.0, shape_type, shape)
    }

    #[bind(name = "CreateSphereFromMesh")]
    pub fn new_sphere_from_mesh(mesh: &mut Mesh) -> Box<RigidBody> {
        let shape_type = CollisionShapeType::Sphere {
            radius: mesh.get_radius(),
        };
        let shape = Self::shape_cache().get(1.0, &shape_type);
        Self::new(1.0, shape_type, shape)
    }

    #[bind(name = "CreateConvexHullFromMesh")]
    pub fn new_convex_hull_from_mesh(mesh: &mut Mesh) -> Box<RigidBody> {
        let shape_type = CollisionShapeType::ConvexHull { mesh: mesh.clone() };
        let shape = Self::shape_cache().get(1.0, &shape_type);
        Self::new(1.0, shape_type, shape)
    }

    #[bind(name = "CreateConvexDecompositionFromMesh")]
    pub fn new_convex_decomposition_from_mesh(mesh: &mut Mesh) -> Box<RigidBody> {
        let shape_type = CollisionShapeType::ConvexDecomposition { mesh: mesh.clone() };
        let shape = Self::shape_cache().get(1.0, &shape_type);
        Self::new(1.0, shape_type, shape)
    }

    #[bind(name = "CreateTrimeshFromMesh")]
    pub fn new_trimesh_from_mesh(mesh: &mut Mesh) -> Box<RigidBody> {
        let shape_type = CollisionShapeType::Trimesh { mesh: mesh.clone() };
        let shape = Self::shape_cache().get(1.0, &shape_type);
        Self::new(1.0, shape_type, shape)
    }

    /// Return a reference to the parent rigid body, that we can guarantee
    /// has a lifetime as long as self.
    #[bind(name = "GetParentBody")]
    pub fn get_parent(&self) -> Option<&mut RigidBody> {
        self.parent
            .as_ref()
            .map(|parent| unsafe { &mut *parent.rigid_body.as_ptr() })
    }

    pub fn apply_force(&mut self, force: &Vec3) {
        self.rigid_body.as_mut().add_force(force.to_na(), true);
    }

    pub fn apply_torque(&mut self, torque: &Vec3) {
        self.rigid_body.as_mut().add_torque(torque.to_na(), true);
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
    /// This function expects that the child is not already in the physics
    /// world, as it will add it if the parent is already in the world.
    ///
    /// This function assumes that `self` is not already a child.
    pub fn attach(&mut self, child: &mut RigidBody, pos: &Vec3, rot: &Quat) {
        if std::ptr::eq(self, child) {
            panic!("Cannot attach object to itself!");
        }

        if self.is_child() {
            panic!(
                "Recursive attachment is not supported. Parent is already attached to something."
            );
        }

        if child.collider.is_added() && child.rigid_body.is_added() {
            panic!("Child has not been removed from physics.");
        }

        if child.is_child() {
            panic!("Child is already attached to a parent.");
        }

        // Compute the colliders relative position, scaled by the scale of the parent shape.
        let scaled_pos = *pos * self.shape_scale;

        // Set the parent-child link.
        self.children.push(type_to_non_null(child));
        child.parent = Some(RigidBodyParent {
            rigid_body: type_to_non_null(self),
            offset: na::Isometry3::from_parts(scaled_pos.to_na().into(), (*rot).to_na()),
        });

        // Set this childs mass to a negligible value.
        child.collider.as_mut().set_mass(0.000001);

        // Multiple colliders in Rapier can be attached to a single
        // rigid body, so there's no need to create a "compound shape"

        // If the rigid body is in the world, add the child now.
        if self.rigid_body.is_added() {
            child.add_to_world(self.rigid_body.added_as_ref().unwrap().1.clone());
        }
    }

    /// Removes a rigid body as a child of this rigid body. This means that
    /// the child's will be under control of it's own position.
    ///
    /// This function will result in a child that is not in the world anymore,
    /// so it will need to be re-added with physics.add_rigid_body(...).
    ///
    /// This function assumes that `self` is not already a child.
    pub fn detach(&mut self, child: &mut RigidBody) {
        if child.parent.is_none() || child.collider.is_removed() {
            panic!("Child is not attached to parent.");
        }

        if child.parent.as_ref().unwrap().rigid_body.as_ptr() != (self as *mut RigidBody) {
            panic!("Child is attached to a different rigid body.");
        }

        // Convert current transform to world coordinates.
        let parent_transform = *self.rigid_body.as_ref().position();

        // Compute the combined transform, then update the rigid body
        // transform of the child so it's in the right place once it's
        // re-added to the world.
        let combined_transform = parent_transform * child.parent.as_ref().unwrap().offset;
        child
            .rigid_body
            .as_mut()
            .set_position(combined_transform, true);

        // Remove from the world if needed.
        if child.collider.is_added() {
            child.remove_collider_from_world();
        }

        // Reset the child collider's mass.
        child.collider.as_mut().set_mass(child.mass as rp::Real);

        // Break parent-child link.
        self.children.swap_remove(
            self.children
                .iter()
                .position(|rb| rb.as_ptr() == child as *mut _)
                .expect("child missing from children list"),
        );
        child.parent = None;
    }

    /// Calculates the bounding box.
    #[bind(out_param = true)]
    pub fn get_bounding_box(&self) -> Box3 {
        let aabb = self.collider.as_ref().compute_aabb();
        Box3::new(
            Vec3::from_na_point(&aabb.mins),
            Vec3::from_na_point(&aabb.maxs),
        )
    }

    /// Calculates the compound bounding box.
    #[bind(out_param = true)]
    pub fn get_bounding_box_compound(&self) -> Box3 {
        if !self.is_parent() {
            panic!("Only enabled for parents");
        }

        if !self.collider.is_added() {
            panic!("Only enabled when added to the world");
        }

        let (collider_handle, world) = self.collider.added_as_ref().unwrap();
        let w = &mut *world.as_mut();

        // Get AABB of the main collider.
        let mut aabb = w.get(*collider_handle).compute_aabb();
        let parent_transform = *w.get(*collider_handle).position();

        // Incorporate the AABBs of any children.
        for child in self.children.iter() {
            let collider = unsafe { child.as_ref() }.collider.as_ref();
            let child_global_transform = parent_transform * collider.position_wrt_parent().unwrap();
            let child_aabb = collider.shape().compute_aabb(&child_global_transform);
            aabb.mins = aabb.mins.inf(&child_aabb.mins);
            aabb.maxs = aabb.maxs.sup(&child_aabb.maxs);
        }

        Box3::new(
            Vec3::from_na_point(&aabb.mins),
            Vec3::from_na_point(&aabb.maxs),
        )
    }

    /// Calculates the local bounding box.
    #[bind(out_param = true)]
    pub fn get_bounding_box_local(&self) -> Box3 {
        let aabb = self.collider.as_ref().shape().compute_local_aabb();
        Box3::new(
            Vec3::from_na_point(&aabb.mins),
            Vec3::from_na_point(&aabb.maxs),
        )
    }

    /// Calculates the local compound bounding box.
    #[bind(out_param = true)]
    pub fn get_bounding_box_local_compound(&self) -> Box3 {
        if !self.is_parent() {
            panic!("Only enabled for parents");
        }

        if !self.collider.is_added() {
            panic!("Only enabled when added to the world");
        }

        let (collider_handle, world) = self.collider.added_as_ref().unwrap();
        let w = &mut *world.as_mut();

        // Get AABB of the main collider.
        let mut aabb = w.get(*collider_handle).shape().compute_local_aabb();

        // Incorporate the AABBs of any children.
        for child in self.children.iter() {
            let collider = unsafe { child.as_ref() }.collider.as_ref();
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
    }

    pub fn get_bounding_radius(&self) -> f32 {
        self.collider
            .as_ref()
            .shape()
            .compute_local_bounding_sphere()
            .radius() as f32
    }

    pub fn get_bounding_radius_compound(&self) -> f32 {
        // Compute the compounds bounding radius by taking the max of all child colliders bounding radii, offset by its local position.
        let mut max_radius = self.get_bounding_radius();
        for child in self.children.iter() {
            let collider = unsafe { child.as_ref() }.collider.as_ref();
            let child_bounding_sphere = collider
                .shape()
                .compute_bounding_sphere(collider.position_wrt_parent().unwrap());
            max_radius = f32::max(max_radius, child_bounding_sphere.radius() as f32);
        }
        max_radius
    }

    pub fn get_speed(&self) -> f32 {
        self.rigid_body.as_ref().linvel().norm() as f32
    }

    /// Returns the local -> world matrix for this rigid body.
    ///
    /// This assumes that the world matrix relative to the cameras frame of reference i.e. the camera is always at the origin.
    pub fn get_to_world_matrix(&self, camera_pos: &Position) -> Matrix {
        (*Matrix::from_rp(&self.get_world_transform_unscaled(), camera_pos)
            * Mat4::from_scale(Vec3::splat(self.get_scale())))
        .into()
    }

    /// Returns the world -> local matrix for this rigid body.
    ///
    /// This assumes that the world matrix relative to the cameras frame of reference i.e. the camera is always at the origin.
    pub fn get_to_local_matrix(&self, camera_pos: &Position) -> Matrix {
        self.get_to_world_matrix(camera_pos).inverse()
    }

    #[bind(out_param = true)]
    pub fn get_velocity(&self) -> Vec3 {
        Vec3::from_na(self.rigid_body.as_ref().linvel())
    }

    #[bind(name = "GetVelocityA", out_param = true)]
    pub fn get_angular_velocity(&self) -> Vec3 {
        Vec3::from_na(self.rigid_body.as_ref().angvel())
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
        self.collider.as_mut().set_collision_groups(collision_group);
    }

    pub fn set_collision_group(&mut self, group: u32) {
        self.collision_group.memberships = group.into();
        let collision_group = if self.collidable {
            self.collision_group
        } else {
            rp::InteractionGroups::none()
        };
        self.collider.as_mut().set_collision_groups(collision_group);
    }

    pub fn set_collision_mask(&mut self, mask: u32) {
        self.collision_group.filter = mask.into();
        let collision_group = if self.collidable {
            self.collision_group
        } else {
            rp::InteractionGroups::none()
        };
        self.collider.as_mut().set_collision_groups(collision_group);
    }

    pub fn set_drag(&mut self, linear: f32, angular: f32) {
        let mut rb = self.rigid_body.as_mut();
        rb.set_linear_damping(linear as rp::Real);
        rb.set_angular_damping(angular as rp::Real);
    }

    pub fn set_friction(&mut self, friction: f32) {
        self.collider.as_mut().set_friction(friction as rp::Real);
    }

    pub fn set_kinematic(&mut self, kinematic: bool) {
        let body_type = if kinematic {
            rp::RigidBodyType::KinematicPositionBased
        } else {
            rp::RigidBodyType::Dynamic
        };
        self.rigid_body.as_mut().set_body_type(body_type, true);
    }

    pub fn set_restitution(&mut self, restitution: f32) {
        self.collider
            .as_mut()
            .set_restitution(restitution as rp::Real);
    }

    pub fn set_sleep_threshold(&mut self, linear: f32, angular: f32) {
        let mut rb = self.rigid_body.as_mut();
        rb.activation_mut().normalized_linear_threshold = linear as rp::Real;
        rb.activation_mut().angular_threshold = angular as rp::Real;
    }

    pub fn get_mass(&self) -> f32 {
        self.mass
    }

    /// The mass of child objects does not affect the mass or inertia of the parent
    pub fn set_mass(&mut self, mass: f32) {
        self.mass = mass;

        // Only update the colliders mass if we're not attached to something, as
        // the expectation is that a child collider's mass does not contribute
        // to the parent's mass.
        if !self.is_child() {
            self.collider.as_mut().set_mass(mass as rp::Real);
        }
    }

    /// Children return the parent position.
    #[bind(name = "GetPos", out_param = true)]
    pub fn get_position(&self) -> Position {
        Position::from_na(&self.get_world_transform_unscaled().translation.vector)
    }

    /// Local coordinates are relative to the parent *before* scaling.
    #[bind(name = "GetPosLocal", out_param = true)]
    pub fn get_position_local(&self) -> Position {
        if let Some(parent) = &self.parent {
            Position::from_na(&parent.offset.translation.vector)
        } else {
            Position::ZERO
        }
    }

    #[bind(name = "SetPos")]
    pub fn set_position(&mut self, pos: &Position) {
        self.rigid_body.as_mut().set_translation(pos.to_na(), true);
    }

    /// Local coordinates are relative to the parent *before* scaling. The
    /// given position will be multiplied by the parent's scale.
    #[bind(name = "SetPosLocal")]
    pub fn set_position_local(&mut self, pos: &Position) {
        if let Some(parent) = &mut self.parent {
            parent.offset.translation.vector = pos.to_na();
            self.collider
                .as_mut()
                .set_position_wrt_parent(parent.offset);

            for trigger in self.triggers.iter_mut() {
                unsafe { trigger.as_mut() }.refresh_collider_offset();
            }
        }
    }

    #[bind(name = "GetRot", out_param = true)]
    pub fn get_rotation(&self) -> Quat {
        Quat::from_na(&self.get_world_transform_unscaled().rotation)
    }

    #[bind(name = "GetRotLocal", out_param = true)]
    pub fn get_rotation_local(&self) -> Quat {
        if let Some(parent) = &self.parent {
            Quat::from_na(&parent.offset.rotation)
        } else {
            Quat::identity()
        }
    }

    #[bind(name = "SetRot")]
    pub fn set_rotation(&mut self, rot: &mut Quat) {
        self.rigid_body.as_mut().set_rotation(rot.to_na(), true);
    }

    #[bind(name = "SetRotLocal")]
    pub fn set_rotation_local(&mut self, rot: &Quat) {
        if let Some(parent) = &mut self.parent {
            parent.offset.rotation = Quat::to_na(rot);
            self.collider
                .as_mut()
                .set_position_wrt_parent(parent.offset);

            for trigger in self.triggers.iter_mut() {
                unsafe { trigger.as_mut() }.refresh_collider_offset();
            }
        }
    }

    pub fn get_scale(&self) -> f32 {
        self.shape_scale
    }

    /// When called on a parent object the positions of all children will be
    /// multiplied such that they retain the same relative position. Child
    /// scale is not affected by parent scale (i.e. it is not inherited).
    pub fn set_scale(&mut self, scale: f32) {
        if scale == self.shape_scale {
            return;
        }

        let scale_ratio = scale / self.shape_scale;

        // Update shape.
        self.shape_scale = scale;
        self.collider
            .as_mut()
            .set_shape(Self::shape_cache().get(scale, &self.shape_type));

        // Children keep the same relative position.
        for child in self.children.iter_mut() {
            let child = unsafe { child.as_mut() };
            let scaled_position = child.get_position_local() * scale_ratio as f64;
            child.set_position_local(&scaled_position);
        }

        for trigger in self.triggers.iter_mut() {
            let trigger = unsafe { trigger.as_mut() };
            let scaled_position = trigger.get_position_local() * scale_ratio as f64;
            trigger.set_position_local(&scaled_position);
        }
    }

    pub fn distance_to(&self, target: &RigidBody) -> f64 {
        let my_position = self.get_position();
        let target_position = target.get_position();

        my_position.distance(target_position)
    }

    pub fn is_sleeping(&self) -> bool {
        self.rigid_body.as_ref().is_sleeping()
    }
}

fn type_to_non_null<T>(reference: &mut T) -> NonNull<T> {
    NonNull::new(reference as *mut _).unwrap()
}
