use crate::common::*;
use crate::math::*;
use crate::physics::*;
use crate::render::*;
use rapier3d::prelude as rp;
use rapier3d::prelude::nalgebra as na;
use std::cell::RefCell;
use std::mem::replace;
use std::rc::{Rc, Weak};

pub enum PhysicsType {
    Null,
    RigidBody,
    Trigger,
}

#[derive(Clone)]
pub(crate) enum WorldState {
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
        world: Weak<RefCell<PhysicsWorld>>,
    },
    // Added to physics, and attached to a compound shape.
    AttachedToCompound {
        parent: *mut RigidBody, // Raw pointer to stable memory address of parent (as it's in a Box).
        collider_handle: rp::ColliderHandle,
        world: Weak<RefCell<PhysicsWorld>>,
    },
}

// impl WorldState {
//     fn replace_collider(&mut self, new_collider: rp::Collider) {
//         match self {
//             // If the RigidBody is not added to physics, then we can just replace the collider in the enum.
//             WorldState::Removed { collider, .. } => {
//                 *collider = new_collider;
//             }
//             // If the RigidBody is already added to physics, then we need to remove and re-add its collider.
//             WorldState::Added {
//                 rb_handle,
//                 collider_handle,
//                 world,
//             } => {
//                 let world_rc = world.upgrade().expect("physics world was freed");
//                 let w = &mut *world_rc.borrow_mut();

//                 let _ = w.collider_set.remove(
//                     *collider_handle,
//                     &mut w.island_manager,
//                     &mut w.rigid_body_set,
//                     false,
//                 );
//                 *collider_handle = w.collider_set.insert_with_parent(
//                     new_collider,
//                     *rb_handle,
//                     &mut w.rigid_body_set,
//                 );
//             }
//             _ => {}
//         }
//     }
// }


/// Convert an nalgebra Isometry to a Matrix.
fn matrix_from_transform(transform: &rp::Isometry<f32>) -> Matrix {
    Matrix::from_cols_slice(transform.to_matrix().as_slice())
}

pub struct RigidBody {
    ty: PhysicsType,

    // Stores the rigid body / collider state.
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
            let rb_handle = w.rigid_body_set.insert(rb);
            let collider_handle =
                w.collider_set
                    .insert_with_parent(collider, rb_handle, &mut w.rigid_body_set);
            self.state = WorldState::Added {
                rb_handle,
                collider_handle,
                world: Rc::downgrade(world),
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
        } = replace(&mut self.state, WorldState::None)
        {
            let world_rc = world.upgrade().expect("physics world was freed");
            let w = &mut *world_rc.borrow_mut();
            let collider = w
                .collider_set
                .remove(
                    collider_handle,
                    &mut w.island_manager,
                    &mut w.rigid_body_set,
                    false,
                )
                .unwrap();
            let rigid_body = w
                .rigid_body_set
                .remove(
                    rb_handle,
                    &mut w.island_manager,
                    &mut w.collider_set,
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
                .expect("physics world was freed")
                .borrow()
                .rigid_body_set
                .get(*rb_handle)
                .unwrap()),
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
                .expect("physics world was freed")
                .borrow_mut()
                .rigid_body_set
                .get_mut(*rb_handle)
                .unwrap()),
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
                .expect("physics world was freed")
                .borrow()
                .collider_set
                .get(*collider_handle)
                .unwrap()),
            WorldState::AttachedToCompound {
                collider_handle,
                world,
                ..
            } => f(world
                .upgrade()
                .expect("physics world was freed")
                .borrow()
                .collider_set
                .get(*collider_handle)
                .unwrap()),
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
                .expect("physics world was freed")
                .borrow_mut()
                .collider_set
                .get_mut(*collider_handle)
                .unwrap()),
            WorldState::AttachedToCompound {
                collider_handle,
                world,
                ..
            } => f(world
                .upgrade()
                .expect("physics world was freed")
                .borrow_mut()
                .collider_set
                .get_mut(*collider_handle)
                .unwrap()),
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

    /// Is this rigid body part of a compound shape?
    pub fn is_in_compound(&self) -> bool {
        self.is_root_in_compound() || self.is_child()
    }

    /// Is this rigid body a child of the root in a compound shape?
    pub fn is_child(&self) -> bool {
        if let WorldState::AttachedToCompound { .. } = &self.state {
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

    /// Adds another rigid body as a child of this rigid body. This means that the child's position will be controlled by `self`.
    ///
    /// This function assumes that `self` is not already a child.
    pub fn attach(&mut self, child: &mut RigidBody, pos: Vec3, rot: Quat) {
        let parent_ptr = self as *mut _;
        match &mut self.state {
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
                        panic!("Child is already part of a compound.");
                    }
                    WorldState::Removed { mut collider, .. } => {
                        let world_rc = world.upgrade().expect("physics world was freed");
                        let w = &mut *world_rc.borrow_mut();

                        // Position collider and attach it to the parent rigid body.
                        collider.set_position_wrt_parent(na::Isometry3::from_parts(
                            pos.to_na().into(),
                            rot.to_na(),
                        ));
                        WorldState::AttachedToCompound {
                            parent: parent_ptr,
                            collider_handle: w.collider_set.insert_with_parent(
                                collider,
                                *parent_handle,
                                &mut w.rigid_body_set,
                            ),
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
        // TODO: Transition the child back into the Added state.
        // TODO: Remove the childs collider from this collider.
        // TODO: If no children remaining, turn back into a simple shape.
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
        // TODO: Get the AABB of the compound shape i.e. the root of a compound tree.
        let aabb = self.with_collider(|c| c.shape().compute_local_aabb());
        out.lower = Vec3::from_na_point(&aabb.mins);
        out.upper = Vec3::from_na_point(&aabb.maxs);
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

    /// Returns the unscaled world matrix of this rigid body.
    fn get_world_matrix_unscaled(&self) -> Matrix {
        if let WorldState::AttachedToCompound { parent, .. } = &self.state {
            let transform =
                self.with_collider(|c| matrix_from_transform(c.position_wrt_parent().unwrap()));
            let parent_transform =
                unsafe { &**parent }.with_rigid_body(|rb| matrix_from_transform(rb.position()));
            // transform.mul_mat4(&parent_transform)
            parent_transform
        } else {
            self.with_rigid_body(|rb| matrix_from_transform(rb.position()))
        }
    }

    /// Returns the local -> world matrix for this rigid body.
    pub fn get_to_world_matrix(&self) -> Matrix {
        self.get_world_matrix_unscaled() * Matrix::from_scale(Vec3::splat(self.get_scale()))
    }

    /// Returns the world -> local matrix for this rigid body.
    pub fn get_to_local_matrix(&self) -> Matrix {
        self.get_to_world_matrix().inverse()
    }

    pub fn get_velocity(&self) -> Vec3 {
        self.with_rigid_body(|rb| Vec3::from_na(rb.linvel()))
    }

    pub fn get_angular_velocity(&self) -> Vec3 {
        self.with_rigid_body(|rb| Vec3::from_na(rb.angvel()))
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

    #[bind(name = "GetPos", out_param = true)]
    pub fn get_position(&self) -> Vec3 {
        self.get_world_matrix_unscaled().get_translation()
    }

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

// pub fn RigidBody_Create(shape: Box<CollisionShape>) -> Box<RigidBody> {
//     RigidBody::new(*shape)
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_CreateBox() -> Box<RigidBody> {
//     RigidBody::new_box()
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_CreateBoxFromMesh(mesh: &mut Mesh) -> Box<RigidBody> {
//     RigidBody::new_box_from_mesh(mesh)
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_CreateSphere() -> Box<RigidBody> {
//     RigidBody::new_sphere()
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_CreateSphereFromMesh(mesh: &mut Mesh) -> Box<RigidBody> {
//     RigidBody::new_sphere_from_mesh(mesh)
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_CreateHullFromMesh(mesh: Rc<Mesh>) -> Box<RigidBody> {
//     RigidBody::new_hull_from_mesh(mesh)
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_Free(_: Box<RigidBody>) {}

// #[no_mangle]
// pub extern "C" fn RigidBody_ApplyForce(this: &mut RigidBody, force: &Vec3) {
//     this.apply_force(force);
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_ApplyTorque(this: &mut RigidBody, torque: &Vec3) {
//     this.apply_torque(torque);
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_Attach(
//     parent: &mut RigidBody,
//     child: &mut RigidBody,
//     pos: &Vec3,
//     rot: &Quat,
// ) {
//     parent.attach(child, *pos, *rot);
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_Detach(parent: &mut RigidBody, child: &mut RigidBody) {
//     parent.detach(child);
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_GetBoundingBox(this: &mut RigidBody, out: &mut Box3) {
//     this.get_bounding_box(out);
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_GetBoundingBoxCompound(this: &mut RigidBody, out: &mut Box3) {
//     this.get_bounding_box_compound(out);
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_GetBoundingBoxLocal(this: &mut RigidBody, out: &mut Box3) {
//     this.get_bounding_box_local(out);
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_GetBoundingBoxLocalCompound(this: &mut RigidBody, out: &mut Box3) {
//     this.get_bounding_box_local_compound(out)
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_GetBoundingRadius(this: &mut RigidBody) -> f32 {
//     this.get_bounding_radius()
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_GetBoundingRadiusCompound(this: &mut RigidBody) -> f32 {
//     this.get_bounding_radius_compound()
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_GetParentBody(this: &mut RigidBody) -> Option<&mut RigidBody> {
//     this.get_parent()
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_GetSpeed(this: &RigidBody) -> f32 {
//     this.get_speed()
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_GetToLocalMatrix(this: &mut RigidBody) -> Box<Matrix> {
//     Box::new(this.get_to_local_matrix())
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_GetToWorldMatrix(this: &mut RigidBody) -> Box<Matrix> {
//     Box::new(this.get_to_world_matrix())
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_GetVelocity(this: &mut RigidBody, out: &mut Vec3) {
//     *out = this.get_velocity()
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_GetVelocityA(this: &mut RigidBody, out: &mut Vec3) {
//     *out = this.get_angular_velocity()
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_SetCollidable(this: &mut RigidBody, collidable: bool) {
//     this.set_collidable(collidable);
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_SetCollisionGroup(this: &mut RigidBody, group: u32) {
//     this.set_collision_group(group);
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_SetCollisionMask(this: &mut RigidBody, mask: u32) {
//     this.set_collision_mask(mask);
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_SetDrag(this: &mut RigidBody, linear: f32, angular: f32) {
//     this.set_drag(linear, angular);
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_SetFriction(this: &mut RigidBody, friction: f32) {
//     this.set_friction(friction);
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_SetKinematic(this: &mut RigidBody, kinematic: bool) {
//     this.set_kinematic(kinematic);
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_SetRestitution(this: &mut RigidBody, restitution: f32) {
//     this.set_restitution(restitution);
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_SetSleepThreshold(this: &mut RigidBody, linear: f32, angular: f32) {
//     this.set_sleep_threshold(linear, angular);
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_GetMass(this: &RigidBody) -> f32 {
//     this.get_mass()
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_SetMass(this: &mut RigidBody, mass: f32) {
//     this.set_mass(mass);
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_GetPos(this: &RigidBody, out: &mut Vec3) {
//     *out = this.get_position();
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_GetPosLocal(this: &RigidBody, out: &mut Vec3) {
//     *out = this.get_position_local();
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_SetPos(this: &mut RigidBody, pos: &mut Vec3) {
//     this.set_position(pos);
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_SetPosLocal(this: &mut RigidBody, pos: &mut Vec3) {
//     this.set_position_local(pos);
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_GetRot(this: &RigidBody, out: &mut Quat) {
//     *out = this.get_rotation();
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_GetRotLocal(this: &mut RigidBody, out: &mut Quat) {
//     *out = this.get_rotation_local();
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_SetRot(this: &mut RigidBody, rot: &mut Quat) {
//     this.set_rotation(rot);
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_SetRotLocal(this: &mut RigidBody, rot: &mut Quat) {
//     this.set_rotation_local(rot);
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_GetScale(this: &RigidBody) -> f32 {
//     this.get_scale()
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_SetScale(this: &mut RigidBody, scale: f32) {
//     this.set_scale(scale);
// }

// #[no_mangle]
// pub extern "C" fn RigidBody_Free(_: Box<RigidBody>) {}
// #[no_mangle]
// pub extern "C" fn RigidBody_Create(shape: Box<CollisionShape>) -> Box<Box> {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_Create")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     let __res__ = RigidBody::new(*shape);
//     __res__.into()
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_CreateBox() -> Box<Box> {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_CreateBox")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     let __res__ = RigidBody::new_box();
//     __res__.into()
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_CreateBoxFromMesh(mesh: &mut Mesh) -> Box<Box> {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_CreateBoxFromMesh")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     let __res__ = RigidBody::new_box_from_mesh(mesh);
//     __res__.into()
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_CreateSphere() -> Box<Box> {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_CreateSphere")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     let __res__ = RigidBody::new_sphere();
//     __res__.into()
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_CreateSphereFromMesh(mesh: &mut Mesh) -> Box<Box> {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!(
//                                         "Calling: {0}",
//                                         "RigidBody_CreateSphereFromMesh",
//                                     ) as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     let __res__ = RigidBody::new_sphere_from_mesh(mesh);
//     __res__.into()
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_CreateHullFromMesh(mesh: Box<Rc>) -> Box<Box> {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!(
//                                         "Calling: {0}",
//                                         "RigidBody_CreateHullFromMesh",
//                                     ) as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     let __res__ = RigidBody::new_hull_from_mesh(*mesh);
//     __res__.into()
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_IsInCompound(this: &RigidBody) -> bool {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_IsInCompound")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     let __res__ = this.is_in_compound();
//     __res__
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_IsChild(this: &RigidBody) -> bool {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_IsChild")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     let __res__ = this.is_child();
//     __res__
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_IsRootInCompound(this: &RigidBody) -> bool {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_IsRootInCompound")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     let __res__ = this.is_root_in_compound();
//     __res__
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_GetParent(this: &RigidBody) -> *mut RigidBody {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_GetParent")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     let __res__ = this.get_parent();
//     let Some(__res__) = __res__ else {
//         return std::ptr::null_mut();
//     };
//     __res__ as *mut RigidBody
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_ApplyForce(this: &mut RigidBody, force: Vec3) {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_ApplyForce")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     this.apply_force(&force);
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_ApplyTorque(this: &mut RigidBody, torque: Vec3) {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_ApplyTorque")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     this.apply_torque(&torque);
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_Attach(
//     this: &mut RigidBody,
//     child: &mut RigidBody,
//     pos: Vec3,
//     rot: Box<Quat>,
// ) {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_Attach")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     this.attach(child, pos, *rot);
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_Detach(this: &mut RigidBody, child: &mut RigidBody) {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_Detach")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     this.detach(child);
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_GetBoundingBox(this: &RigidBody, out: &mut Box3) {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_GetBoundingBox")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     this.get_bounding_box(out);
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_GetBoundingBoxCompound(
//     this: &RigidBody,
//     out: &mut Box3,
// ) {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!(
//                                         "Calling: {0}",
//                                         "RigidBody_GetBoundingBoxCompound",
//                                     ) as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     this.get_bounding_box_compound(out);
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_GetBoundingBoxLocal(this: &RigidBody, out: &mut Box3) {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!(
//                                         "Calling: {0}",
//                                         "RigidBody_GetBoundingBoxLocal",
//                                     ) as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     this.get_bounding_box_local(out);
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_GetBoundingBoxLocalCompound(
//     this: &RigidBody,
//     out: &mut Box3,
// ) {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!(
//                                         "Calling: {0}",
//                                         "RigidBody_GetBoundingBoxLocalCompound",
//                                     ) as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     this.get_bounding_box_local_compound(out);
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_GetBoundingRadius(this: &RigidBody) -> f32 {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_GetBoundingRadius")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     let __res__ = this.get_bounding_radius();
//     __res__
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_GetBoundingRadiusCompound(this: &RigidBody) -> f32 {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!(
//                                         "Calling: {0}",
//                                         "RigidBody_GetBoundingRadiusCompound",
//                                     ) as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     let __res__ = this.get_bounding_radius_compound();
//     __res__
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_GetSpeed(this: &RigidBody) -> f32 {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_GetSpeed")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     let __res__ = this.get_speed();
//     __res__
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_GetWorldMatrixUnscaled(this: &RigidBody) -> Box<Matrix> {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!(
//                                         "Calling: {0}",
//                                         "RigidBody_GetWorldMatrixUnscaled",
//                                     ) as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     let __res__ = this.get_world_matrix_unscaled();
//     __res__.into()
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_GetToWorldMatrix(this: &RigidBody) -> Box<Matrix> {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_GetToWorldMatrix")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     let __res__ = this.get_to_world_matrix();
//     __res__.into()
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_GetToLocalMatrix(this: &RigidBody) -> Box<Matrix> {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_GetToLocalMatrix")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     let __res__ = this.get_to_local_matrix();
//     __res__.into()
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_GetVelocity(this: &RigidBody) -> Vec3 {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_GetVelocity")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     let __res__ = this.get_velocity();
//     __res__
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_GetAngularVelocity(this: &RigidBody) -> Vec3 {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!(
//                                         "Calling: {0}",
//                                         "RigidBody_GetAngularVelocity",
//                                     ) as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     let __res__ = this.get_angular_velocity();
//     __res__
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_SetCollidable(this: &mut RigidBody, collidable: bool) {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_SetCollidable")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     this.set_collidable(collidable);
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_SetCollisionGroup(this: &mut RigidBody, group: u32) {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_SetCollisionGroup")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     this.set_collision_group(group);
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_SetCollisionMask(this: &mut RigidBody, mask: u32) {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_SetCollisionMask")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     this.set_collision_mask(mask);
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_SetDrag(
//     this: &mut RigidBody,
//     linear: f32,
//     angular: f32,
// ) {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_SetDrag")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     this.set_drag(linear, angular);
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_SetFriction(this: &mut RigidBody, friction: f32) {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_SetFriction")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     this.set_friction(friction);
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_SetKinematic(this: &mut RigidBody, kinematic: bool) {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_SetKinematic")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     this.set_kinematic(kinematic);
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_SetRestitution(this: &mut RigidBody, restitution: f32) {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_SetRestitution")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     this.set_restitution(restitution);
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_SetSleepThreshold(
//     this: &mut RigidBody,
//     linear: f32,
//     angular: f32,
// ) {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_SetSleepThreshold")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     this.set_sleep_threshold(linear, angular);
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_GetMass(this: &RigidBody) -> f32 {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_GetMass")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     let __res__ = this.get_mass();
//     __res__
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_SetMass(this: &mut RigidBody, mass: f32) {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_SetMass")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     this.set_mass(mass);
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_GetPos(this: &RigidBody, out: &mut Vec3) {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_GetPos")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     let __res__ = this.get_position();
//     *out = __res__;
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_GetPosLocal(this: &RigidBody, out: &mut Vec3) {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_GetPosLocal")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     let __res__ = this.get_position_local();
//     *out = __res__;
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_SetPos(this: &mut RigidBody, pos: Vec3) {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_SetPos")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     this.set_position(&pos);
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_SetPosLocal(this: &mut RigidBody, pos: Vec3) {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_SetPosLocal")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     this.set_position_local(&pos);
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_GetRot(this: &RigidBody, out: &mut Quat) {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_GetRot")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     let __res__ = this.get_rotation();
//     *out = __res__;
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_GetRotLocal(this: &mut RigidBody, out: &mut Quat) {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_GetRotLocal")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     let __res__ = this.get_rotation_local();
//     *out = __res__;
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_SetRot(this: &mut RigidBody, rot: &mut Quat) {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_SetRot")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     this.set_rotation(rot);
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_SetRotLocal(this: &mut RigidBody, rot: &Quat) {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_SetRotLocal")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     this.set_rotation_local(rot);
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_GetScale(this: &RigidBody) -> f32 {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_GetScale")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     let __res__ = this.get_scale();
//     __res__
// }
// #[no_mangle]
// pub extern "C" fn RigidBody_SetScale(this: &mut RigidBody, scale: f32) {
//     {
//         use ::tracing::__macro_support::Callsite as _;
//         static CALLSITE: ::tracing::callsite::DefaultCallsite = {
//             static META: ::tracing::Metadata<'static> = {
//                 ::tracing_core::metadata::Metadata::new(
//                     "event engine/lib/phx/src/physics/rigid_body.rs:271",
//                     "phx::physics::rigid_body",
//                     ::tracing::Level::TRACE,
//                     Some("engine/lib/phx/src/physics/rigid_body.rs"),
//                     Some(271u32),
//                     Some("phx::physics::rigid_body"),
//                     ::tracing_core::field::FieldSet::new(
//                         &["message"],
//                         ::tracing_core::callsite::Identifier(&CALLSITE),
//                     ),
//                     ::tracing::metadata::Kind::EVENT,
//                 )
//             };
//             ::tracing::callsite::DefaultCallsite::new(&META)
//         };
//         let enabled = ::tracing::Level::TRACE
//             <= ::tracing::level_filters::STATIC_MAX_LEVEL
//             && ::tracing::Level::TRACE
//                 <= ::tracing::level_filters::LevelFilter::current()
//             && {
//                 let interest = CALLSITE.interest();
//                 !interest.is_never()
//                     && ::tracing::__macro_support::__is_enabled(
//                         CALLSITE.metadata(),
//                         interest,
//                     )
//             };
//         if enabled {
//             (|value_set: ::tracing::field::ValueSet| {
//                 let meta = CALLSITE.metadata();
//                 ::tracing::Event::dispatch(meta, &value_set);
//             })({
//                 #[allow(unused_imports)]
//                 use ::tracing::field::{debug, display, Value};
//                 let mut iter = CALLSITE.metadata().fields().iter();
//                 CALLSITE
//                     .metadata()
//                     .fields()
//                     .value_set(
//                         &[
//                             (
//                                 &iter.next().expect("FieldSet corrupted (this is a bug)"),
//                                 Some(
//                                     &format_args!("Calling: {0}", "RigidBody_SetScale")
//                                         as &dyn Value,
//                                 ),
//                             ),
//                         ],
//                     )
//             });
//         } else {
//         }
//     };
//     this.set_scale(scale);
// }