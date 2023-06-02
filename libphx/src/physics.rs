use crate::math::{Sphere, Vec3};
use crate::quat::*;
use crate::ray::*;
use crate::rigid_body::*;
use crate::trigger::*;
use rapier3d::prelude as rp;
use rapier3d::prelude::nalgebra as na;

pub struct Physics {
    pub rigidBodySet: rp::RigidBodySet,
    pub colliderSet: rp::ColliderSet,

    pub integrationParameters: rp::IntegrationParameters,
    pub physicsPipeline: rp::PhysicsPipeline,
    pub queryPipeline: rp::QueryPipeline,
    pub islandManager: rp::IslandManager,
    pub broadphase: rp::BroadPhase,
    pub narrowphase: rp::NarrowPhase,
    pub impulseJointSet: rp::ImpulseJointSet,
    pub multibodyJointSet: rp::MultibodyJointSet,
    pub ccdSolver: rp::CCDSolver,

    pub triggers: Vec<Trigger>,
}

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
    fn toNA(&self) -> na::Vector3<f32>;
    fn toNAPoint(&self) -> na::Point3<f32>;
    fn fromNA(_: &na::Vector3<f32>) -> Self;
    fn fromNAPoint(_: &na::Point3<f32>) -> Self;
}

impl NalgebraVec3Interop for Vec3 {
    fn toNA(&self) -> na::Vector3<f32> {
        na::Vector3::new(self.x, self.y, self.z)
    }
    fn toNAPoint(&self) -> na::Point3<f32> {
        na::Point3::new(self.x, self.y, self.z)
    }
    fn fromNA(v: &na::Vector3<f32>) -> Vec3 {
        Vec3::new(v.x, v.y, v.z)
    }
    fn fromNAPoint(v: &na::Point3<f32>) -> Vec3 {
        Vec3::new(v.x, v.y, v.z)
    }
}

pub trait NalgebraQuatInterop {
    fn toNA(&self) -> na::UnitQuaternion<f32>;
    fn fromNA(_: &na::UnitQuaternion<f32>) -> Self;
}

impl NalgebraQuatInterop for Quat {
    fn toNA(&self) -> na::UnitQuaternion<f32> {
        na::UnitQuaternion::from_quaternion(na::Quaternion::new(self.w, self.x, self.y, self.z))
    }
    fn fromNA(v: &na::UnitQuaternion<f32>) -> Quat {
        Quat_Create(v.i, v.j, v.k, v.w)
    }
}

#[no_mangle]
pub unsafe extern "C" fn Physics_Create() -> Box<Physics> {
    Box::new(Physics {
        integrationParameters: rp::IntegrationParameters::default(),
        rigidBodySet: rp::RigidBodySet::new(),
        colliderSet: rp::ColliderSet::new(),
        physicsPipeline: rp::PhysicsPipeline::new(),
        queryPipeline: rp::QueryPipeline::new(),
        islandManager: rp::IslandManager::new(),
        broadphase: rp::BroadPhase::new(),
        narrowphase: rp::NarrowPhase::new(),
        impulseJointSet: rp::ImpulseJointSet::new(),
        multibodyJointSet: rp::MultibodyJointSet::new(),
        ccdSolver: rp::CCDSolver::new(),
        triggers: Vec::new(),
    })
}

#[no_mangle]
pub unsafe extern "C" fn Physics_Free(_: Box<Physics>) {}

#[no_mangle]
pub unsafe extern "C" fn Physics_AddRigidBody(this: &mut Physics, rb: &mut RigidBody) {
    // this.rigidBodySet.insert(rb)
}

#[no_mangle]
pub unsafe extern "C" fn Physics_RemoveRigidBody(this: &mut Physics, rb: &mut RigidBody) {
    // this.rigidBodySet.remove()
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
    for trigger in this.triggers.iter_mut() {
        Trigger_Update(trigger);
    }

    let gravity = Vec3::ZERO.toNA();
    let physics_hooks = ();
    let event_handler = ();

    let mut integrationParameters = this.integrationParameters;
    integrationParameters.dt = dt;
    this.physicsPipeline.step(
        &gravity,
        &integrationParameters,
        &mut this.islandManager,
        &mut this.broadphase,
        &mut this.narrowphase,
        &mut this.rigidBodySet,
        &mut this.colliderSet,
        &mut this.impulseJointSet,
        &mut this.multibodyJointSet,
        &mut this.ccdSolver,
        None,
        &physics_hooks,
        &event_handler,
    );
    this.queryPipeline
        .update(&this.rigidBodySet, &this.colliderSet);
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
        data.toNAPoint()
    };
    let to = {
        let mut data = Vec3::ZERO;
        Ray_GetPoint(ray, ray.tMax, &mut data);
        data.toNAPoint()
    };
    let dir = to - from;
    let length = dir.norm();

    let ray = rp::Ray::new(from, dir / length);
    let filter = rp::QueryFilter::default();
    if let Some((handle, toi)) = this.queryPipeline.cast_ray(
        &this.rigidBodySet,
        &this.colliderSet,
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
