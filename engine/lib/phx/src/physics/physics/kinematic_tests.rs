//! Kinematic targets: a body moved with `set_kinematic_target` reaches the target after the
//! next step and has the velocity of that move, which a teleport doesn't give it.

use super::test_fixture::{STEP, sphere_body};
use super::*;

const RADIUS: f64 = 1.0;
/// How far a body moves in one step, in game units.
const MOVE: f64 = 0.1;
/// Angle turned in one step, in radians.
const TURN: f32 = 0.05;
/// Error allowed in velocities, as a fraction of the expected value.
const TOLERANCE: f64 = 1e-3;
/// Steps a kinematic body pushes another one for.
const PUSH_STEPS: usize = 30;

fn step_seconds() -> f64 {
    f64::from(STEP)
}

/// A kinematic sphere at the origin, in a world.
fn kinematic_in_world() -> (Physics, Box<RigidBody>) {
    let mut physics = Physics::new();
    let mut body = sphere_body(RADIUS, true);
    physics.add_rigid_body(&mut body);
    body.set_position(&Position::ZERO);
    physics.update(STEP);
    (physics, body)
}

fn assert_near(actual: f64, expected: f64, what: &str) {
    assert!(
        (actual - expected).abs() <= TOLERANCE * expected.abs(),
        "{what}: {actual}, expected {expected}"
    );
}

#[test]
fn a_kinematic_target_is_reached_after_the_step_and_not_before() {
    let (mut physics, mut body) = kinematic_in_world();
    body.set_kinematic_target(&Position::new(MOVE, 0.0, 0.0), &Quat::identity());
    assert_eq!(body.get_position().v.x, 0.0);

    physics.update(STEP);
    assert_near(body.get_position().v.x, MOVE, "position after the step");
}

#[test]
fn a_kinematic_target_gives_the_body_the_velocity_of_the_move() {
    let (mut physics, mut body) = kinematic_in_world();
    body.set_kinematic_target(&Position::new(MOVE, 0.0, 0.0), &Quat::identity());
    physics.update(STEP);

    let velocity = body.get_velocity();
    assert_near(f64::from(velocity.x), MOVE / step_seconds(), "velocity");
    assert_eq!((velocity.y, velocity.z), (0.0, 0.0));
}

#[test]
fn a_kinematic_target_gives_the_body_the_angular_velocity_of_the_turn() {
    let (mut physics, mut body) = kinematic_in_world();
    let turn = Quat::from_axis_angle(&Vec3::new(0.0, 1.0, 0.0), TURN);
    body.set_kinematic_target(&Position::ZERO, &turn);
    physics.update(STEP);

    assert_near(
        f64::from(body.get_angular_velocity().y),
        f64::from(TURN) / step_seconds(),
        "angular velocity",
    );
}

#[test]
fn a_teleport_leaves_the_body_without_velocity() {
    let (mut physics, mut body) = kinematic_in_world();
    body.set_position(&Position::new(MOVE, 0.0, 0.0));
    physics.update(STEP);

    assert_eq!(body.get_position().v.x, MOVE);
    assert_eq!(body.get_speed(), 0.0);
}

#[test]
fn a_moving_kinematic_body_pushes_a_dynamic_one() {
    let (mut physics, mut pusher) = kinematic_in_world();
    let mut pushed = sphere_body(RADIUS, false);
    physics.add_rigid_body(&mut pushed);
    let start_x = 2.0 * RADIUS + MOVE;
    pushed.set_position(&Position::new(start_x, 0.0, 0.0));

    for step in 1..=PUSH_STEPS {
        pusher.set_kinematic_target(
            &Position::new(step as f64 * MOVE, 0.0, 0.0),
            &Quat::identity(),
        );
        physics.update(STEP);
    }

    assert!(
        pushed.get_position().v.x > start_x + MOVE,
        "the pushed body didn't move"
    );
    assert!(pushed.get_velocity().x > 0.0);
}

#[test]
#[should_panic(expected = "needs a kinematic body")]
fn a_dynamic_body_cannot_take_a_kinematic_target() {
    let mut body = sphere_body(RADIUS, false);
    body.set_kinematic_target(&Position::ZERO, &Quat::identity());
}
