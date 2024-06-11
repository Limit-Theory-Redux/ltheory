#![allow(unused)]

use crate::math::*;
use crate::physics::*;
use crate::render::*;
use crate::rf::Rf;
use rapier3d_f64::parry::query::RayCast;
use rapier3d_f64::prelude as rp;
use rapier3d_f64::prelude::nalgebra as na;
use std::ffi::{CStr, CString};

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
    pos: Position,
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

pub trait NalgebraVecInterop {
    fn to_na(&self) -> na::Vector3<rp::Real>;
    fn to_na_point(&self) -> na::Point3<rp::Real>;
    fn from_na(_: &na::Vector3<rp::Real>) -> Self;
    fn from_na_point(_: &na::Point3<rp::Real>) -> Self;
}

impl NalgebraVecInterop for Vec3 {
    fn to_na(&self) -> na::Vector3<rp::Real> {
        na::Vector3::new(self.x as rp::Real, self.y as rp::Real, self.z as rp::Real)
    }
    fn to_na_point(&self) -> na::Point3<rp::Real> {
        na::Point3::new(self.x as rp::Real, self.y as rp::Real, self.z as rp::Real)
    }
    fn from_na(v: &na::Vector3<rp::Real>) -> Vec3 {
        Vec3::new(v.x as f32, v.y as f32, v.z as f32)
    }
    fn from_na_point(v: &na::Point3<rp::Real>) -> Vec3 {
        Vec3::new(v.x as f32, v.y as f32, v.z as f32)
    }
}

impl NalgebraVecInterop for Position {
    fn to_na(&self) -> na::Vector3<rp::Real> {
        na::Vector3::new(self.v.x, self.v.y, self.v.z)
    }
    fn to_na_point(&self) -> na::Point3<rp::Real> {
        na::Point3::new(self.v.x, self.v.y, self.v.z)
    }
    fn from_na(v: &na::Vector3<rp::Real>) -> Position {
        Position::new(v.x, v.y, v.z)
    }
    fn from_na_point(v: &na::Point3<rp::Real>) -> Position {
        Position::new(v.x, v.y, v.z)
    }
}

pub trait NalgebraQuatInterop {
    fn to_na(&self) -> na::UnitQuaternion<rp::Real>;
    fn from_na(_: &na::UnitQuaternion<rp::Real>) -> Self;
}

impl NalgebraQuatInterop for Quat {
    fn to_na(&self) -> na::UnitQuaternion<rp::Real> {
        na::UnitQuaternion::from_quaternion(na::Quaternion::new(
            self.w as rp::Real,
            self.x as rp::Real,
            self.y as rp::Real,
            self.z as rp::Real,
        ))
    }
    fn from_na(v: &na::UnitQuaternion<rp::Real>) -> Quat {
        Quat_Create(v.i as f32, v.j as f32, v.k as f32, v.w as f32)
    }
}

pub trait RapierMatrixInterop {
    fn from_rp(_: &rp::Isometry<rp::Real>, frame: &Position) -> Self;
}

impl RapierMatrixInterop for Matrix {
    fn from_rp(t: &rp::Isometry<rp::Real>, frame: &Position) -> Matrix {
        Matrix::from_rotation_translation(
            Quat::from_na(&t.rotation),
            Position::from_na(&t.translation.vector).relative_to(*frame),
        )
    }
}

pub(crate) struct PhysicsWorld {
    pub island_manager: rp::IslandManager,
    pub rigid_bodies: rp::RigidBodySet,
    pub colliders: rp::ColliderSet,
    pub narrow_phase: rp::NarrowPhase,
}

// All Rapier handles are Copy
pub(crate) trait RapierHandle: Copy {
    type Object;

    fn invalid() -> Self;
    fn lookup_object<'a>(&self, world: &'a PhysicsWorld) -> &'a Self::Object;
    fn lookup_object_mut<'a>(&self, world: &'a mut PhysicsWorld) -> &'a mut Self::Object;
}

impl RapierHandle for rp::ColliderHandle {
    type Object = rp::Collider;

    fn invalid() -> rp::ColliderHandle {
        rp::ColliderHandle::invalid()
    }

    fn lookup_object<'a>(&self, world: &'a PhysicsWorld) -> &'a rp::Collider {
        world.colliders.get(*self).unwrap()
    }

    fn lookup_object_mut<'a>(&self, world: &'a mut PhysicsWorld) -> &'a mut rp::Collider {
        world.colliders.get_mut(*self).unwrap()
    }
}

impl RapierHandle for rp::RigidBodyHandle {
    type Object = rp::RigidBody;

    fn invalid() -> rp::RigidBodyHandle {
        rp::RigidBodyHandle::invalid()
    }

    fn lookup_object<'a>(&self, world: &'a PhysicsWorld) -> &'a rp::RigidBody {
        world.rigid_bodies.get(*self).unwrap()
    }

    fn lookup_object_mut<'a>(&self, world: &'a mut PhysicsWorld) -> &'a mut rp::RigidBody {
        world.rigid_bodies.get_mut(*self).unwrap()
    }
}

impl PhysicsWorld {
    pub fn get<H: RapierHandle>(&self, handle: H) -> &H::Object {
        handle.lookup_object(self)
    }

    pub fn get_mut<H: RapierHandle>(&mut self, handle: H) -> &mut H::Object {
        handle.lookup_object_mut(self)
    }
}

/// Ray/shape casts/overlaps will return RigidBodys but not Triggers.
pub struct Physics {
    world: Rf<PhysicsWorld>,

    integration_parameters: rp::IntegrationParameters,
    physics_pipeline: rp::PhysicsPipeline,
    query_pipeline: rp::QueryPipeline,
    broad_phase: rp::BroadPhase,
    impulse_joints: rp::ImpulseJointSet,
    multibody_joints: rp::MultibodyJointSet,
    ccd_solver: rp::CCDSolver,

    debug_renderer: rp::DebugRenderPipeline,
}

#[luajit_ffi_gen::luajit_ffi]
impl Physics {
    #[bind(name = "Create")]
    pub fn new() -> Physics {
        Physics {
            world: Rf::new(PhysicsWorld {
                island_manager: rp::IslandManager::new(),
                rigid_bodies: rp::RigidBodySet::new(),
                colliders: rp::ColliderSet::new(),
                narrow_phase: rp::NarrowPhase::new(),
            }),
            integration_parameters: rp::IntegrationParameters::default(),
            physics_pipeline: rp::PhysicsPipeline::new(),
            query_pipeline: rp::QueryPipeline::new(),
            broad_phase: rp::BroadPhase::new(),
            impulse_joints: rp::ImpulseJointSet::new(),
            multibody_joints: rp::MultibodyJointSet::new(),
            ccd_solver: rp::CCDSolver::new(),
            debug_renderer: rp::DebugRenderPipeline::new(
                rp::DebugRenderStyle::default(),
                rp::DebugRenderMode::COLLIDER_SHAPES
                    | rp::DebugRenderMode::RIGID_BODY_AXES
                    | rp::DebugRenderMode::CONTACTS,
            ),
        }
    }

    /// Adds this rigid body to this physics world if it doesn't exist, otherwise do nothing.
    ///
    /// Automatically adds all attached Triggers. Automatically adds all
    /// attached children and their Triggers.
    pub fn add_rigid_body(&mut self, rigid_body: &mut RigidBody) {
        rigid_body.add_to_world(self.world.clone());
    }

    /// Removes this rigid body from this physics world if it's added, otherwise do nothing.
    ///
    /// Automatically removes all attached Triggers. Automatically removes all
    /// attached children and their Triggers.
    pub fn remove_rigid_body(&mut self, rigid_body: &mut RigidBody) {
        rigid_body.remove_from_world(&mut self.impulse_joints, &mut self.multibody_joints);
    }

    pub fn add_trigger(&mut self, trigger: &mut Trigger) {
        trigger.add_to_world(self.world.clone());
    }

    pub fn remove_trigger(&mut self, trigger: &mut Trigger) {
        trigger.remove_from_world();
    }

    pub fn update(&mut self, dt: f32) {
        let gravity = Vec3::ZERO.to_na();
        let physics_hooks = ();
        let event_handler = ();

        let mut integration_parameters = self.integration_parameters;
        integration_parameters.dt = dt as rp::Real;
        let world = &mut *self.world.as_mut();
        self.physics_pipeline.step(
            &gravity,
            &integration_parameters,
            &mut world.island_manager,
            &mut self.broad_phase,
            &mut world.narrow_phase,
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

    /// This will fill the collision object with the collision information.
    ///
    /// Will include results for both child and parent RigidBodys that are
    /// colliding. Will not include Triggers.
    pub fn get_next_collision(&self, iterator: &mut Collision) -> bool {
        let world = &*self.world.as_ref();

        let collision_count = world
            .narrow_phase
            .contact_graph()
            .raw_graph()
            .raw_edges()
            .len();

        while (iterator.index as usize) < collision_count {
            let contact_pair = world
                .narrow_phase
                .contact_pair_at_index(rp::TemporaryInteractionIndex::new(iterator.index));
            iterator.index += 1;

            // Evaluate contact.
            let c1_parent = world
                .colliders
                .get(contact_pair.collider1)
                .and_then(RigidBody::linked_with_collider_mut);
            let c2_parent = world
                .colliders
                .get(contact_pair.collider2)
                .and_then(RigidBody::linked_with_collider_mut);
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
            let mut data = Position::ZERO;
            Ray_GetPoint(ray, ray.tMin, &mut data);
            data.to_na_point()
        };
        let to = {
            let mut data = Position::ZERO;
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
            pos: Position::ZERO,
            t: 0.0,
        };
        let world = self.world.as_ref();
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
                    result.body = parent_rb;
                    result.pos = Position::from_na_point(&ray.point_at(intersection.toi));
                    result.norm = Vec3::from_na(&intersection.normal);
                    result.t = intersection.toi as f32;
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
        let result = self.shape_cast(
            &rp::Ball {
                radius: sphere.r as rp::Real,
            },
            sphere.p,
            Quat::IDENTITY,
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
        self.shape_overlap(
            &rp::Ball {
                radius: sphere.r as rp::Real,
            },
            sphere.p,
            Quat::IDENTITY,
        )
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

    pub fn draw_wireframes(&mut self) {
        let world = self.world.as_ref();
        self.debug_renderer.render(
            &mut RapierDebugRenderer,
            &world.rigid_bodies,
            &world.colliders,
            &self.impulse_joints,
            &self.multibody_joints,
            &world.narrow_phase,
        )
    }
}

impl Physics {
    /// Returns a list of all rigid bodies that are contained within the shape
    /// at the given position and rotation.
    fn shape_cast(&self, shape: &dyn rp::Shape, pos: Vec3, rot: Quat) -> Vec<*mut RigidBody> {
        let rp_transform =
            rp::Isometry::from_parts(rp::Translation::from(pos.to_na()), rot.to_na());
        let world = self.world.as_ref();

        // Trigger scene query and populate results.
        let mut result: Vec<*mut RigidBody> = vec![];
        self.query_pipeline.intersections_with_shape(
            &world.rigid_bodies,
            &world.colliders,
            &rp_transform,
            shape,
            rp::QueryFilter::default(),
            |handle| {
                if let Some(rigid_body) = RigidBody::linked_with_collider_mut(world.get(handle)) {
                    result.push(rigid_body);
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
        let world = self.world.as_ref();
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
        start: rp::Point<rp::Real>,
        end: rp::Point<rp::Real>,
        color: [f32; 4],
    ) {
        let Color { r, g, b, a } = Color::from_hsl(color[0], color[1], color[2], color[3]);

        unsafe {
            Shader_SetFloat4(
                CString::new("color").unwrap().as_c_str().as_ptr(),
                r,
                g,
                b,
                a,
            );
            Draw_Line3(
                &Vec3::from_na_point(&start) as *const _,
                &Vec3::from_na_point(&end) as *const _,
            );
        }
    }
}
