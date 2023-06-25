use crate::math::*;
use crate::physics::*;
use rapier3d::prelude as rp;
use rapier3d::prelude::nalgebra as na;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Collision {
    index: i32,
    count: i32,
    body0: RigidBody,
    body1: RigidBody,
}

pub struct RayCastResult {
    body: RigidBody,
    norm: Vec3,
    pos: Vec3,
    t: f32,
}

pub struct ShapeCastResult {
    hits: Vec<RigidBody>,
}

pub trait NalgebraVec3Interop {
    fn to_na(&self) -> na::Vector3<f32>;
    fn to_na_point(&self) -> na::Point3<f32>;
    fn from_na(_: &na::Vector3<f32>) -> Self;
    fn from_na_point(_: &na::Point3<f32>) -> Self;
}

impl NalgebraVec3Interop for Vec3 {
    fn to_na(&self) -> na::Vector3<f32> {
        na::Vector3::new(self.x, self.y, self.z)
    }
    fn to_na_point(&self) -> na::Point3<f32> {
        na::Point3::new(self.x, self.y, self.z)
    }
    fn from_na(v: &na::Vector3<f32>) -> Vec3 {
        Vec3::new(v.x, v.y, v.z)
    }
    fn from_na_point(v: &na::Point3<f32>) -> Vec3 {
        Vec3::new(v.x, v.y, v.z)
    }
}

pub trait NalgebraQuatInterop {
    fn to_na(&self) -> na::UnitQuaternion<f32>;
    fn from_na(_: &na::UnitQuaternion<f32>) -> Self;
}

impl NalgebraQuatInterop for Quat {
    fn to_na(&self) -> na::UnitQuaternion<f32> {
        na::UnitQuaternion::from_quaternion(na::Quaternion::new(self.w, self.x, self.y, self.z))
    }
    fn from_na(v: &na::UnitQuaternion<f32>) -> Quat {
        Quat_Create(v.i, v.j, v.k, v.w)
    }
}

pub(crate) struct PhysicsWorld {
    pub(crate) island_manager: rp::IslandManager,
    pub(crate) rigid_body_set: rp::RigidBodySet,
    pub(crate) collider_set: rp::ColliderSet,
}

pub struct Physics {
    world: Rc<RefCell<PhysicsWorld>>,

    integration_parameters: rp::IntegrationParameters,
    physics_pipeline: rp::PhysicsPipeline,
    query_pipeline: rp::QueryPipeline,
    broadphase: rp::BroadPhase,
    narrowphase: rp::NarrowPhase,
    impulse_joint_set: rp::ImpulseJointSet,
    multibody_joint_set: rp::MultibodyJointSet,
    ccd_solver: rp::CCDSolver,

    triggers: Vec<Trigger>,
}

impl Physics {
    pub fn new() -> Physics {
        Physics {
            world: Rc::new(RefCell::new(PhysicsWorld {
                island_manager: rp::IslandManager::new(),
                rigid_body_set: rp::RigidBodySet::new(),
                collider_set: rp::ColliderSet::new(),
            })),
            integration_parameters: rp::IntegrationParameters::default(),
            physics_pipeline: rp::PhysicsPipeline::new(),
            query_pipeline: rp::QueryPipeline::new(),
            broadphase: rp::BroadPhase::new(),
            narrowphase: rp::NarrowPhase::new(),
            impulse_joint_set: rp::ImpulseJointSet::new(),
            multibody_joint_set: rp::MultibodyJointSet::new(),
            ccd_solver: rp::CCDSolver::new(),
            triggers: Vec::new(),
        }
    }

    /// Adds this rigid body to this physics world.
    pub fn add(&mut self, rigid_body: &mut RigidBody) {
        rigid_body.add_to_world(&self.world);
    }

    /// Removes this rigid body from this physics world.
    pub fn remove(&mut self, rigid_body: &mut RigidBody) {
        rigid_body.remove_from_world(&mut self.impulse_joint_set, &mut self.multibody_joint_set);
    }

    pub fn update(&mut self, dt: f32) {
        for trigger in self.triggers.iter_mut() {
            Trigger_Update(trigger);
        }

        let gravity = Vec3::ZERO.to_na();
        let physics_hooks = ();
        let event_handler = ();

        let mut integration_parameters = self.integration_parameters;
        integration_parameters.dt = dt;
        let mut world = &mut *self.world.borrow_mut();
        self.physics_pipeline.step(
            &gravity,
            &integration_parameters,
            &mut world.island_manager,
            &mut self.broadphase,
            &mut self.narrowphase,
            &mut world.rigid_body_set,
            &mut world.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            None,
            &physics_hooks,
            &event_handler,
        );
        self.query_pipeline
            .update(&world.rigid_body_set, &world.collider_set);
    }
}

#[no_mangle]
pub unsafe extern "C" fn Physics_Create() -> Box<Physics> {
    Box::new(Physics::new())
}

#[no_mangle]
pub unsafe extern "C" fn Physics_Free(_: Box<Physics>) {}

#[no_mangle]
pub unsafe extern "C" fn Physics_AddRigidBody(this: &mut Physics, rb: &mut RigidBody) {
    this.add(rb);
}

#[no_mangle]
pub unsafe extern "C" fn Physics_RemoveRigidBody(this: &mut Physics, rb: &mut RigidBody) {
    this.remove(rb);
}

#[no_mangle]
pub unsafe extern "C" fn Physics_AddTrigger(this: &mut Physics, t: *mut Trigger) {}

#[no_mangle]
pub unsafe extern "C" fn Physics_RemoveTrigger(this: &mut Physics, t: *mut Trigger) {}

#[no_mangle]
pub unsafe extern "C" fn Physics_GetNextCollision(this: &mut Physics, c: *mut Collision) -> bool {
    false
}

#[no_mangle]
pub unsafe extern "C" fn Physics_Update(this: &mut Physics, dt: f32) {
    this.update(dt);
}

#[no_mangle]
pub unsafe extern "C" fn Physics_RayCast(
    this: &mut Physics,
    ray: &mut Ray,
    result: &mut RayCastResult,
) {
    let from = {
        let mut data = Vec3::ZERO;
        Ray_GetPoint(ray, ray.tMin, &mut data);
        data.to_na_point()
    };
    let to = {
        let mut data = Vec3::ZERO;
        Ray_GetPoint(ray, ray.tMax, &mut data);
        data.to_na_point()
    };
    let dir = to - from;
    let length = dir.norm();

    let ray = rp::Ray::new(from, dir / length);
    let filter = rp::QueryFilter::default();
    if let Some((handle, toi)) = this.query_pipeline.cast_ray(
        &this.world.borrow().rigid_body_set,
        &this.world.borrow().collider_set,
        &ray,
        length,
        true,
        filter,
    ) {
        // TODO: Fill out RayCastResult data structure.
        // result.body;
    }
}

#[no_mangle]
pub unsafe extern "C" fn Physics_SphereCast(
    this: &mut Physics,
    sphere: *mut Sphere,
    result: *mut ShapeCastResult,
) {
}

#[no_mangle]
pub unsafe extern "C" fn Physics_BoxCast(
    this: &mut Physics,
    pos: *mut Vec3,
    rot: *mut Quat,
    halfExtents: *mut Vec3,
    result: *mut ShapeCastResult,
) {
}

#[no_mangle]
pub unsafe extern "C" fn Physics_SphereOverlap(this: &mut Physics, sphere: *mut Sphere) -> bool {
    false
}

#[no_mangle]
pub unsafe extern "C" fn Physics_BoxOverlap(
    this: &mut Physics,
    pos: *mut Vec3,
    rot: *mut Quat,
    halfExtents: *mut Vec3,
) -> bool {
    false
}

#[no_mangle]
pub unsafe extern "C" fn Physics_PrintProfiling(this: &mut Physics) {}

#[no_mangle]
pub unsafe extern "C" fn Physics_DrawBoundingBoxesLocal(this: &mut Physics) {}

#[no_mangle]
pub unsafe extern "C" fn Physics_DrawBoundingBoxesWorld(this: &mut Physics) {}

#[no_mangle]
pub unsafe extern "C" fn Physics_DrawTriggers(this: &mut Physics) {}

#[no_mangle]
pub unsafe extern "C" fn Physics_DrawWireframes(this: &mut Physics) {}
