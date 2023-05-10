use crate::Math::Vec3;
use crate::Ray::*;
use crate::Quat::*;
use crate::Trigger::*;
use rapier3d::prelude as rp;
use rapier3d::prelude::nalgebra;

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
    pub ccdSolve: rp::CCDSolver,

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
    hits: Vec<RigidBody>
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
        ccdSolve: rp::CCDSolver::new(),
        triggers: Vec::new(),
    })
}

#[no_mangle]
pub unsafe extern "C" fn Physics_Free(_: Box<Physics>) {
}

#[no_mangle]
pub unsafe extern "C" fn Physics_AddRigidBody(this: &mut Physics, rb: *mut RigidBody) {
}

#[no_mangle]
pub unsafe extern "C" fn Physics_RemoveRigidBody(this: &mut Physics, rb: *mut RigidBody) {
}

#[no_mangle]
pub unsafe extern "C" fn Physics_AddTrigger(this: &mut Physics, t: *mut Trigger) {
}

#[no_mangle]
pub unsafe extern "C" fn Physics_RemoveTrigger(this: &mut Physics, t: *mut Trigger) {
}

#[no_mangle]
pub unsafe extern "C" fn Physics_GetNextCollision(this: &mut Physics, c: *mut Collision) -> bool {
    false
}

#[no_mangle]
pub unsafe extern "C" fn Physics_Update(this: &mut Physics, dt: f32) {
    for trigger in this.triggers.iter() {
        Trigger_Update(trigger);
    }

    let gravity = rp::vector![0.0, 0.0, 0.0];
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
    this.queryPipeline.update(&this.rigidBodySet, &this.colliderSet);
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
        rp::Point::new(data.x, data.y, data.z)
    };
    let to = {
        let mut data = Vec3::ZERO;
        Ray_GetPoint(ray, ray.tMax, &mut data);
        rp::Point::new(data.x, data.y, data.z)
    };
    let dir = to - from;
    let length = dir.norm();

    let ray = rp::Ray::new(from, dir / length);
    let filter = rp::QueryFilter::default();
    if let Some((handle, toi)) = this.queryPipeline.cast_ray(&this.rigidBodySet, &this.colliderSet, &ray, length, true, filter) {
        
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
pub unsafe extern "C" fn Physics_PrintProfiling(this: &mut Physics) {
}

#[no_mangle]
pub unsafe extern "C" fn Physics_DrawBoundingBoxesLocal(this: &mut Physics) {
}

#[no_mangle]
pub unsafe extern "C" fn Physics_DrawBoundingBoxesWorld(this: &mut Physics) {
}

#[no_mangle]
pub unsafe extern "C" fn Physics_DrawTriggers(this: &mut Physics) {
}

#[no_mangle]
pub unsafe extern "C" fn Physics_DrawWireframes(this: &mut Physics) {
}
