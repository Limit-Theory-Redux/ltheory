use crate::collision_shape::*;
use crate::common::*;
use crate::math::{Box3, Vec3};
use crate::matrix::*;
use crate::mesh::*;
use crate::physics::*;
use crate::quat::Quat;
use rapier3d::prelude as rp;
use rapier3d::prelude::nalgebra as na;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub enum PhysicsType {
    Null,
    RigidBody,
    Trigger,
}

pub(crate) enum RigidBodyState {
    None,
    Removed {
        rb: rp::RigidBody,
        collider: rp::Collider,
    },
    Added {
        rb: rp::RigidBodyHandle,
        collider: rp::ColliderHandle,
        world: Weak<RefCell<PhysicsWorld>>,
    },
    AddedWithinCompound {
        parent: *mut RigidBody, // Raw pointer to stable memory address of parent (as it's in a Box).
        collider: rp::ColliderHandle,
        world: Weak<RefCell<PhysicsWorld>>,
    },
}

impl RigidBodyState {
    pub(crate) fn add_to_world(self, world: &Rc<RefCell<PhysicsWorld>>) -> RigidBodyState {
        if let RigidBodyState::Removed { rb, collider } = self {
            let mut w = &mut *world.borrow_mut();
            let rigid_body_handle = w.rigid_body_set.insert(rb);
            let collider_handle = w.collider_set.insert_with_parent(
                collider,
                rigid_body_handle,
                &mut w.rigid_body_set,
            );
            RigidBodyState::Added {
                rb: rigid_body_handle,
                collider: collider_handle,
                world: Rc::downgrade(world),
            }
        } else {
            // Do nothing, keep existing state.
            self
        }
    }

    pub(crate) fn remove_from_world(
        self,
        island_manager: &mut rp::IslandManager,
        impulse_joint_set: &mut rp::ImpulseJointSet,
        multibody_joint_set: &mut rp::MultibodyJointSet,
    ) -> RigidBodyState {
        if let RigidBodyState::Added {
            rb: rigid_body_handle,
            collider: collider_handle,
            world: world,
        } = self
        {
            let world_rc = world.upgrade().expect("physics world was freed");
            let mut w = &mut *world_rc.borrow_mut();
            let collider = w
                .collider_set
                .remove(
                    collider_handle,
                    island_manager,
                    &mut w.rigid_body_set,
                    false,
                )
                .unwrap();
            let rigid_body = w
                .rigid_body_set
                .remove(
                    rigid_body_handle,
                    island_manager,
                    &mut w.collider_set,
                    impulse_joint_set,
                    multibody_joint_set,
                    false,
                )
                .unwrap();
            RigidBodyState::Removed {
                rb: rigid_body,
                collider,
            }
        } else {
            // Do nothing, keep existing state.
            self
        }
    }
}

/*
struct RigidBody {
  PhysicsType  type;
  btRigidBody* handle;         // Always references *this* object, even when part of a compound
  int          iShape;         // Always references *this* object, even when part of a compound
  int          collision_group; // Which group this object is part of
  int          collisionMask;  // Which other groups this object collides with
  float        mass;           // For GetMass and calculating inertia (mass is not stored in btRigidBody)

  int          iCompound;      // The index within the compound (-1 when not a compound)
  int          iCompoundShape; // The compound shape (-1 when not a compound)
  RigidBody*   parent;         // The parent object in the compound (null when not a compound)
  RigidBody*   next;           // The next object in the compound (null when this is the last child or not a compound)

  Physics*     physics;
  Trigger*     triggers;
  Matrix       mat;
};

 */

pub struct RigidBody {
    ty: PhysicsType,

    // Stores the rigid body / collider state.
    pub(crate) state: RigidBodyState,

    // Fields to allow us to reconstruct the collision shape object.
    shape_type: CollisionShapeType,
    shape_scale: f32,

    collidable: bool,
    collision_group: rp::InteractionGroups,
    mass: f32,
}

impl RigidBody {
    pub fn new(shape: CollisionShape) -> Box<RigidBody> {
        let rigidBody = rp::RigidBodyBuilder::dynamic().build();
        Box::new(RigidBody {
            ty: PhysicsType::RigidBody,
            state: RigidBodyState::Removed {
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

    pub fn new_box() -> Box<RigidBody> {
        Self::new(CollisionShape::new_box(&Vec3::ONE))
    }

    pub fn new_box_from_mesh(mesh: &mut Mesh) -> Box<RigidBody> {
        Self::new(CollisionShape::new_box_from_mesh(mesh))
    }

    pub fn new_sphere() -> Box<RigidBody> {
        Self::new(CollisionShape::new_sphere(1.0))
    }

    pub fn new_sphere_from_mesh(mesh: &mut Mesh) -> Box<RigidBody> {
        Self::new(CollisionShape::new_sphere_from_mesh(mesh))
    }

    pub fn new_hull_from_mesh(mesh: Rc<Mesh>) -> Box<RigidBody> {
        Self::new(CollisionShape::new_hull_from_mesh(mesh))
    }

    /// Is this rigid body part of a compound shape?
    pub fn is_in_compound(&self) -> bool {
        self.is_root_in_compound() || self.is_child()
    }

    /// Is this rigid body a child of the root in a compound shape?
    pub fn is_child(&self) -> bool {
        if let RigidBodyState::AddedWithinCompound {
            parent: _,
            collider: _,
            world: _,
        } = &self.state
        {
            true
        } else {
            false
        }
    }

    /// Is this rigid body part of a compound shape, and is also the root?
    pub fn is_root_in_compound(&self) -> bool {
        // TODO: The collider is a compound shape
        false
    }

    /// Return a reference to the parent rigid body, that we can guarantee
    /// has a lifetime as long as self.
    pub fn get_parent(&self) -> Option<&mut RigidBody> {
        match &self.state {
            RigidBodyState::AddedWithinCompound {
                parent,
                collider: _,
                world: _,
            } => {
                // SAFETY: This only works if the pointer to the RigidBody is
                // stable, i.e. it's never moved outside the Box<...>.
                // SAFETY: The parent must outlive this rigid body.
                Some(unsafe { &mut **parent })
            }
            _ => None,
        }
    }

    pub fn apply_force(&mut self, force: &Vec3) {
        self.with_rigid_body_mut(|rb| rb.add_force(force.toNA(), true))
    }

    pub fn apply_torque(&mut self, torque: &Vec3) {
        self.with_rigid_body_mut(|rb| rb.add_torque(torque.toNA(), true))
    }

    /// Turns this shape into a compound shape (if needed), and adds the child as part of it.
    ///
    /// This function assumes that `self` is not already a child.
    pub fn attach(&mut self, child: &mut RigidBody, pos: Vec3, rot: Quat) {
        // TODO: Convert this collider into a compound if needed.
        // TODO: Incorporate childs' collider into this collider.
        // TODO: Transition the child into the AddedWithinCompound state.
    }

    /// Removes a shape from this compound shape, and changes it back to a singular shape if
    /// no children are left.
    ///
    /// This function assumes that `self` is not already a child.
    pub fn detach(&mut self, child: &mut RigidBody) {
        // TODO: Transition the child back into the Added state.
        // TODO: Remove the childs collider from this collider.
        // TODO: If no children remaining, turn back into a simple shape.
    }

    /// Calculates the bounding box, and assigns it to `out`.
    pub fn get_bounding_box(&self, out: &mut Box3) {
        let aabb = self.with_collider(|c| c.compute_aabb());
        out.lower = Vec3::fromNAPoint(&aabb.mins);
        out.upper = Vec3::fromNAPoint(&aabb.maxs);
    }

    /// Calculates the compoind bounding box, and assigns it to `out`.
    pub fn get_bounding_box_compound(&self, out: &mut Box3) {
        // TODO: Get the AABB of the compound shape i.e. the root of a compound tree.
        let aabb = self.with_collider(|c| c.compute_aabb());
        out.lower = Vec3::fromNAPoint(&aabb.mins);
        out.upper = Vec3::fromNAPoint(&aabb.maxs);
    }

    /// Calculates the local bounding box, and assigns it to `out`.
    pub fn get_bounding_box_local(&self, out: &mut Box3) {
        let aabb = self.with_collider(|c| c.shape().compute_local_aabb());
        out.lower = Vec3::fromNAPoint(&aabb.mins);
        out.upper = Vec3::fromNAPoint(&aabb.maxs);
    }

    /// Calculates the local compound bounding box, and assigns it to `out`.
    pub fn get_bounding_box_local_compound(&self, out: &mut Box3) {
        // TODO: Get the AABB of the compound shape i.e. the root of a compound tree.
        let aabb = self.with_collider(|c| c.shape().compute_local_aabb());
        out.lower = Vec3::fromNAPoint(&aabb.mins);
        out.upper = Vec3::fromNAPoint(&aabb.maxs);
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

    pub fn get_to_local_matrix(&self) -> Matrix {
        let scale = self.get_scale();
        self.with_rigid_body(|rb| matrix_from_transform(rb.position(), scale).inverted())
    }

    pub fn get_to_world_matrix(&self) -> Matrix {
        let scale = self.get_scale();
        self.with_rigid_body(|rb| matrix_from_transform(rb.position(), scale))
    }

    pub fn get_velocity(&self) -> Vec3 {
        self.with_rigid_body(|rb| Vec3::fromNA(rb.linvel()))
    }

    pub fn get_angular_velocity(&self) -> Vec3 {
        self.with_rigid_body(|rb| Vec3::fromNA(rb.angvel()))
    }

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

    pub fn set_mass(&mut self, mass: f32) {
        self.with_rigid_body_mut(|rb| rb.set_additional_mass(mass, true));
    }

    pub fn get_position(&self) -> Vec3 {
        self.with_rigid_body(|rb| Vec3::fromNA(rb.translation()))
    }

    pub fn get_position_local(&self) -> Vec3 {
        Vec3::ZERO
    }

    pub fn set_position(&mut self, pos: &Vec3) {
        self.with_rigid_body_mut(|rb| rb.set_translation(pos.toNA(), true));
    }

    pub fn set_position_local(&mut self, pos: &Vec3) {}

    pub fn get_rotation(&self) -> Quat {
        self.with_rigid_body(|rb| Quat::fromNA(rb.rotation()))
    }

    pub fn get_rotation_local(&mut self) -> Quat {
        // TODO
        self.with_rigid_body(|rb| Quat::fromNA(rb.rotation()))
    }

    pub fn set_rotation(&mut self, rot: &mut Quat) {
        self.with_rigid_body_mut(|rb| rb.set_rotation(rot.toNA(), true));
    }

    pub fn set_rotation_local(&mut self, rot: &Quat) {}

    pub fn get_scale(&self) -> f32 {
        self.shape_scale
    }

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

    /// Executes a function f with a reference to the RigidBody associated with this object.
    fn with_rigid_body<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&rp::RigidBody) -> R,
    {
        match &self.state {
            RigidBodyState::None => panic!("Uninitialized RigidBody."),
            RigidBodyState::Removed { rb, collider: _ } => f(rb),
            RigidBodyState::Added {
                rb,
                collider: _,
                world,
            } => f(world
                .upgrade()
                .expect("physics world was freed")
                .borrow()
                .rigid_body_set
                .get(*rb)
                .unwrap()),
            RigidBodyState::AddedWithinCompound {
                parent: _,
                collider: _,
                world: _,
            } => panic!("Not supported on children."),
        }
    }

    /// Executes a function f with a mutable reference to the RigidBody associated with this object.
    fn with_rigid_body_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut rp::RigidBody) -> R,
    {
        match &mut self.state {
            RigidBodyState::None => panic!("Uninitialized RigidBody."),
            RigidBodyState::Removed { rb, collider: _ } => f(rb),
            RigidBodyState::Added {
                rb,
                collider: _,
                world,
            } => f(world
                .upgrade()
                .expect("physics world was freed")
                .borrow_mut()
                .rigid_body_set
                .get_mut(*rb)
                .unwrap()),
            RigidBodyState::AddedWithinCompound {
                parent: _,
                collider: _,
                world: _,
            } => panic!("Not supported on children."),
        }
    }

    /// Executes a function f with a reference to the collider associated with this object.
    fn with_collider<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&rp::Collider) -> R,
    {
        match &self.state {
            RigidBodyState::None => panic!("Uninitialized RigidBody."),
            RigidBodyState::Removed { rb: _, collider } => f(collider),
            RigidBodyState::Added {
                rb: _,
                collider,
                world,
            } => f(world
                .upgrade()
                .expect("physics world was freed")
                .borrow()
                .collider_set
                .get(*collider)
                .unwrap()),
            RigidBodyState::AddedWithinCompound {
                parent: _,
                collider,
                world,
            } => f(world
                .upgrade()
                .expect("physics world was freed")
                .borrow()
                .collider_set
                .get(*collider)
                .unwrap()),
        }
    }

    /// Executes a function f with a mutable reference to the collider associated with this object.
    fn with_collider_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut rp::Collider) -> R,
    {
        match &mut self.state {
            RigidBodyState::None => panic!("Uninitialized RigidBody."),
            RigidBodyState::Removed { rb: _, collider } => f(collider),
            RigidBodyState::Added {
                rb: _,
                collider,
                world,
            } => f(world
                .upgrade()
                .expect("physics world was freed")
                .borrow_mut()
                .collider_set
                .get_mut(*collider)
                .unwrap()),
            RigidBodyState::AddedWithinCompound {
                parent: _,
                collider,
                world,
            } => f(world
                .upgrade()
                .expect("physics world was freed")
                .borrow_mut()
                .collider_set
                .get_mut(*collider)
                .unwrap()),
        }
    }
}

fn matrix_from_transform(transform: &rp::Isometry<f32>, scale: f32) -> Matrix {
    let rp_matrix = transform.to_matrix();
    Matrix::from_slice(rp_matrix.as_slice()).scaled(scale)
}

pub fn RigidBody_Create(shape: Box<CollisionShape>) -> Box<RigidBody> {
    RigidBody::new(*shape)
}

#[no_mangle]
pub extern "C" fn RigidBody_CreateBox() -> Box<RigidBody> {
    RigidBody::new_box()
}

#[no_mangle]
pub extern "C" fn RigidBody_CreateBoxFromMesh(mesh: &mut Mesh) -> Box<RigidBody> {
    RigidBody::new_box_from_mesh(mesh)
}

#[no_mangle]
pub extern "C" fn RigidBody_CreateSphere() -> Box<RigidBody> {
    RigidBody::new_sphere()
}

#[no_mangle]
pub extern "C" fn RigidBody_CreateSphereFromMesh(mesh: &mut Mesh) -> Box<RigidBody> {
    RigidBody::new_sphere_from_mesh(mesh)
}

#[no_mangle]
pub extern "C" fn RigidBody_CreateHullFromMesh(mesh: Rc<Mesh>) -> Box<RigidBody> {
    RigidBody::new_hull_from_mesh(mesh)
}

#[no_mangle]
pub extern "C" fn RigidBody_Free(_: Box<RigidBody>) {}

#[no_mangle]
pub extern "C" fn RigidBody_ApplyForce(this: &mut RigidBody, force: &Vec3) {
    this.apply_force(force);
}

#[no_mangle]
pub extern "C" fn RigidBody_ApplyTorque(this: &mut RigidBody, torque: &Vec3) {
    this.apply_torque(torque);
}

#[no_mangle]
pub extern "C" fn RigidBody_Attach(
    parent: &mut RigidBody,
    child: &mut RigidBody,
    pos: &Vec3,
    rot: &Quat,
) {
    parent.attach(child, *pos, *rot);
}

#[no_mangle]
pub extern "C" fn RigidBody_Detach(parent: &mut RigidBody, child: &mut RigidBody) {
    parent.detach(child);
}

#[no_mangle]
pub extern "C" fn RigidBody_GetBoundingBox(this: &mut RigidBody, out: &mut Box3) {
    this.get_bounding_box(out);
}

#[no_mangle]
pub extern "C" fn RigidBody_GetBoundingBoxCompound(this: &mut RigidBody, out: &mut Box3) {
    this.get_bounding_box_compound(out);
}

#[no_mangle]
pub extern "C" fn RigidBody_GetBoundingBoxLocal(this: &mut RigidBody, out: &mut Box3) {
    this.get_bounding_box_local(out);
}

#[no_mangle]
pub extern "C" fn RigidBody_GetBoundingBoxLocalCompound(this: &mut RigidBody, out: &mut Box3) {
    this.get_bounding_box_local_compound(out)
}

#[no_mangle]
pub extern "C" fn RigidBody_GetBoundingRadius(this: &mut RigidBody) -> f32 {
    this.get_bounding_radius()
}

#[no_mangle]
pub extern "C" fn RigidBody_GetBoundingRadiusCompound(this: &mut RigidBody) -> f32 {
    this.get_bounding_radius_compound()
}

#[no_mangle]
pub extern "C" fn RigidBody_GetParentBody(this: &mut RigidBody) -> Option<&mut RigidBody> {
    this.get_parent()
}

#[no_mangle]
pub extern "C" fn RigidBody_GetSpeed(this: &RigidBody) -> f32 {
    this.get_speed()
}

#[no_mangle]
pub extern "C" fn RigidBody_GetToLocalMatrix(this: &mut RigidBody) -> Box<Matrix> {
    Box::new(this.get_to_local_matrix())
}

#[no_mangle]
pub extern "C" fn RigidBody_GetToWorldMatrix(this: &mut RigidBody) -> Box<Matrix> {
    Box::new(this.get_to_world_matrix())
}

#[no_mangle]
pub extern "C" fn RigidBody_GetVelocity(this: &mut RigidBody, out: &mut Vec3) {
    *out = this.get_velocity()
}

#[no_mangle]
pub extern "C" fn RigidBody_GetVelocityA(this: &mut RigidBody, out: &mut Vec3) {
    *out = this.get_angular_velocity()
}

#[no_mangle]
pub extern "C" fn RigidBody_SetCollidable(this: &mut RigidBody, collidable: bool) {
    this.set_collidable(collidable);
}

#[no_mangle]
pub extern "C" fn RigidBody_SetCollisionGroup(this: &mut RigidBody, group: u32) {
    this.set_collision_group(group);
}

#[no_mangle]
pub extern "C" fn RigidBody_SetCollisionMask(this: &mut RigidBody, mask: u32) {
    this.set_collision_mask(mask);
}

#[no_mangle]
pub extern "C" fn RigidBody_SetDrag(this: &mut RigidBody, linear: f32, angular: f32) {
    this.set_drag(linear, angular);
}

#[no_mangle]
pub extern "C" fn RigidBody_SetFriction(this: &mut RigidBody, friction: f32) {
    this.set_friction(friction);
}

#[no_mangle]
pub extern "C" fn RigidBody_SetKinematic(this: &mut RigidBody, kinematic: bool) {
    this.set_kinematic(kinematic);
}

#[no_mangle]
pub extern "C" fn RigidBody_SetRestitution(this: &mut RigidBody, restitution: f32) {
    this.set_restitution(restitution);
}

#[no_mangle]
pub extern "C" fn RigidBody_SetSleepThreshold(this: &mut RigidBody, linear: f32, angular: f32) {
    this.set_sleep_threshold(linear, angular);
}

#[no_mangle]
pub extern "C" fn RigidBody_GetMass(this: &RigidBody) -> f32 {
    this.get_mass()
}

#[no_mangle]
pub extern "C" fn RigidBody_SetMass(this: &mut RigidBody, mass: f32) {
    this.set_mass(mass);
}

#[no_mangle]
pub extern "C" fn RigidBody_GetPos(this: &RigidBody, out: &mut Vec3) {
    *out = this.get_position();
}

#[no_mangle]
pub extern "C" fn RigidBody_GetPosLocal(this: &RigidBody, out: &mut Vec3) {
    *out = this.get_position_local();
}

#[no_mangle]
pub extern "C" fn RigidBody_SetPos(this: &mut RigidBody, pos: &mut Vec3) {
    this.set_position(pos);
}

#[no_mangle]
pub extern "C" fn RigidBody_SetPosLocal(this: &mut RigidBody, pos: &mut Vec3) {
    this.set_position_local(pos);
}

#[no_mangle]
pub extern "C" fn RigidBody_GetRot(this: &RigidBody, out: &mut Quat) {
    *out = this.get_rotation();
}

#[no_mangle]
pub extern "C" fn RigidBody_GetRotLocal(this: &mut RigidBody, out: &mut Quat) {
    *out = this.get_rotation_local();
}

#[no_mangle]
pub extern "C" fn RigidBody_SetRot(this: &mut RigidBody, rot: &mut Quat) {
    this.set_rotation(rot);
}

#[no_mangle]
pub extern "C" fn RigidBody_SetRotLocal(this: &mut RigidBody, rot: &mut Quat) {
    this.set_rotation_local(rot);
}

#[no_mangle]
pub extern "C" fn RigidBody_GetScale(this: &RigidBody) -> f32 {
    this.get_scale()
}

#[no_mangle]
pub extern "C" fn RigidBody_SetScale(this: &mut RigidBody, scale: f32) {
    this.set_scale(scale);
}
