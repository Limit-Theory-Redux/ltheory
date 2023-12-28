#![allow(unused)]

use crate::math::*;
use crate::physics::*;
use crate::render::*;
use rapier3d::parry::query::RayCast;
use rapier3d::prelude as rp;
use rapier3d::prelude::nalgebra as na;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

#[repr(C)]
pub struct Collision {
    index: u32,
    count: u32,
    body0: *mut RigidBody,
    body1: *mut RigidBody,
}

#[repr(C)]
pub struct RayCastResult {
    body: *mut RigidBody,
    norm: Vec3,
    pos: Vec3,
    t: f32,
}

pub struct ShapeCastResult {
    hits: *const *mut RigidBody,
    hits_len: u32,
}

impl ShapeCastResult {
    pub fn get_hits(&self) -> &[*mut RigidBody] {
        unsafe { std::slice::from_raw_parts(self.hits, self.hits_len as usize) }
    }
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

pub trait RapierMatrixInterop {
    fn from_rp(_: &rp::Isometry<f32>) -> Self;
}

impl RapierMatrixInterop for Matrix {
    fn from_rp(t: &rp::Isometry<f32>) -> Matrix {
        Matrix::from_cols_slice(t.to_matrix().as_slice())
    }
}

pub(crate) struct PhysicsWorld {
    pub(crate) island_manager: rp::IslandManager,
    pub(crate) rigid_bodies: rp::RigidBodySet,
    pub(crate) colliders: rp::ColliderSet,
}

impl PhysicsWorld {
    pub fn get_rigid_body(&self, handle: rp::RigidBodyHandle) -> &rp::RigidBody {
        self.rigid_bodies.get(handle).unwrap()
    }

    pub fn get_rigid_body_mut(&mut self, handle: rp::RigidBodyHandle) -> &mut rp::RigidBody {
        self.rigid_bodies.get_mut(handle).unwrap()
    }

    pub fn get_collider(&self, handle: rp::ColliderHandle) -> &rp::Collider {
        self.colliders.get(handle).unwrap()
    }

    pub fn get_collider_mut(&mut self, handle: rp::ColliderHandle) -> &mut rp::Collider {
        self.colliders.get_mut(handle).unwrap()
    }
}

#[derive(Clone)]
pub(crate) struct PhysicsWorldHandle(Weak<RefCell<PhysicsWorld>>);

impl PhysicsWorldHandle {
    pub fn from_rc(rc: &Rc<RefCell<PhysicsWorld>>) -> PhysicsWorldHandle {
        PhysicsWorldHandle(Rc::downgrade(rc))
    }

    pub fn upgrade(&self) -> Rc<RefCell<PhysicsWorld>> {
        self.0.upgrade().expect("physics world was freed")
    }
}

/// Ray/shape casts/overlaps will return RigidBodys but not Triggers.
pub struct Physics {
    world: Rc<RefCell<PhysicsWorld>>,

    integration_parameters: rp::IntegrationParameters,
    physics_pipeline: rp::PhysicsPipeline,
    query_pipeline: rp::QueryPipeline,
    broad_phase: rp::BroadPhase,
    narrow_phase: rp::NarrowPhase,
    impulse_joints: rp::ImpulseJointSet,
    multibody_joints: rp::MultibodyJointSet,
    ccd_solver: rp::CCDSolver,

    triggers: Vec<Trigger>,

    rigid_body_map: HashMap<rp::RigidBodyHandle, *mut RigidBody>,

    debug_renderer: rp::DebugRenderPipeline,
}

#[luajit_ffi_gen::luajit_ffi(managed = true)]
impl Physics {
    #[bind(name = "Create")]
    pub fn new() -> Physics {
        Physics {
            world: Rc::new(RefCell::new(PhysicsWorld {
                island_manager: rp::IslandManager::new(),
                rigid_bodies: rp::RigidBodySet::new(),
                colliders: rp::ColliderSet::new(),
            })),
            integration_parameters: rp::IntegrationParameters::default(),
            physics_pipeline: rp::PhysicsPipeline::new(),
            query_pipeline: rp::QueryPipeline::new(),
            broad_phase: rp::BroadPhase::new(),
            narrow_phase: rp::NarrowPhase::new(),
            impulse_joints: rp::ImpulseJointSet::new(),
            multibody_joints: rp::MultibodyJointSet::new(),
            ccd_solver: rp::CCDSolver::new(),
            triggers: Vec::new(),
            rigid_body_map: HashMap::new(),
            debug_renderer: rp::DebugRenderPipeline::new(Default::default(), Default::default()),
        }
    }

    /// Adds this rigid body to this physics world if it doesn't exist, otherwise do nothing.
    ///
    /// Automatically adds all attached Triggers. Automatically adds all
    /// attached children and their Triggers.
    pub fn add_rigid_body(&mut self, rigid_body: &mut RigidBody) {
        if let Some((_, rb_handle)) = rigid_body.add_to_world(&self.world) {
            self.rigid_body_map
                .insert(rb_handle, rigid_body as *mut RigidBody);
        }
    }

    /// Removes this rigid body from this physics world if it's added, otherwise do nothing.
    ///
    /// Automatically removes all attached Triggers. Automatically removes all
    /// attached children and their Triggers.
    pub fn remove_rigid_body(&mut self, rigid_body: &mut RigidBody) {
        if let Some((_, rb_handle)) =
            rigid_body.remove_from_world(&mut self.impulse_joints, &mut self.multibody_joints)
        {
            self.rigid_body_map.remove(&rb_handle);
        }
    }

    pub fn add_trigger(&mut self, trigger: &mut Trigger) {}

    pub fn remove_trigger(&mut self, trigger: &mut Trigger) {}

    pub fn update(&mut self, dt: f32) {
        for trigger in self.triggers.iter_mut() {
            Trigger_Update(trigger);
        }

        let gravity = Vec3::ZERO.to_na();
        let physics_hooks = ();
        let event_handler = ();

        let mut integration_parameters = self.integration_parameters;
        integration_parameters.dt = dt;
        let world = &mut *self.world.borrow_mut();
        self.physics_pipeline.step(
            &gravity,
            &integration_parameters,
            &mut world.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut world.rigid_bodies,
            &mut world.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            None,
            &physics_hooks,
            &event_handler,
        );
        self.query_pipeline
            .update(&world.rigid_bodies, &world.colliders);
        for (_, rb) in world.rigid_bodies.iter_mut() {
            rb.reset_forces(false);
            rb.reset_torques(false);
        }
    }

    /// This will fill the collision object c with the collision information.
    ///
    /// Will include results for both child and parent RigidBodys that are
    /// colliding. Will not include Triggers.
    pub fn get_next_collision(&self, iterator: &mut Collision) -> bool {
        let collision_count = self
            .narrow_phase
            .contact_graph()
            .raw_graph()
            .raw_edges()
            .len();

        let world = &mut *self.world.borrow_mut();
        while (iterator.index as usize) < collision_count {
            let contact_pair = self
                .narrow_phase
                .contact_pair_at_index(rp::TemporaryInteractionIndex::new(iterator.index));
            iterator.index += 1;

            // Evaluate contact.
            let c1_parent = RigidBody::linked_with_collider_mut(
                world.colliders.get(contact_pair.collider1).unwrap(),
            );
            let c2_parent = RigidBody::linked_with_collider_mut(
                world.colliders.get(contact_pair.collider2).unwrap(),
            );
            if !c1_parent.is_some() || !c2_parent.is_some() {
                continue;
            }

            iterator.count += 1;
            iterator.body0 = c1_parent.unwrap() as *mut RigidBody;
            iterator.body1 = c2_parent.unwrap() as *mut RigidBody;
            return true;
        }

        iterator.body0 = std::ptr::null_mut();
        iterator.body1 = std::ptr::null_mut();
        return false;
    }

    #[bind(out_param = true)]
    pub fn ray_cast(&self, ray: &Ray) -> RayCastResult {
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

        let mut result = RayCastResult {
            body: std::ptr::null_mut(),
            norm: Vec3::ZERO,
            pos: Vec3::ZERO,
            t: 0.0,
        };
        let world = self.world.borrow();
        if let Some((handle, intersection)) = self.query_pipeline.cast_ray_and_get_normal(
            &world.rigid_bodies,
            &world.colliders,
            &ray,
            length,
            true,
            filter,
        ) {
            if let Some(collider) = world.colliders.get(handle) {
                if let Some(parent_rb) = RigidBody::linked_with_collider_mut(collider) {
                    result.body = parent_rb as *mut RigidBody;
                    result.pos = Vec3::from_na_point(&ray.point_at(intersection.toi));
                    result.norm = Vec3::from_na(&intersection.normal);
                    result.t = intersection.toi;
                }
            }
        }
        result
    }

    /// Results are unsorted and will include child objects.
    ///
    /// The array stored inside ShapeCastResult is valid until the next call to sphere_cast.
    #[bind(out_param = true)]
    pub fn sphere_cast(&self, sphere: &Sphere) -> ShapeCastResult {
        let result = self.shape_cast(&rp::Ball { radius: sphere.r }, sphere.p, Quat::IDENTITY);
        unsafe {
            static mut storage: Option<Box<[*mut RigidBody]>> = None;
            storage = Some(result.into_boxed_slice());
            ShapeCastResult {
                hits: storage.as_ref().unwrap().as_ptr(),
                hits_len: storage.as_ref().unwrap().len() as u32,
            }
        }
    }

    /// Results are unsorted and will include child objects.
    ///
    /// The array stored inside ShapeCastResult is valid until the next call to box_cast.
    #[bind(out_param = true)]
    pub fn box_cast(&self, pos: &Vec3, rot: &Quat, half_extents: &Vec3) -> ShapeCastResult {
        let result = self.shape_cast(
            &rp::Cuboid {
                half_extents: half_extents.to_na(),
            },
            *pos,
            *rot,
        );
        unsafe {
            static mut storage: Option<Box<[*mut RigidBody]>> = None;
            storage = Some(result.into_boxed_slice());
            ShapeCastResult {
                hits: storage.as_ref().unwrap().as_ptr(),
                hits_len: storage.as_ref().unwrap().len() as u32,
            }
        }
    }

    pub fn sphere_overlap(&self, sphere: &Sphere) -> bool {
        self.shape_overlap(&rp::Ball { radius: sphere.r }, sphere.p, Quat::IDENTITY)
    }

    pub fn box_overlap(&self, pos: &Vec3, rot: &Quat, half_extents: &Vec3) -> bool {
        self.shape_overlap(
            &rp::Cuboid {
                half_extents: half_extents.to_na(),
            },
            *pos,
            *rot,
        )
    }

    pub fn draw_bounding_boxes_local(&self) {}

    pub fn draw_bounding_boxes_world(&self) {}

    pub fn draw_triggers(&self) {}

    pub fn draw_wireframes(&mut self) {
        let world = self.world.borrow();
        self.debug_renderer.render(
            &mut RapierDebugRenderer,
            &world.rigid_bodies,
            &world.colliders,
            &self.impulse_joints,
            &self.multibody_joints,
            &self.narrow_phase,
        )
    }
}

impl Physics {
    /// Returns a list of all rigid bodies that are contained within the shape
    /// at the given position and rotation.
    fn shape_cast(&self, shape: &dyn rp::Shape, pos: Vec3, rot: Quat) -> Vec<*mut RigidBody> {
        let rp_transform =
            rp::Isometry::from_parts(rp::Translation::from(pos.to_na()), rot.to_na());
        let world = self.world.borrow();

        // Trigger scene query and populate results.
        let mut result: Vec<*mut RigidBody> = vec![];
        self.query_pipeline.intersections_with_shape(
            &world.rigid_bodies,
            &world.colliders,
            &rp_transform,
            shape,
            rp::QueryFilter::default(),
            |handle| {
                if let Some(rigid_body) =
                    RigidBody::linked_with_collider_mut(world.get_collider(handle))
                {
                    result.push(rigid_body as *mut RigidBody);
                }
                true
            },
        );

        result
    }

    /// Returns true if any rigid bodies are contained within the shape at the
    /// given position and rotation.
    fn shape_overlap(&self, shape: &dyn rp::Shape, pos: Vec3, rot: Quat) -> bool {
        let rp_transform =
            rp::Isometry::from_parts(rp::Translation::from(pos.to_na()), rot.to_na());
        let world = self.world.borrow();
        self.query_pipeline
            .intersection_with_shape(
                &world.rigid_bodies,
                &world.colliders,
                &rp_transform,
                shape,
                rp::QueryFilter::default(),
            )
            .is_some()
    }
}

struct RapierDebugRenderer;

impl rp::DebugRenderBackend for RapierDebugRenderer {
    fn draw_line(
        &mut self,
        object: rp::DebugRenderObject<'_>,
        a: rp::Point<rp::Real>,
        b: rp::Point<rp::Real>,
        color: [f32; 4],
    ) {
        unsafe {
            Draw_Color(color[0], color[1], color[2], color[3]);
            Draw_Line3(
                &Vec3::from_na_point(&a) as *const _,
                &Vec3::from_na_point(&b) as *const _,
            );
        }
    }
}
