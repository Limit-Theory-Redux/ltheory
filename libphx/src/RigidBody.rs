use crate::CollisionShape::*;
use crate::Common::*;
use crate::Math::{Box3, Vec3};
use crate::Matrix::Matrix;
use crate::Mesh::*;
use crate::Physics::*;
use crate::Quat::Quat;
use rapier3d::prelude as rp;
use rapier3d::prelude::nalgebra as na;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

pub enum PhysicsType {
    Null,
    RigidBody,
    Trigger,
}

enum RigidBodyState {
    Detached {
        rb: rp::RigidBody,
        collider: rp::Collider,
    },
    Attached {
        rb: rp::RigidBodyHandle,
        collider: rp::ColliderHandle,
        world: Rc<RefCell<Physics>>,
    },
}

/*
struct RigidBody {
  PhysicsType  type;
  btRigidBody* handle;         // Always references *this* object, even when part of a compound
  int          iShape;         // Always references *this* object, even when part of a compound
  int          collisionGroup; // Which group this object is part of
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
    handle: RigidBodyState,

    // Fields to allow us to reconstruct the collision shape object.
    shapeType: CollisionShapeType,
    shapeScale: f32,

    collidable: bool,
    collisionGroup: rp::InteractionGroups,
    mass: f32,
}

impl RigidBody {
    pub fn new(shape: CollisionShape) -> RigidBody {
        let rigidBody = rp::RigidBodyBuilder::dynamic().build();
        RigidBody {
            ty: PhysicsType::RigidBody,
            handle: RigidBodyState::Detached {
                rb: rigidBody,
                collider: shape.collider,
            },
            shapeType: shape.shape,
            shapeScale: shape.scale,
            collidable: true,
            collisionGroup: rp::InteractionGroups::default(),
            mass: 1.0,
        }
    }

    pub fn isChild(&self) -> bool {
        false
    }

    /// Replaces the collider of this rigid body in the physics world it's contained within.
    // pub fn replaceCollider(&mut self, collider: rp::Collider) {
    //     self.handle = match self.handle {
    //         RigidBodyState::Detached { rb, collider: _ } => {
    //             RigidBodyState::Detached { rb, collider }
    //         }
    //         RigidBodyState::Attached {
    //             rb,
    //             collider: currentColliderHandle,
    //             world,
    //         } => {
    //             let newHandle = {
    //                 let mut worldMut = world.borrow_mut();
    //                 let mut colliderSet = &mut worldMut.colliderSet;
    //                 let mut islandManager = &mut worldMut.islandManager;
    //                 let mut rigidBodySet = &mut worldMut.rigidBodySet;
    //                 colliderSet.remove(
    //                     currentColliderHandle,
    //                     islandManager, 
    //                     rigidBodySet,
    //                     true,
    //                 );
    //                 colliderSet.insert_with_parent(
    //                     collider,
    //                     rb,
    //                     rigidBodySet,
    //                 )
    //             };
    //             RigidBodyState::Attached {
    //                 rb,
    //                 collider: newHandle,
    //                 world,
    //             }
    //         }
    //     }
    // }

    /// Executes a function f with a reference to the RigidBody associated with this object.
    pub fn withRigidBody<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&rp::RigidBody) -> R,
    {
        match &self.handle {
            RigidBodyState::Detached { rb, collider: _ } => f(rb),
            RigidBodyState::Attached {
                rb,
                collider: _,
                world,
            } => f(world.borrow().rigidBodySet.get(*rb).unwrap()),
        }
    }

    /// Executes a function f with a mutable reference to the RigidBody associated with this object.
    pub fn withRigidBodyMut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut rp::RigidBody) -> R,
    {
        match &mut self.handle {
            RigidBodyState::Detached { rb, collider: _ } => f(rb),
            RigidBodyState::Attached {
                rb,
                collider: _,
                world,
            } => f(world.borrow_mut().rigidBodySet.get_mut(*rb).unwrap()),
        }
    }

    /// Executes a function f with a reference to the collider associated with this object.
    pub fn withCollider<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&rp::Collider) -> R,
    {
        match &self.handle {
            RigidBodyState::Detached { rb: _, collider } => f(collider),
            RigidBodyState::Attached {
                rb: _,
                collider,
                world,
            } => f(world.borrow().colliderSet.get(*collider).unwrap()),
        }
    }

    /// Executes a function f with a mutable reference to the collider associated with this object.
    pub fn withColliderMut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut rp::Collider) -> R,
    {
        match &mut self.handle {
            RigidBodyState::Detached { rb: _, collider } => f(collider),
            RigidBodyState::Attached {
                rb: _,
                collider,
                world,
            } => f(world.borrow_mut().colliderSet.get_mut(*collider).unwrap()),
        }
    }
}

pub fn RigidBody_Create(shape: Box<CollisionShape>) -> Box<RigidBody> {
    Box::new(RigidBody::new(*shape))
}

#[no_mangle]
pub extern "C" fn RigidBody_CreateBox() -> Box<RigidBody> {
    RigidBody_Create(CollisionShape::newBox(&Vec3::ONE))
}

#[no_mangle]
pub extern "C" fn RigidBody_CreateBoxFromMesh(mesh: &mut Mesh) -> Box<RigidBody> {
    RigidBody_Create(CollisionShape::newBoxFromMesh(mesh))
}

#[no_mangle]
pub extern "C" fn RigidBody_CreateSphere() -> Box<RigidBody> {
    RigidBody_Create(CollisionShape::newSphere(1.0))
}

#[no_mangle]
pub extern "C" fn RigidBody_CreateSphereFromMesh(mesh: &mut Mesh) -> Box<RigidBody> {
    RigidBody_Create(CollisionShape::newSphereFromMesh(mesh))
}

#[no_mangle]
pub extern "C" fn RigidBody_CreateHullFromMesh(mesh: Rc<Mesh>) -> Box<RigidBody> {
    RigidBody_Create(CollisionShape::newHullFromMesh(mesh))
}

#[no_mangle]
pub extern "C" fn RigidBody_Free(_: Box<RigidBody>) {}

#[no_mangle]
pub extern "C" fn RigidBody_ApplyForce(this: &mut RigidBody, force: &Vec3) {
    if this.isChild() {
        Fatal!("RigidBody_ApplyForce: Not supported on children.");
    }
    this.withRigidBodyMut(|rb| rb.add_force(force.toNA(), true))
}

#[no_mangle]
pub extern "C" fn RigidBody_ApplyTorque(this: &mut RigidBody, torque: &Vec3) {
    if this.isChild() {
        Fatal!("RigidBody_ApplyTorque: Not supported on children.");
    }
    this.withRigidBodyMut(|rb| rb.add_torque(torque.toNA(), true))
}

#[no_mangle]
pub extern "C" fn RigidBody_Attach(
    this: &mut RigidBody,
    child: &mut RigidBody,
    offset: &Vec3,
    rot: &Quat,
) {
}

#[no_mangle]
pub extern "C" fn RigidBody_Detach(this: &mut RigidBody, other: *mut RigidBody) {}

#[no_mangle]
pub extern "C" fn RigidBody_GetBoundingBox(this: &mut RigidBody, out: &mut Box3) {
    let aabb = this.withCollider(|c| c.compute_aabb());
    out.lower = Vec3::fromNAPoint(&aabb.mins);
    out.upper = Vec3::fromNAPoint(&aabb.maxs);
}

#[no_mangle]
pub extern "C" fn RigidBody_GetBoundingBoxCompound(this: &mut RigidBody, out: &mut Box3) {
    // TODO: Get the AABB of the compound shape i.e. the root of a compound tree.
    let aabb = this.withCollider(|c| c.compute_aabb());
    out.lower = Vec3::fromNAPoint(&aabb.mins);
    out.upper = Vec3::fromNAPoint(&aabb.maxs);
}

#[no_mangle]
pub extern "C" fn RigidBody_GetBoundingBoxLocal(this: &mut RigidBody, out: &mut Box3) {
    let aabb = this.withCollider(|c| c.shape().compute_local_aabb());
    out.lower = Vec3::fromNAPoint(&aabb.mins);
    out.upper = Vec3::fromNAPoint(&aabb.maxs);
}

#[no_mangle]
pub extern "C" fn RigidBody_GetBoundingBoxLocalCompound(this: &mut RigidBody, out: &mut Box3) {
    // TODO: Get the AABB of the compound shape i.e. the root of a compound tree.
    let aabb = this.withCollider(|c| c.shape().compute_local_aabb());
    out.lower = Vec3::fromNAPoint(&aabb.mins);
    out.upper = Vec3::fromNAPoint(&aabb.maxs);
}

#[no_mangle]
pub extern "C" fn RigidBody_GetBoundingRadius(this: &mut RigidBody) -> f32 {
    this.withCollider(|c| c.shape().compute_local_bounding_sphere().radius)
}

#[no_mangle]
pub extern "C" fn RigidBody_GetBoundingRadiusCompound(this: &mut RigidBody) -> f32 {
    // TODO: Get the AABB of the compound shape i.e. the root of a compound tree.
    this.withCollider(|c| c.shape().compute_local_bounding_sphere().radius)
}

#[no_mangle]
pub extern "C" fn RigidBody_GetParentBody(this: &mut RigidBody) -> Option<&mut RigidBody> {
    None
}

#[no_mangle]
pub extern "C" fn RigidBody_GetSpeed(this: &RigidBody) -> f32 {
    if this.isChild() {
        Fatal!("RigidBody_GetSpeed: Not supported on children.");
    }
    this.withRigidBody(|rb| rb.linvel().norm())
}

#[no_mangle]
pub extern "C" fn RigidBody_GetToLocalMatrix(this: &mut RigidBody) -> *mut Matrix {
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn RigidBody_GetToWorldMatrix(this: &mut RigidBody) -> *mut Matrix {
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn RigidBody_GetVelocity(this: &mut RigidBody, out: &mut Vec3) {
    if this.isChild() {
        Fatal!("RigidBody_GetVelocity: Not supported on children.");
    }
    this.withRigidBody(|rb| *out = Vec3::fromNA(rb.linvel()))
}

#[no_mangle]
pub extern "C" fn RigidBody_GetVelocityA(this: &mut RigidBody, out: &mut Vec3) {
    if this.isChild() {
        Fatal!("RigidBody_GetVelocityA: Not supported on children.");
    }
    this.withRigidBody(|rb| *out = Vec3::fromNA(rb.angvel()))
}

#[no_mangle]
pub extern "C" fn RigidBody_SetCollidable(this: &mut RigidBody, collidable: bool) {
    this.collidable = collidable;
    let collisionGroup = if this.collidable {
        this.collisionGroup
    } else {
        rp::InteractionGroups::none()
    };
    this.withColliderMut(|c| c.set_collision_groups(collisionGroup));
}

#[no_mangle]
pub extern "C" fn RigidBody_SetCollisionGroup(this: &mut RigidBody, group: u32) {
    this.collisionGroup.memberships = group.into();
    let collisionGroup = if this.collidable {
        this.collisionGroup
    } else {
        rp::InteractionGroups::none()
    };
    this.withColliderMut(|c| c.set_collision_groups(collisionGroup));
}

#[no_mangle]
pub extern "C" fn RigidBody_SetCollisionMask(this: &mut RigidBody, mask: u32) {
    this.collisionGroup.filter = mask.into();
    let collisionGroup = if this.collidable {
        this.collisionGroup
    } else {
        rp::InteractionGroups::none()
    };
    this.withColliderMut(|c| c.set_collision_groups(collisionGroup));
}

#[no_mangle]
pub extern "C" fn RigidBody_SetDrag(this: &mut RigidBody, linear: f32, angular: f32) {
    this.withRigidBodyMut(|rb| {
        rb.set_linear_damping(linear);
        rb.set_angular_damping(angular);
    });
}

#[no_mangle]
pub extern "C" fn RigidBody_SetFriction(this: &mut RigidBody, friction: f32) {
    this.withColliderMut(|c| c.set_friction(friction));
}

#[no_mangle]
pub extern "C" fn RigidBody_SetKinematic(this: &mut RigidBody, kinematic: bool) {
    this.withRigidBodyMut(|rb| {
        if kinematic {
            rb.set_body_type(rp::RigidBodyType::KinematicPositionBased, true);
        } else {
            rb.set_body_type(rp::RigidBodyType::Dynamic, true);
        }
    });
}

#[no_mangle]
pub extern "C" fn RigidBody_SetRestitution(this: &mut RigidBody, restitution: f32) {
    this.withColliderMut(|c| c.set_restitution(restitution));
}

#[no_mangle]
pub extern "C" fn RigidBody_SetSleepThreshold(this: &mut RigidBody, linear: f32, angular: f32) {
    this.withRigidBodyMut(|rb| {
        rb.activation_mut().linear_threshold = linear;
        rb.activation_mut().angular_threshold = angular;
    });
}

#[no_mangle]
pub extern "C" fn RigidBody_GetMass(this: &RigidBody) -> f32 {
    this.withRigidBody(|rb| rb.mass())
}

#[no_mangle]
pub extern "C" fn RigidBody_SetMass(this: &mut RigidBody, mass: f32) {
    this.withRigidBodyMut(|rb| rb.set_additional_mass(mass, true));
}

#[no_mangle]
pub extern "C" fn RigidBody_GetPos(this: &RigidBody, out: &mut Vec3) {
    this.withRigidBody(|rb| *out = Vec3::fromNA(rb.translation()));
}

#[no_mangle]
pub extern "C" fn RigidBody_GetPosLocal(this: &RigidBody, out: &mut Vec3) {}

#[no_mangle]
pub extern "C" fn RigidBody_SetPos(this: &mut RigidBody, pos: &mut Vec3) {
    this.withRigidBodyMut(|rb| rb.set_translation(pos.toNA(), true));
}

#[no_mangle]
pub extern "C" fn RigidBody_SetPosLocal(this: &mut RigidBody, pos: &mut Vec3) {}

#[no_mangle]
pub extern "C" fn RigidBody_GetRot(this: &RigidBody, out: &mut Quat) {
    this.withRigidBody(|rb| *out = Quat::fromNA(rb.rotation()));
}

#[no_mangle]
pub extern "C" fn RigidBody_GetRotLocal(this: &mut RigidBody, out: &mut Quat) {}

#[no_mangle]
pub extern "C" fn RigidBody_SetRot(this: &mut RigidBody, rot: &mut Quat) {
    this.withRigidBodyMut(|rb| rb.set_rotation(rot.toNA(), true));
}

#[no_mangle]
pub extern "C" fn RigidBody_SetRotLocal(this: &mut RigidBody, rot: &mut Quat) {}

#[no_mangle]
pub extern "C" fn RigidBody_GetScale(this: &RigidBody) -> f32 {
    this.shapeScale
}

#[no_mangle]
pub extern "C" fn RigidBody_SetScale(this: &mut RigidBody, scale: f32) {
    let scaledShape = CollisionShape::new(scale, this.shapeType.clone());
    this.withColliderMut(|c| {
        // Replace the shape of the current collider by cloning the reference counted Shape object.
        c.set_shape(rp::SharedShape(
            scaledShape.collider.shared_shape().0.clone(),
        ));
    });
    this.shapeType = scaledShape.shape;
    this.shapeScale = scale;
}
