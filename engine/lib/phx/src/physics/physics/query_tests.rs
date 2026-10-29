//! Physics queries: user ids in the results, the exclude and mask filters, f64 positions
//! far from the origin, sphere sweeps, radius queries, and the world tuning setters.

use super::test_fixture::{STEP, sphere_body};
use super::*;

const RADIUS: f64 = 0.003;
/// Far enough from the origin that single precision can't place a ship-sized body.
const FAR: f64 = 1.496e7 + 0.3;
const ALL: CollisionMask = CollisionMask::MAX;
/// Error allowed in query distances, as a fraction of `RADIUS`.
const TOLERANCE: f64 = 0.01;

/// The sphere radius the engine actually uses: `set_scale` takes single precision.
fn radius() -> f64 {
    f64::from(RADIUS as f32)
}

/// A world with a sphere of `RADIUS` at each x along the x axis, with user ids 1, 2, ...
/// in order, stepped once so the queries see them.
// The bodies are boxed so their addresses stay stable while registered with the world.
#[expect(
    clippy::vec_box,
    reason = "bodies must not move once added to the world"
)]
fn world_with_bodies_at(xs: &[f64]) -> (Physics, Vec<Box<RigidBody>>) {
    let mut physics = Physics::new();
    let mut bodies = vec![];
    for (i, &x) in xs.iter().enumerate() {
        let mut body = sphere_body(RADIUS, true);
        physics.add_rigid_body(&mut body);
        body.set_position(&Position::new(x, 0.0, 0.0));
        body.set_user_id(i as u64 + 1);
        bodies.push(body);
    }
    physics.update(STEP);
    (physics, bodies)
}

fn ray_along_x(from: f64, length: f64) -> Ray {
    Ray {
        p: Position::new(from, 0.0, 0.0),
        dir: DVec3::X,
        t_min: 0.0,
        t_max: length,
    }
}

fn shape_ids(result: &ShapeCastResult) -> Vec<u64> {
    #[allow(unsafe_code)]
    // Sound: `ids` points to `hits_len` ids in the query's storage, which no other query
    // replaces before this copy.
    let ids = unsafe { std::slice::from_raw_parts(result.ids, result.hits_len as usize) };
    ids.to_vec()
}

fn radius_hits(result: &RadiusQueryResult) -> Vec<(u64, f64)> {
    #[allow(unsafe_code)]
    // Sound: both arrays hold `len` entries in the query's storage, which no other query
    // replaces before this copy.
    let (ids, distances) = unsafe {
        (
            std::slice::from_raw_parts(result.ids, result.len as usize),
            std::slice::from_raw_parts(result.distances, result.len as usize),
        )
    };
    ids.iter().copied().zip(distances.iter().copied()).collect()
}

fn assert_near(actual: f64, expected: f64, what: &str) {
    let error = (actual - expected).abs();
    assert!(
        error <= TOLERANCE * radius(),
        "{what}: {actual}, expected {expected}"
    );
}

#[test]
fn a_body_reports_the_user_id_it_was_given() {
    let mut body = sphere_body(RADIUS, false);
    assert_eq!(body.get_user_id(), 0);
    body.set_user_id(u64::MAX);
    assert_eq!(body.get_user_id(), u64::MAX);
}

#[test]
fn ray_cast_reports_the_hit_body_and_its_user_id() {
    let (physics, bodies) = world_with_bodies_at(&[1.0]);
    let hit = physics.ray_cast(&ray_along_x(0.0, 2.0));
    assert!(std::ptr::eq(hit.body, &*bodies[0]));
    assert_eq!(hit.id, 1);
    assert_near(hit.pos.v.x, 1.0 - radius(), "hit position");

    let miss = physics.ray_cast(&ray_along_x(0.0, 0.5));
    assert!(miss.body.is_null());
    assert_eq!(miss.id, 0);
}

#[test]
fn filtered_ray_cast_skips_the_excluded_body() {
    let (physics, bodies) = world_with_bodies_at(&[1.0, 2.0]);
    let ray = ray_along_x(0.0, 3.0);
    assert_eq!(physics.ray_cast_filtered(&ray, None, ALL).id, 1);
    assert_eq!(physics.ray_cast_filtered(&ray, Some(&bodies[0]), ALL).id, 2);
}

#[test]
fn filtered_ray_cast_skips_bodies_outside_the_mask() {
    let (physics, mut bodies) = world_with_bodies_at(&[1.0, 2.0]);
    bodies[0].set_collision_group(0b10);
    bodies[1].set_collision_group(0b01);
    let ray = ray_along_x(0.0, 3.0);
    assert_eq!(physics.ray_cast_filtered(&ray, None, 0b01).id, 2);
    assert_eq!(physics.ray_cast_filtered(&ray, None, 0b10).id, 1);
    assert!(physics.ray_cast_filtered(&ray, None, 0b100).body.is_null());
}

#[test]
fn queries_hit_non_collidable_bodies() {
    let (physics, mut bodies) = world_with_bodies_at(&[1.0]);
    bodies[0].set_collidable(false);
    let ray = ray_along_x(0.0, 2.0);
    assert_eq!(physics.ray_cast(&ray).id, 1);
    assert_eq!(physics.ray_cast_filtered(&ray, None, ALL).id, 1);
    let from = Position::new(0.0, 0.0, 0.0);
    let to = Position::new(2.0, 0.0, 0.0);
    assert_eq!(physics.sphere_sweep(&from, &to, RADIUS, None, ALL).id, 1);
    let near = physics.query_radius(&Position::new(1.0, 0.0, 0.0), RADIUS, None, ALL);
    assert_eq!(radius_hits(&near).len(), 1);
}

#[test]
fn exclusion_skips_the_whole_compound() {
    let (mut physics, mut bodies) = world_with_bodies_at(&[1.0]);
    let mut child = sphere_body(RADIUS, false);
    child.set_user_id(7);
    // The offset is in parent radii: the child overlaps the parent's far side, so a ray that
    // skipped only the parent would hit it.
    bodies[0].attach(&mut child, &Vec3::new(1.0, 0.0, 0.0), &Quat::identity());
    physics.update(STEP);

    let ray = ray_along_x(0.0, 3.0);
    assert_eq!(physics.ray_cast(&ray).id, 1);
    assert!(
        physics
            .ray_cast_filtered(&ray, Some(&bodies[0]), ALL)
            .body
            .is_null()
    );
    assert!(
        physics
            .ray_cast_filtered(&ray, Some(&child), ALL)
            .body
            .is_null()
    );
}

#[test]
fn sphere_and_box_casts_report_ids_at_system_distances() {
    let (physics, _bodies) = world_with_bodies_at(&[FAR]);
    let probe = 2.0 / 3.0 * RADIUS;
    let half_extents = Vec3::splat(probe as f32);
    // Centred just outside the body's surface, the probes overlap it by a third of its radius.
    let touching = Position::new(FAR + RADIUS + probe / 2.0, 0.0, 0.0);
    // Centred a third of a radius further out, they miss it by a sixth.
    let clear = Position::new(FAR + RADIUS + probe * 1.25, 0.0, 0.0);

    assert_eq!(shape_ids(&physics.sphere_cast(&touching, probe)), [1]);
    assert!(shape_ids(&physics.sphere_cast(&clear, probe)).is_empty());
    assert!(physics.sphere_overlap(&touching, probe));
    assert!(!physics.sphere_overlap(&clear, probe));

    let rot = Quat::identity();
    assert_eq!(
        shape_ids(&physics.box_cast(&touching, &rot, &half_extents)),
        [1]
    );
    assert!(shape_ids(&physics.box_cast(&clear, &rot, &half_extents)).is_empty());
    assert!(physics.box_overlap(&touching, &rot, &half_extents));
    assert!(!physics.box_overlap(&clear, &rot, &half_extents));
}

#[test]
fn sphere_sweep_reports_the_travel_distance_and_the_contact() {
    let (physics, _bodies) = world_with_bodies_at(&[FAR]);
    let sweep_radius = 2.0 * RADIUS;
    let from = Position::new(FAR - 10.0, 0.0, 0.0);
    let to = Position::new(FAR + 10.0, 0.0, 0.0);
    let hit = physics.sphere_sweep(&from, &to, sweep_radius, None, ALL);
    assert_eq!(hit.id, 1);
    assert_near(hit.t, 10.0 - radius() - sweep_radius, "travel distance");
    assert_near(hit.pos.v.x, FAR - radius(), "contact point");
    assert!(hit.norm.x < -0.99, "normal {:?}", hit.norm);
}

#[test]
fn sphere_sweep_stops_at_its_end_point() {
    let (physics, _bodies) = world_with_bodies_at(&[1.0]);
    let from = Position::new(0.0, 0.0, 0.0);
    let short = Position::new(0.5, 0.0, 0.0);
    let miss = physics.sphere_sweep(&from, &short, RADIUS, None, ALL);
    assert!(miss.body.is_null());
    assert_eq!(miss.id, 0);
}

#[test]
fn sphere_sweep_that_starts_inside_a_body_hits_it_at_zero() {
    let (physics, _bodies) = world_with_bodies_at(&[1.0]);
    let from = Position::new(1.0, 0.0, 0.0);
    let to = Position::new(2.0, 0.0, 0.0);
    let hit = physics.sphere_sweep(&from, &to, RADIUS, None, ALL);
    assert_eq!(hit.id, 1);
    assert_eq!(hit.t, 0.0);
}

#[test]
fn sphere_sweep_applies_the_exclude_and_mask_filters() {
    let (physics, mut bodies) = world_with_bodies_at(&[1.0, 2.0, 3.0]);
    bodies[1].set_collision_group(0b10);
    let from = Position::new(0.0, 0.0, 0.0);
    let to = Position::new(4.0, 0.0, 0.0);
    let hit = physics.sphere_sweep(&from, &to, RADIUS, Some(&bodies[0]), 0b01);
    assert_eq!(hit.id, 3);
}

#[test]
fn radius_query_lists_ids_nearest_first_with_surface_distances() {
    let (physics, _bodies) = world_with_bodies_at(&[FAR + 2.0, FAR + 1.0, FAR + 3.0, FAR]);
    let hits = radius_hits(&physics.query_radius(&Position::new(FAR, 0.0, 0.0), 2.5, None, ALL));
    let ids: Vec<u64> = hits.iter().map(|&(id, _)| id).collect();
    assert_eq!(ids, [4, 2, 1]);
    assert_eq!(hits[0].1, 0.0, "the centre is inside body 4");
    assert_near(hits[1].1, 1.0 - radius(), "distance to body 2");
    assert_near(hits[2].1, 2.0 - radius(), "distance to body 1");
}

#[test]
fn radius_query_applies_the_exclude_and_mask_filters() {
    let (physics, mut bodies) = world_with_bodies_at(&[1.0, 2.0, 3.0]);
    bodies[1].set_collision_group(0b10);
    let center = Position::new(0.0, 0.0, 0.0);
    let hits = radius_hits(&physics.query_radius(&center, 5.0, Some(&bodies[0]), 0b01));
    let ids: Vec<u64> = hits.iter().map(|&(id, _)| id).collect();
    assert_eq!(ids, [3]);
}

#[test]
fn tuning_setters_set_the_integration_parameters() {
    let mut physics = Physics::new();
    physics.set_length_unit(1e-2);
    physics.set_contact_natural_frequency(1920.0);
    assert_eq!(physics.integration_parameters.length_unit, 1e-2);
    assert_eq!(
        physics.integration_parameters.contact_natural_frequency,
        1920.0
    );
}

#[test]
#[should_panic(expected = "length unit must be positive")]
fn length_unit_must_be_positive() {
    Physics::new().set_length_unit(0.0);
}

#[test]
#[should_panic(expected = "contact natural frequency must be positive")]
fn contact_natural_frequency_must_be_positive() {
    Physics::new().set_contact_natural_frequency(-1.0);
}
