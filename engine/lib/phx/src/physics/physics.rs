use std::cell::RefCell;
use std::thread::LocalKey;

use glam::Mat4;
use rapier3d_f64::parry::query::ShapeCastOptions;
use rapier3d_f64::prelude as rp;
use rapier3d_f64::prelude::nalgebra as na;

use crate::math::*;
use crate::physics::*;
use crate::render::*;
use crate::rf::Rf;

#[repr(C)]
pub struct Collision {
    index: u32,
    count: u32,
    body0: *mut RigidBody,
    body1: *mut RigidBody,
}

// The layouts of these result structs are declared again in `script/libphx.lua`.

/// The first body a ray hits; `body` is null and `id` is 0 when it hits nothing.
#[repr(C)]
pub struct RayCastResult {
    body: *mut RigidBody,
    norm: Vec3,
    pos: Position,
    t: f32,
    /// The hit body's user id.
    id: BodyUserId,
}

/// The bodies a shape overlaps, and their user ids in the same order.
#[repr(C)]
pub struct ShapeCastResult {
    hits: *const *mut RigidBody,
    hits_len: u32,
    ids: *const BodyUserId,
}

/// The first body a swept sphere touches; `body` is null and `id` is 0 when it touches nothing.
#[repr(C)]
pub struct SphereSweepResult {
    body: *mut RigidBody,
    /// The hit body's user id.
    id: BodyUserId,
    /// The hit body's outward surface normal at the contact.
    norm: Vec3,
    /// The contact point, on the hit body's surface.
    pos: Position,
    /// Distance in game units that the sphere's centre travels before the contact.
    t: f64,
}

/// The bodies within a radius, nearest first: their user ids and the distances in game
/// units from the query centre to their surfaces (0 inside).
#[repr(C)]
pub struct RadiusQueryResult {
    ids: *const BodyUserId,
    distances: *const f64,
    len: u32,
}

/// Storage of a shape cast's results, which Lua reads until the next cast that uses it.
#[derive(Default)]
struct ShapeHits {
    bodies: Vec<*mut RigidBody>,
    ids: Vec<BodyUserId>,
}

/// Storage of a radius query's results, which Lua reads until the next radius query.
#[derive(Default)]
struct RadiusHits {
    ids: Vec<BodyUserId>,
    distances: Vec<f64>,
}

fn ffi_len(len: usize) -> u32 {
    u32::try_from(len).expect("query result count fits in u32")
}

type ColliderPredicate = dyn Fn(rp::ColliderHandle, &rp::Collider) -> bool;

/// Accepts the colliders of bodies whose collision group shares a bit with `mask`. It tests
/// the group set on the body rather than Rapier's groups, which are empty for a
/// non-collidable body, so masked queries hit the same bodies as unmasked ones.
fn in_mask(mask: CollisionMask) -> impl Fn(rp::ColliderHandle, &rp::Collider) -> bool {
    move |_, collider| RigidBody::linked_collision_group(collider) & mask != 0
}

/// Rapier filter for queries with an excluded body and a collision mask: it skips every
/// collider of the compound `exclude` belongs to, and applies `in_mask`.
fn masked_filter<'a>(
    exclude: Option<&RigidBody>,
    in_mask: &'a ColliderPredicate,
) -> rp::QueryFilter<'a> {
    let filter = rp::QueryFilter {
        predicate: Some(in_mask),
        ..Default::default()
    };
    match exclude.and_then(RigidBody::get_rigid_body_handle) {
        Some(handle) => filter.exclude_rigid_body(handle),
        None => filter,
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
        Quat::new(v.i as f32, v.j as f32, v.k as f32, v.w as f32)
    }
}

pub trait RapierMatrixInterop {
    fn from_rp(_: &rp::Isometry<rp::Real>, frame: &Position) -> Self;
}

impl RapierMatrixInterop for Matrix {
    fn from_rp(t: &rp::Isometry<rp::Real>, frame: &Position) -> Matrix {
        Mat4::from_rotation_translation(
            *Quat::from_na(&t.rotation),
            Position::from_na(&t.translation.vector).relative_to(*frame),
        )
        .into()
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
    broad_phase: rp::BroadPhaseMultiSap,
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
            broad_phase: rp::BroadPhaseMultiSap::new(),
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

    /// Sets the typical object size in game units that Rapier scales its internal
    /// tolerances by, such as the allowed overlap and the speed below which a body may
    /// sleep. Rapier's default is 1. Panics unless positive.
    pub fn set_length_unit(&mut self, length_unit: f64) {
        assert!(length_unit > 0.0, "length unit must be positive");
        self.integration_parameters.length_unit = length_unit;
    }

    /// Sets the natural frequency of contact constraints, in hertz: higher values push
    /// overlapping bodies apart faster. Rapier's default is 30. Panics unless positive.
    pub fn set_contact_natural_frequency(&mut self, frequency: f64) {
        assert!(
            frequency > 0.0,
            "contact natural frequency must be positive"
        );
        self.integration_parameters.contact_natural_frequency = frequency;
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
        self.query_pipeline.update(&world.colliders);
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
            if c1_parent.is_none() || c2_parent.is_none() {
                continue;
            }

            iterator.count += 1;
            iterator.body0 = c1_parent.unwrap() as *mut RigidBody;
            iterator.body1 = c2_parent.unwrap() as *mut RigidBody;
            return true;
        }

        iterator.body0 = std::ptr::null_mut();
        iterator.body1 = std::ptr::null_mut();
        false
    }

    /// The first body the ray hits between `t_min` and `t_max`, including non-collidable ones.
    #[bind(out_param = true)]
    pub fn ray_cast(&self, ray: &Ray) -> RayCastResult {
        self.cast_ray(ray, rp::QueryFilter::default())
    }

    /// The first body the ray hits between `t_min` and `t_max`, skipping the compound
    /// `exclude` belongs to, and hitting only bodies whose collision group shares a bit
    /// with `mask`.
    #[bind(out_param = true)]
    pub fn ray_cast_filtered(
        &self,
        ray: &Ray,
        exclude: Option<&RigidBody>,
        mask: u32,
    ) -> RayCastResult {
        let in_mask = in_mask(mask);
        self.cast_ray(ray, masked_filter(exclude, &in_mask))
    }

    /// The bodies a sphere at `pos` overlaps, unsorted, including child bodies.
    ///
    /// The arrays in the result are valid until the next call to sphere_cast.
    #[bind(out_param = true)]
    pub fn sphere_cast(&self, pos: &Position, radius: f64) -> ShapeCastResult {
        thread_local! { static STORAGE: RefCell<ShapeHits> = Default::default(); }
        let hits = self.shape_cast(&rp::Ball::new(radius), pos, &Quat::identity());
        store_shape_hits(&STORAGE, hits)
    }

    /// The bodies a box at `pos` overlaps, unsorted, including child bodies.
    ///
    /// The arrays in the result are valid until the next call to box_cast.
    #[bind(out_param = true)]
    pub fn box_cast(&self, pos: &Position, rot: &Quat, half_extents: &Vec3) -> ShapeCastResult {
        thread_local! { static STORAGE: RefCell<ShapeHits> = Default::default(); }
        let hits = self.shape_cast(&rp::Cuboid::new(half_extents.to_na()), pos, rot);
        store_shape_hits(&STORAGE, hits)
    }

    pub fn sphere_overlap(&self, pos: &Position, radius: f64) -> bool {
        self.shape_overlap(&rp::Ball::new(radius), pos, &Quat::identity())
    }

    pub fn box_overlap(&self, pos: &Position, rot: &Quat, half_extents: &Vec3) -> bool {
        self.shape_overlap(&rp::Cuboid::new(half_extents.to_na()), pos, rot)
    }

    /// The first body a sphere of `radius` touches while its centre moves from `from` to
    /// `to`, skipping the compound `exclude` belongs to, and hitting only bodies whose
    /// collision group shares a bit with `mask`. A sphere that starts overlapping a body
    /// hits it at distance 0.
    #[bind(out_param = true)]
    pub fn sphere_sweep(
        &self,
        from: &Position,
        to: &Position,
        radius: f64,
        exclude: Option<&RigidBody>,
        mask: u32,
    ) -> SphereSweepResult {
        let motion = to.to_na() - from.to_na();
        let options = ShapeCastOptions {
            max_time_of_impact: 1.0,
            target_distance: 0.0,
            stop_at_penetration: true,
            compute_impact_geometry_on_penetration: true,
        };
        let in_mask = in_mask(mask);
        let world = self.world.as_ref();
        let hit = self.query_pipeline.cast_shape(
            &world.rigid_bodies,
            &world.colliders,
            &isometry(from, &Quat::identity()),
            &motion,
            &rp::Ball::new(radius),
            options,
            masked_filter(exclude, &in_mask),
        );
        let mut result = SphereSweepResult {
            body: std::ptr::null_mut(),
            id: 0,
            norm: Vec3::ZERO,
            pos: Position::ZERO,
            t: 0.0,
        };
        if let Some((handle, hit)) = hit {
            let collider = world.get(handle);
            if let Some(body) = RigidBody::linked_with_collider_mut(collider) {
                result.body = body;
                result.id = RigidBody::linked_user_id(collider);
                result.norm = Vec3::from_na(&hit.normal1);
                result.pos = Position::from_na_point(&hit.witness1);
                result.t = hit.time_of_impact * motion.norm();
            }
        }
        result
    }

    /// The bodies within `radius` of `pos`, nearest first, skipping the compound `exclude`
    /// belongs to, and including only bodies whose collision group shares a bit with `mask`.
    /// Child bodies are listed on their own; a body without a user id is listed with id 0.
    ///
    /// The arrays in the result are valid until the next call to query_radius.
    #[bind(out_param = true)]
    pub fn query_radius(
        &self,
        pos: &Position,
        radius: f64,
        exclude: Option<&RigidBody>,
        mask: u32,
    ) -> RadiusQueryResult {
        thread_local! { static STORAGE: RefCell<RadiusHits> = Default::default(); }

        let center = pos.to_na_point();
        let in_mask = in_mask(mask);
        let world = self.world.as_ref();
        let mut found: Vec<(BodyUserId, f64)> = vec![];
        self.query_pipeline.intersections_with_shape(
            &world.rigid_bodies,
            &world.colliders,
            &isometry(pos, &Quat::identity()),
            &rp::Ball::new(radius),
            masked_filter(exclude, &in_mask),
            |handle| {
                let collider = world.get(handle);
                if RigidBody::linked_with_collider(collider).is_some() {
                    let distance =
                        collider
                            .shape()
                            .distance_to_point(collider.position(), &center, true);
                    found.push((RigidBody::linked_user_id(collider), distance));
                }
                true
            },
        );
        found.sort_by(|a, b| a.1.total_cmp(&b.1));

        STORAGE.with_borrow_mut(|s| {
            s.ids = found.iter().map(|&(id, _)| id).collect();
            s.distances = found.iter().map(|&(_, distance)| distance).collect();
            RadiusQueryResult {
                ids: s.ids.as_ptr(),
                distances: s.distances.as_ptr(),
                len: ffi_len(s.ids.len()),
            }
        })
    }

    pub fn draw_bounding_boxes_local(&self) {}

    pub fn draw_bounding_boxes_world(&self) {}

    pub fn draw_wireframes(&mut self, r: &mut Renderer, shader: &mut Shader, eye: &Position) {
        self.draw_wireframes_in_range(r, shader, eye, f64::MAX);
    }

    pub fn draw_wireframes_in_range(
        &mut self,
        r: &mut Renderer,
        shader: &mut Shader,
        eye: &Position,
        max_range: f64,
    ) {
        let world = self.world.as_ref();
        self.debug_renderer.render(
            &mut RapierDebugRenderer {
                r,
                shader,
                eye: *eye,
                max_range_sq: max_range * max_range,
            },
            &world.rigid_bodies,
            &world.colliders,
            &self.impulse_joints,
            &self.multibody_joints,
            &world.narrow_phase,
        )
    }
}

fn isometry(pos: &Position, rot: &Quat) -> rp::Isometry<rp::Real> {
    rp::Isometry::from_parts(rp::Translation::from(pos.to_na()), rot.to_na())
}

/// Moves a shape cast's results into `storage`, where the returned pointers stay valid
/// until the next call with the same storage.
fn store_shape_hits(
    storage: &'static LocalKey<RefCell<ShapeHits>>,
    hits: ShapeHits,
) -> ShapeCastResult {
    storage.with_borrow_mut(|s| {
        *s = hits;
        ShapeCastResult {
            hits: s.bodies.as_ptr(),
            hits_len: ffi_len(s.bodies.len()),
            ids: s.ids.as_ptr(),
        }
    })
}

impl Physics {
    fn cast_ray(&self, ray: &Ray, filter: rp::QueryFilter) -> RayCastResult {
        let from = ray.get_point(ray.t_min).to_na_point();
        let to = ray.get_point(ray.t_max).to_na_point();
        let dir = to - from;
        let length = dir.norm();

        let ray = rp::Ray::new(from, dir / length);

        let mut result = RayCastResult {
            body: std::ptr::null_mut(),
            norm: Vec3::ZERO,
            pos: Position::ZERO,
            t: 0.0,
            id: 0,
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
                    result.id = RigidBody::linked_user_id(collider);
                    result.pos =
                        Position::from_na_point(&ray.point_at(intersection.time_of_impact));
                    result.norm = Vec3::from_na(&intersection.normal);
                    result.t = intersection.time_of_impact as f32;
                }
            }
        }
        result
    }

    /// All rigid bodies that overlap the shape at the given position and rotation.
    fn shape_cast(&self, shape: &dyn rp::Shape, pos: &Position, rot: &Quat) -> ShapeHits {
        let world = self.world.as_ref();
        let mut hits = ShapeHits::default();
        self.query_pipeline.intersections_with_shape(
            &world.rigid_bodies,
            &world.colliders,
            &isometry(pos, rot),
            shape,
            rp::QueryFilter::default(),
            |handle| {
                let collider = world.get(handle);
                if let Some(rigid_body) = RigidBody::linked_with_collider_mut(collider) {
                    hits.bodies.push(rigid_body);
                    hits.ids.push(RigidBody::linked_user_id(collider));
                }
                true
            },
        );
        hits
    }

    /// Returns true if any rigid bodies are contained within the shape at the
    /// given position and rotation.
    fn shape_overlap(&self, shape: &dyn rp::Shape, pos: &Position, rot: &Quat) -> bool {
        let rp_transform = isometry(pos, rot);
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

#[cfg(test)]
mod query_tests;
#[cfg(test)]
mod scale_tests;
#[cfg(test)]
mod test_fixture;

struct RapierDebugRenderer<'a> {
    r: &'a mut Renderer,
    shader: &'a mut Shader,
    eye: Position,
    max_range_sq: f64,
}

impl rp::DebugRenderBackend for RapierDebugRenderer<'_> {
    fn draw_line(
        &mut self,
        _object: rp::DebugRenderObject<'_>,
        start: rp::Point<rp::Real>,
        end: rp::Point<rp::Real>,
        _color: [f32; 4],
    ) {
        // Distance check: skip lines far from camera
        let mid_x = (start.x + end.x) * 0.5;
        let mid_y = (start.y + end.y) * 0.5;
        let mid_z = (start.z + end.z) * 0.5;
        let dx = mid_x - self.eye.v.x;
        let dy = mid_y - self.eye.v.y;
        let dz = mid_z - self.eye.v.z;
        let dist_sq = dx * dx + dy * dy + dz * dz;
        if dist_sq > self.max_range_sq {
            return;
        }

        // Green wireframe
        self.shader.set_float4(self.r, "color", 0.0, 1.0, 0.3, 0.8);
        Draw::line3(
            self.r,
            &Position::from_na_point(&start).relative_to(self.eye),
            &Position::from_na_point(&end).relative_to(self.eye),
        );
    }
}
