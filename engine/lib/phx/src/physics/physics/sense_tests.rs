//! `PhysicsWorld::sense_ball`: which root bodies a ball finds, and the sphere it gives each.

use std::collections::{BTreeMap, HashSet};

use super::test_fixture::{STEP, sphere_body};
use super::*;

const ALL: CollisionMask = CollisionMask::MAX;
const BODY_RADIUS: f64 = 2.0;
const PART_OFFSET: f32 = 10.0;
const BALL: f64 = 5.0;
const FAR: f64 = 100.0;

/// A sphere body in the world at `x`, which the next `update` makes visible to queries.
fn body_at(physics: &mut Physics, x: f64) -> Box<RigidBody> {
    let mut body = sphere_body(BODY_RADIUS, true);
    physics.add_rigid_body(&mut body);
    body.set_position(&Position::new(x, 0.0, 0.0));
    body
}

fn key_of(physics: &Physics, body: &RigidBody) -> BodyKey {
    body.handles_in(&physics.world())
        .expect("a root body in this world")
        .key()
}

fn sense_within(
    physics: &Physics,
    x: f64,
    ball: f64,
    mask: CollisionMask,
    skip: &HashSet<BodyKey>,
) -> BTreeMap<BodyKey, SensedBody> {
    let mut found = BTreeMap::new();
    physics
        .world()
        .as_ref()
        .sense_ball(DVec3::new(x, 0.0, 0.0), ball, mask, skip, &mut found);
    found
}

fn sense(
    physics: &Physics,
    x: f64,
    mask: CollisionMask,
    skip: &HashSet<BodyKey>,
) -> BTreeMap<BodyKey, SensedBody> {
    sense_within(physics, x, BALL, mask, skip)
}

#[test]
fn a_ball_finds_the_bodies_it_reaches_with_their_state() {
    let mut physics = Physics::new();
    let near = body_at(&mut physics, 0.0);
    let reached_at = BALL + BODY_RADIUS - 0.5;
    let reached = body_at(&mut physics, reached_at);
    let _far = body_at(&mut physics, FAR);
    physics.update(STEP);
    let found = sense(&physics, 0.0, ALL, &HashSet::new());
    let mut expected = vec![key_of(&physics, &near), key_of(&physics, &reached)];
    expected.sort_unstable();
    assert_eq!(
        found.keys().copied().collect::<Vec<_>>(),
        expected,
        "a body whose surface is in the ball counts"
    );
    let sphere = found[&key_of(&physics, &reached)];
    assert_eq!(sphere.position, DVec3::new(reached_at, 0.0, 0.0));
    assert_eq!(sphere.radius, BODY_RADIUS);
}

#[test]
fn bodies_outside_the_mask_or_in_the_skip_set_are_left_out() {
    let mut physics = Physics::new();
    let mut hidden = body_at(&mut physics, 0.0);
    let skipped = body_at(&mut physics, 1.0);
    let kept = body_at(&mut physics, 2.0);
    hidden.set_collision_group(0b10);
    physics.update(STEP);
    let skip = HashSet::from([key_of(&physics, &skipped)]);
    let found = sense(&physics, 0.0, 0b01, &skip);
    assert_eq!(
        found.keys().copied().collect::<Vec<_>>(),
        [key_of(&physics, &kept)]
    );
    assert!(sense(&physics, 0.0, 0, &HashSet::new()).is_empty());
}

#[test]
fn a_compound_is_one_body_whose_sphere_covers_its_parts() {
    let mut physics = Physics::new();
    let mut root = sphere_body(BODY_RADIUS, true);
    let mut part = sphere_body(BODY_RADIUS, true);
    root.attach(
        &mut part,
        &Vec3::new(PART_OFFSET, 0.0, 0.0),
        &Quat::identity(),
    );
    physics.add_rigid_body(&mut root);
    physics.update(STEP);
    // A child's offset is scaled by the parent's scale, which is its radius here.
    let part_at = f64::from(PART_OFFSET) * BODY_RADIUS;
    // The ball reaches the part only, and finds the whole body.
    let by_part = sense(&physics, part_at, ALL, &HashSet::new());
    assert_eq!(by_part.len(), 1);
    let sphere = by_part[&key_of(&physics, &root)];
    assert_eq!(sphere.position, DVec3::ZERO);
    assert_eq!(sphere.radius, part_at + BODY_RADIUS);
    // A ball that reaches both parts finds one body.
    let by_both = sense_within(&physics, part_at / 2.0, part_at, ALL, &HashSet::new());
    assert_eq!(by_both.len(), 1);
}

#[test]
fn a_trigger_is_not_a_body() {
    let mut physics = Physics::new();
    let mut trigger = Trigger::create_box(&Vec3::new(1.0, 1.0, 1.0));
    physics.add_trigger(&mut trigger);
    physics.update(STEP);
    assert!(sense(&physics, 0.0, ALL, &HashSet::new()).is_empty());
}

#[test]
fn a_removed_body_is_found_no_more() {
    let mut physics = Physics::new();
    let mut body = body_at(&mut physics, 0.0);
    physics.update(STEP);
    assert_eq!(sense(&physics, 0.0, ALL, &HashSet::new()).len(), 1);
    physics.remove_rigid_body(&mut body);
    physics.update(STEP);
    assert!(sense(&physics, 0.0, ALL, &HashSet::new()).is_empty());
}

#[test]
fn a_body_added_after_the_last_update_is_found_after_the_next() {
    let mut physics = Physics::new();
    physics.update(STEP);
    let _body = body_at(&mut physics, 0.0);
    assert!(sense(&physics, 0.0, ALL, &HashSet::new()).is_empty());
    physics.update(STEP);
    assert_eq!(sense(&physics, 0.0, ALL, &HashSet::new()).len(), 1);
}
